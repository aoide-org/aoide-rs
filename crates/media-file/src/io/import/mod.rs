// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    borrow::Cow,
    io::{Cursor, Read, Seek},
    path::Path,
    result::Result as StdResult,
};

use anyhow::anyhow;
use bitflags::bitflags;
use image::{DynamicImage, ImageReader};
use itertools::Itertools as _;
use lofty::{
    config::ParseOptions,
    file::{AudioFile, FileType},
    probe::Probe,
};
use mime::Mime;
use nonicle::{Canonical, CanonicalizeInto as _};
use semval::prelude::*;
use url::Url;

use aoide_core::{
    PlainTag, TagFacetKey, TagLabel, TagScore, TagsMap,
    audio::signal::LoudnessLufs,
    media::{
        Content, Source,
        artwork::{ApicType, Artwork, ArtworkImage, EmbeddedArtwork, LinkedArtwork},
        content::{ContentLink, ContentMetadata},
    },
    music::{key::KeySignature, tempo::TempoBpm},
    tag::ScoreValue,
    track::{
        Actors, Track,
        actor::{self, Actor},
        title::Title,
    },
    util::clock::{DateOrDateTime, OffsetDateTimeMs},
};

use crate::{
    Error, Result,
    fmt::parse_options,
    util::{
        db2lufs,
        digest::MediaDigest,
        parse_key_signature, parse_replay_gain_db, parse_year_tag,
        tag::{FacetedTagMappingConfig, TagMappingConfig},
        trim_readable,
    },
};

#[rustfmt::skip]
bitflags! {
    /// Flags for controlling the import
    ///
    /// It is recommended to enable all for maximum information and
    /// maximum compatibility.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ImportTrackFlags: u16 {
        /// Import metadata
        ///
        /// Import metadata from file tags like ID3 frames, MPEG4 atoms,
        /// or Vorbis Comments.
        const METADATA                                          = 0b0000_0000_0000_0001;

        /// Import embedded artwork
        ///
        /// Imports a single cover image embedded in the metadata.
        const METADATA_EMBEDDED_ARTWORK                         = 0b0000_0000_0000_0010;

        /// Hash cover image
        const METADATA_EMBEDDED_ARTWORK_DIGEST                  = 0b0000_0000_0000_0100;

        /// Use Apple GRP1/TIT1 instead of TIT1/TXXX:WORK ID3v2 frames for Content Group
        /// and Work Title respectively.
        ///
        /// Use the mapping for grouping and work fields as introduced by iTunes v12.5.4.
        /// This is the preferred mapping and existing files that still use TIT1 instead
        /// of GRP1 for storing the grouping property should be updated accordingly.
        const COMPATIBILITY_ID3V2_APPLE_GRP1                    = 0b0000_0001_0000_0000;

        #[cfg(feature = "gigtag")]
        /// Import gigtags from Content Group/Grouping file tag
        const GIGTAGS_CGRP                                      = 0b0001_0000_0000_0000;

        #[cfg(feature = "gigtag")]
        /// Import gigtags from Comment file tag
        const GIGTAGS_COMM                                      = 0b0010_0000_0000_0000;

        #[cfg(feature = "serato-markers")]
        /// Import metadata (cue points, loops, track color) from Serato file tags
        const SERATO_MARKERS                                    = 0b0100_0000_0000_0000;
    }
}

impl ImportTrackFlags {
    #[must_use]
    pub const fn is_valid(self) -> bool {
        Self::all().contains(self)
    }

    #[must_use]
    pub fn new_artwork_digest(self) -> MediaDigest {
        if self.contains(Self::METADATA_EMBEDDED_ARTWORK_DIGEST) {
            MediaDigest::new()
        } else {
            MediaDigest::dummy()
        }
    }
}

impl Default for ImportTrackFlags {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportTrackConfig {
    pub faceted_tag_mapping: FacetedTagMappingConfig,
    pub flags: ImportTrackFlags,
}

impl Default for ImportTrackConfig {
    fn default() -> Self {
        Self {
            faceted_tag_mapping: Default::default(),
            flags: ImportTrackFlags::all()
                .difference(ImportTrackFlags::COMPATIBILITY_ID3V2_APPLE_GRP1),
        }
    }
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum ImportTrack {
    NewTrack { collected_at: OffsetDateTimeMs },
    UpdateTrack(Track),
}

impl ImportTrack {
    #[must_use]
    pub fn with_content(self, content_link: ContentLink, content_type: Mime) -> Track {
        match self {
            ImportTrack::NewTrack { collected_at } => {
                let content = Content {
                    link: content_link,
                    r#type: content_type,
                    metadata: ContentMetadata::Audio(Default::default()),
                    metadata_flags: Default::default(),
                    digest: None,
                };
                let media_source = Source {
                    collected_at,
                    content,
                    artwork: Default::default(),
                };
                Track::new_from_media_source(media_source)
            }
            ImportTrack::UpdateTrack(mut track) => {
                // Neither the content path nor the content type are supposed to change here!?
                debug_assert_eq!(track.media_source.content.link.path, content_link.path);
                debug_assert_eq!(track.media_source.content.r#type, content_type);
                track.media_source.content.link = content_link;
                track.media_source.content.r#type = content_type;
                track
            }
        }
    }
}

pub trait Reader: Read + Seek + 'static {}

impl<T> Reader for T where T: Read + Seek + 'static {}

/// Recoverable errors and warnings
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Issues {
    messages: Vec<String>,
}

impl Issues {
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        let Self { messages } = self;
        messages.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        let Self { messages } = self;
        messages.len()
    }

    pub fn add_message(&mut self, message: impl Into<String>) {
        let message = message.into();
        debug_assert!(!message.trim().is_empty());
        self.messages.push(message);
    }

    #[must_use]
    pub fn into_messages(self) -> Vec<String> {
        let Self { messages } = self;
        messages
    }
}

pub fn import_into_track(
    reader: &mut Box<dyn Reader>,
    config: &ImportTrackConfig,
    track: &mut Track,
) -> Result<Issues> {
    let probe = Probe::new(reader)
        // Workaround for <https://github.com/Serial-ATA/lofty-rs/issues/260>
        .options(ParseOptions::new().max_junk_bytes(usize::MAX))
        .guess_file_type()?;
    let Some(file_type) = probe.file_type() else {
        log::debug!(
            "Skipping import of track {media_source_content_link:?}: {config:?}",
            media_source_content_link = track.media_source.content.link
        );
        return Err(Error::UnsupportedContentType(
            track.media_source.content.r#type.clone(),
        ));
    };
    let mut importer = Importer::new();
    match file_type {
        FileType::Aiff => {
            let reader = probe.into_inner();
            let aiff_file = AudioFile::read_from(reader, parse_options())?;
            crate::fmt::aiff::import_file_into_track(&mut importer, config, aiff_file, track);
        }
        FileType::Flac => {
            let reader = probe.into_inner();
            let flac_file = AudioFile::read_from(reader, parse_options())?;
            crate::fmt::flac::import_file_into_track(&mut importer, config, flac_file, track);
        }
        FileType::Mp4 => {
            let reader = probe.into_inner();
            let mp4_file = AudioFile::read_from(reader, parse_options())?;
            crate::fmt::mp4::import_file_into_track(&mut importer, config, mp4_file, track);
        }
        FileType::Mpeg => {
            let reader = probe.into_inner();
            let mpeg_file = AudioFile::read_from(reader, parse_options())?;
            crate::fmt::mpeg::import_file_into_track(&mut importer, config, mpeg_file, track);
        }
        FileType::Opus => {
            let reader = probe.into_inner();
            let opus_file = AudioFile::read_from(reader, parse_options())?;
            crate::fmt::opus::import_file_into_track(&mut importer, config, opus_file, track);
        }
        FileType::Vorbis => {
            let reader = probe.into_inner();
            let vorbis_file = AudioFile::read_from(reader, parse_options())?;
            crate::fmt::ogg::import_file_into_track(&mut importer, config, vorbis_file, track);
        }
        _ => {
            // Generic fallback
            let tagged_file = probe.read()?;
            crate::fmt::import_tagged_file_into_track(&mut importer, config, tagged_file, track);
        }
    }
    Ok(importer.finish())
}

#[derive(Debug, Clone)]
pub struct LoadedArtworkImageData {
    /// The APIC type of an embedded image
    ///
    /// `Some` for embedded images and `None` for custom, external images.
    pub apic_type: Option<ApicType>,

    /// The MIME type of `image_data`
    pub media_type: Mime,

    /// The actual image data
    pub image_data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct LoadedArtworkImage {
    /// The APIC type of an embedded image
    ///
    /// `Some` for embedded images and `None` for custom, external images.
    pub apic_type: Option<ApicType>,

    /// The MIME type of `image_data`
    pub media_type: Mime,

    /// The actual image
    pub image: DynamicImage,
}

pub fn load_embedded_artwork_image_data_from_file_path(
    file_path: &Path,
) -> Result<Option<LoadedArtworkImageData>> {
    let tag = {
        let mut tagged_file = lofty::read_from_path(file_path)?;
        crate::fmt::take_primary_or_first_tag(&mut tagged_file)
    };
    if let Some((apic_type, media_type, image_data)) = tag
        .as_ref()
        .and_then(crate::fmt::find_embedded_artwork_image)
    {
        let media_type = media_type.parse::<Mime>()?;
        let loaded_artwork_image = LoadedArtworkImageData {
            apic_type: Some(apic_type),
            media_type,
            image_data: image_data.to_owned(),
        };
        Ok(Some(loaded_artwork_image))
    } else {
        Ok(None)
    }
}

fn verify_artwork_image_metadata(
    artwork_image: &ArtworkImage,
    apic_type: Option<ApicType>,
    media_type: &Mime,
) {
    if let Some(apic_type) = apic_type {
        if apic_type != artwork_image.apic_type {
            log::warn!(
                "Mismatching artwork image APIC types: expected = {expected:?}, actual = \
                 {apic_type:?}",
                expected = artwork_image.apic_type
            );
        }
    }
    if media_type.essence_str() != artwork_image.media_type.essence_str() {
        log::warn!(
            "Mismatching artwork image MIME types: expected = {expected}, actual = {actual}",
            expected = artwork_image.media_type.essence_str(),
            actual = media_type.essence_str(),
        );
    }
}

pub fn load_artwork_image_data(
    file_path: &Path,
    artwork: &Artwork,
) -> Result<Option<LoadedArtworkImageData>> {
    match artwork {
        Artwork::Embedded(EmbeddedArtwork { image }) => {
            let loaded = load_embedded_artwork_image_data_from_file_path(file_path)?;
            let Some(loaded) = loaded else {
                return Ok(None);
            };
            verify_artwork_image_metadata(image, loaded.apic_type, &loaded.media_type);
            Ok(Some(loaded))
        }
        Artwork::Linked(LinkedArtwork { uri, image }) => {
            let url = uri
                .parse::<Url>()
                .map_err(|err| Error::Other(anyhow!("invalid URL: {err}")))?;
            let file_path = url
                .to_file_path()
                .map_err(|()| Error::Other(anyhow!("no local file URL")))?;
            let image_data = std::fs::read(file_path)?;
            let loaded = LoadedArtworkImageData {
                apic_type: Some(image.apic_type),
                media_type: image.media_type.clone(),
                image_data,
            };
            Ok(Some(loaded))
        }
        Artwork::Missing | Artwork::Unsupported | Artwork::Irregular => Ok(None),
    }
}

pub fn load_artwork_image(file_path: &Path, artwork: &Artwork) -> Result<Option<DynamicImage>> {
    match artwork {
        Artwork::Embedded(EmbeddedArtwork { image }) => {
            let loaded = load_embedded_artwork_image_data_from_file_path(file_path)?;
            let Some(loaded) = loaded else {
                return Ok(None);
            };
            verify_artwork_image_metadata(image, loaded.apic_type, &loaded.media_type);
            let reader = ImageReader::new(Cursor::new(loaded.image_data.as_slice()));
            reader.decode().map(Some).map_err(Into::into)
        }
        Artwork::Linked(LinkedArtwork { uri, image: _ }) => {
            let url = uri
                .parse::<Url>()
                .map_err(|err| Error::Other(anyhow!("invalid URL: {err}")))?;
            let file_path = url
                .to_file_path()
                .map_err(|()| Error::Other(anyhow!("no local file URL")))?;
            let reader = ImageReader::open(file_path)?;
            reader.decode().map(Some).map_err(Into::into)
        }
        Artwork::Missing | Artwork::Unsupported | Artwork::Irregular => Ok(None),
    }
}

pub fn try_import_plain_tag<'a>(
    label: impl Into<Option<TagLabel<'a>>>,
    score_value: impl Into<ScoreValue>,
) -> StdResult<PlainTag<'a>, PlainTag<'a>> {
    let label = label.into();
    let score = TagScore::clamp_from(score_value);
    let plain_tag = PlainTag { label, score };
    if plain_tag.is_valid() {
        Ok(plain_tag)
    } else {
        Err(plain_tag)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TrackScope {
    Track,
    Album,
}

impl TrackScope {
    const fn message_str(self) -> &'static str {
        match self {
            Self::Track => "track",
            Self::Album => "album",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ImportedTempoBpm {
    /// Field contained a decimal point
    Fractional(TempoBpm),
    /// Field didn't contain a decimal point and is an integer value
    Integer(TempoBpm),
}

impl ImportedTempoBpm {
    #[must_use]
    pub(crate) const fn is_integer(&self) -> bool {
        matches!(self, Self::Integer(_))
    }
}

impl From<ImportedTempoBpm> for TempoBpm {
    fn from(from: ImportedTempoBpm) -> Self {
        match from {
            ImportedTempoBpm::Fractional(into) | ImportedTempoBpm::Integer(into) => into,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Importer {
    issues: Issues,
}

impl Importer {
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self {
            issues: Issues::new(),
        }
    }

    pub(crate) fn add_issue(&mut self, message: impl Into<String>) {
        self.issues.add_message(message);
    }

    #[must_use]
    pub(crate) fn finish(self) -> Issues {
        let Self { issues } = self;
        issues
    }

    #[must_use]
    pub(crate) fn import_year_tag_from_field(
        &mut self,
        field: &str,
        input: &str,
    ) -> Option<DateOrDateTime> {
        let parsed = parse_year_tag(input);
        if parsed.is_none() {
            self.issues.messages.push(format!(
                "Failed to parse year tag from input '{input}' in field '{field}'"
            ));
        }
        parsed
    }

    #[must_use]
    pub(crate) fn finish_import_of_titles(
        &mut self,
        scope: TrackScope,
        titles: Vec<Title>,
    ) -> Canonical<Vec<Title>> {
        let titles_len = titles.len();
        let titles = titles.canonicalize_into();
        if titles.len() < titles_len {
            self.issues.add_message(format!(
                "Discarded {num_duplicates} duplicate {scope} titles",
                num_duplicates = titles_len - titles.len(),
                scope = scope.message_str(),
            ));
        }
        titles
    }

    #[must_use]
    #[expect(unstable_name_collisions, reason = "intersperse")]
    pub(crate) fn finish_import_of_actors(
        &mut self,
        scope: TrackScope,
        actors: Vec<Actor>,
    ) -> Canonical<Vec<Actor>> {
        let actors_len = actors.len();
        let mut actors = actors.canonicalize_into();
        if actors.len() < actors_len {
            self.issues.add_message(format!(
                "Discarded {duplicate_count} duplicate {scope} actors",
                duplicate_count = actors_len - actors.len(),
                scope = scope.message_str(),
            ));
        }
        let mut roles = actors
            .iter()
            .map(|actor| actor.role)
            .dedup()
            .collect::<Vec<_>>();
        roles.sort_unstable();
        roles.dedup();
        for role in roles {
            if Actors::summary_actor(actors.iter(), role).is_some() {
                // Summary actor for this role is present.
                continue;
            }
            // If no summary actor is present then create one by concatenating
            // the names of all individual actors.
            let summary_name =
                Actors::filter_kind_role(actors.iter(), actor::Kind::Individual, role)
                    .map(|actor| actor.name.as_str())
                    .intersperse(", ")
                    .collect::<String>();
            if summary_name.is_empty() {
                continue;
            }
            let summary_actor = Actor {
                role,
                kind: actor::Kind::Summary,
                name: summary_name,
                ..Default::default()
            };
            let mut actors_edit = std::mem::take(&mut actors).untie();
            actors_edit.push(summary_actor);
            actors = actors_edit.canonicalize_into();
        }
        actors
    }

    #[must_use]
    pub(crate) fn import_tempo_bpm(&mut self, input: &str) -> Option<ImportedTempoBpm> {
        let input = trim_readable(input);
        if input.is_empty() {
            return None;
        }
        match input.parse() {
            Ok(bpm) => {
                let tempo_bpm = TempoBpm::new(bpm);
                if !tempo_bpm.is_valid() {
                    // The value 0 is often used for an unknown bpm.
                    // Silently ignore this special value to prevent log spam.
                    if bpm != 0.0 {
                        self.add_issue(format!(
                            "Invalid tempo parsed from input '{input}': {tempo_bpm}"
                        ));
                    }
                    return None;
                }
                log::debug!("Parsed tempo from input '{input}': {tempo_bpm}");
                let imported = if input.contains('.') {
                    ImportedTempoBpm::Fractional(tempo_bpm)
                } else {
                    ImportedTempoBpm::Integer(tempo_bpm)
                };
                Some(imported)
            }
            Err(err) => {
                self.add_issue(format!(
                    "Failed to parse tempo (BPM) from input '{input}': {err}"
                ));
                None
            }
        }
    }

    #[must_use]
    pub(crate) fn import_loudness_from_replay_gain(&mut self, input: &str) -> Option<LoudnessLufs> {
        let input = trim_readable(input);
        if input.is_empty() {
            return None;
        }
        match parse_replay_gain_db(input) {
            Ok((remainder, relative_gain_db)) => {
                if !remainder.is_empty() {
                    self.add_issue(format!(
                        "Unexpected remainder '{remainder}' after parsing replay gain input \
                         '{input}'"
                    ));
                }
                let loudness_lufs = db2lufs(relative_gain_db);
                if !loudness_lufs.is_valid() {
                    self.add_issue(format!(
                        "Invalid loudness parsed from replay gain input '{input}': {loudness_lufs}"
                    ));
                    return None;
                }
                log::debug!("Parsed loudness from replay gain input '{input}': {loudness_lufs}");
                Some(loudness_lufs)
            }
            Err(err) => {
                // Silently ignore any 0 values
                if input.parse().ok() == Some(0.0) {
                    log::debug!("Ignoring invalid replay gain (dB) from input '{input}': {err}");
                } else {
                    self.add_issue(format!(
                        "Failed to parse replay gain (dB) from input '{input}': {err}"
                    ));
                }
                None
            }
        }
    }

    pub(crate) fn import_key_signature(&mut self, input: &str) -> Option<KeySignature> {
        let key_signature = parse_key_signature(input);
        if key_signature.is_none() {
            let input_bytes = input.as_bytes();
            self.add_issue(format!(
                "Failed to parse musical key signature from input '{input}' (UTF-8 bytes: \
                 {input_bytes:X?})",
            ));
        }
        key_signature
    }

    pub(crate) fn import_faceted_tags_from_label_values<'a>(
        &mut self,
        tags_map: &mut TagsMap<'a>,
        faceted_tag_mapping_config: &FacetedTagMappingConfig,
        facet_key: &TagFacetKey,
        label_values: impl IntoIterator<Item = Cow<'a, str>>,
    ) -> usize {
        let tag_mapping_config = faceted_tag_mapping_config.get(facet_key.as_str());
        let mut total_import_count = 0;
        let mut plain_tags = Vec::with_capacity(8);
        let mut next_score_value = PlainTag::DEFAULT_SCORE.value();
        for label_value in label_values {
            total_import_count += self.import_plain_tags_from_joined_label_value(
                tag_mapping_config,
                &mut next_score_value,
                &mut plain_tags,
                label_value,
            );
        }
        let count = plain_tags.len();
        if count < total_import_count {
            self.issues.add_message(format!(
                "Discarded {duplicate_count} duplicate tag labels for facet \"{facet_key}\"",
                duplicate_count = total_import_count - count,
            ));
        }
        tags_map.update_tags_by_label_ordering(facet_key, plain_tags);
        count
    }

    pub(crate) fn import_plain_tags_from_joined_label_value<'a>(
        &mut self,
        tag_mapping_config: Option<&TagMappingConfig>,
        next_score_value: &mut ScoreValue,
        plain_tags: &mut Vec<PlainTag<'a>>,
        joined_label_value: impl Into<Cow<'a, str>>,
    ) -> usize {
        if let Some(joined_label) = TagLabel::clamp_from(joined_label_value) {
            debug_assert!(!joined_label.is_empty());
            let mut import_count = 0;
            if let Some(tag_mapping_config) = tag_mapping_config {
                if !tag_mapping_config.label_separator.is_empty() {
                    for split in joined_label
                        .as_str()
                        .split(&tag_mapping_config.label_separator)
                    {
                        let label = TagLabel::clamp_from(split).map(TagLabel::into_owned);
                        match try_import_plain_tag(label, *next_score_value) {
                            Ok(plain_tag) => {
                                plain_tags.push(plain_tag);
                                import_count += 1;
                                *next_score_value =
                                    tag_mapping_config.next_score_value(*next_score_value);
                            }
                            Err(plain_tag) => {
                                self.add_issue(format!(
                                    "Failed to import plain tag: {plain_tag:?}"
                                ));
                            }
                        }
                    }
                }
            }
            if import_count == 0 {
                // Try to import the whole string as a single tag label
                match try_import_plain_tag(joined_label, *next_score_value) {
                    Ok(plain_tag) => {
                        plain_tags.push(plain_tag);
                        import_count += 1;
                        if let Some(tag_mapping_config) = tag_mapping_config {
                            *next_score_value =
                                tag_mapping_config.next_score_value(*next_score_value);
                        }
                    }
                    Err(plain_tag) => {
                        self.add_issue(format!("Failed to import plain tag: {plain_tag:?}"));
                    }
                }
            }
            import_count
        } else {
            log::debug!("Skipping empty tag label");
            0
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
