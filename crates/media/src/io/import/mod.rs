// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    borrow::Cow,
    io::{Read, Seek},
    path::Path,
    result::Result as StdResult,
};

use bitflags::bitflags;
use lofty::FileType;
use mime::Mime;
use semval::IsValid as _;

use aoide_core::{
    audio::signal::LoudnessLufs,
    media::{
        artwork::ApicType,
        content::{ContentMetadata, ContentPath, ContentRevision},
        Content, Source,
    },
    music::{key::KeySignature, tempo::TempoBpm},
    tag::{
        FacetId as TagFacetId, Label as TagLabel, PlainTag, Score as TagScore, ScoreValue, TagsMap,
    },
    track::{actor::Actor, title::Title, Track},
    util::{
        canonical::{Canonical, CanonicalizeInto as _},
        clock::{DateOrDateTime, DateTime},
    },
};

use crate::{
    util::{
        db2lufs,
        digest::MediaDigest,
        parse_key_signature, parse_replay_gain_db, parse_year_tag,
        tag::{FacetedTagMappingConfig, TagMappingConfig},
        trim_readable,
    },
    Error, Result,
};

#[rustfmt::skip]
bitflags! {
    /// Flags for controlling the import
    ///
    /// It is recommended to enable all for maximum information and
    /// maximum compatibility.
    pub struct ImportTrackFlags: u16 {
        /// Import metadata
        ///
        /// Import metadata from file tags like ID3 frames, MPEG4 atoms,
        /// or Vorbis Comments.
        const METADATA                                          = 0b0000_0000_0000_0001;

        /// Import embedded artwork
        ///
        /// Imports a single cover image embedded in the metadata.
        ///
        /// Implies METADATA.
        const METADATA_EMBEDDED_ARTWORK                         = 0b0000_0000_0000_0011;

        /// Hash cover image
        ///
        /// Implies METADATA_EMBEDDED_ARTWORK.
        const METADATA_EMBEDDED_ARTWORK_DIGEST                  = 0b0000_0000_0000_0111;

        /// Use iTunes grouping/movement/work mapping
        ///
        /// Use the mapping for grouping and movement/work fields as introduced
        /// by iTunes v12.5.4. This is the preferred mapping and existing files
        /// that still use TIT1 instead of GRP1 for storing the grouping property
        /// should be updated accordingly.
        ///
        /// Implies METADATA.
        const COMPATIBILITY_ID3V2_ITUNES_GROUPING_MOVEMENT_WORK = 0b0000_0001_0000_0001;

        #[cfg(feature = "gigtag")]
        /// Import gigtags from Content Group/Grouping file tag
        ///
        /// Implies METADATA.
        const GIGTAGS                                           = 0b0001_0000_0000_0001;

        #[cfg(feature = "serato-markers")]
        /// Import metadata (cue points, loops, track color) from Serato file tags
        ///
        /// Implies METADATA.
        const SERATO_MARKERS                                    = 0b0010_0000_0000_0001;
    }
}

impl ImportTrackFlags {
    #[must_use]
    pub fn is_valid(self) -> bool {
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

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ImportTrackConfig {
    pub faceted_tag_mapping: FacetedTagMappingConfig,
    pub flags: ImportTrackFlags,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewTrackInput {
    pub collected_at: DateTime,
    pub content_rev: Option<ContentRevision>,
}

impl NewTrackInput {
    #[must_use]
    pub fn into_new_track(self, path: ContentPath, content_type: Mime) -> Track {
        let Self {
            collected_at,
            content_rev,
        } = self;
        let content = Content {
            link: aoide_core::media::content::ContentLink {
                path,
                rev: content_rev,
            },
            r#type: content_type,
            metadata: ContentMetadata::Audio(Default::default()),
            metadata_flags: Default::default(),
            digest: None,
        };
        let media_source = Source {
            collected_at,
            content,
            artwork: Default::default(),
            advisory_rating: None,
        };
        Track::new_from_media_source(media_source)
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
    let probe = lofty::Probe::new(reader).guess_file_type()?;
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
        FileType::AIFF => {
            let reader = probe.into_inner();
            let aiff_file = lofty::AudioFile::read_from(reader, Default::default())
                .map_err(anyhow::Error::from)?;
            crate::fmt::lofty::aiff::import_file_into_track(
                &mut importer,
                config,
                aiff_file,
                track,
            );
        }
        FileType::FLAC => {
            let reader = probe.into_inner();
            let flac_file = lofty::AudioFile::read_from(reader, Default::default())
                .map_err(anyhow::Error::from)?;
            crate::fmt::lofty::flac::import_file_into_track(
                &mut importer,
                config,
                flac_file,
                track,
            );
        }
        FileType::MP4 => {
            let reader = probe.into_inner();
            let mp4_file = lofty::AudioFile::read_from(reader, Default::default())
                .map_err(anyhow::Error::from)?;
            crate::fmt::lofty::mp4::import_file_into_track(&mut importer, config, mp4_file, track);
        }
        FileType::MPEG => {
            let reader = probe.into_inner();
            let mpeg_file = lofty::AudioFile::read_from(reader, Default::default())
                .map_err(anyhow::Error::from)?;
            crate::fmt::lofty::mpeg::import_file_into_track(
                &mut importer,
                config,
                mpeg_file,
                track,
            );
        }
        FileType::Opus => {
            let reader = probe.into_inner();
            let opus_file = lofty::AudioFile::read_from(reader, Default::default())
                .map_err(anyhow::Error::from)?;
            crate::fmt::lofty::opus::import_file_into_track(
                &mut importer,
                config,
                opus_file,
                track,
            );
        }
        FileType::Vorbis => {
            let reader = probe.into_inner();
            let vorbis_file = lofty::AudioFile::read_from(reader, Default::default())
                .map_err(anyhow::Error::from)?;
            crate::fmt::lofty::ogg::import_file_into_track(
                &mut importer,
                config,
                vorbis_file,
                track,
            );
        }
        _ => {
            // Generic fallback
            let tagged_file = probe.read().map_err(anyhow::Error::from)?;
            crate::fmt::lofty::import_tagged_file_into_track(
                &mut importer,
                config,
                tagged_file,
                track,
            );
        }
    }
    Ok(importer.finish())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedArtworkImage {
    /// The APIC type of an embedded image
    ///
    /// `Some` for embedded images and `None` for custom, external images.
    pub apic_type: Option<ApicType>,

    /// The MIME type of `image_data`
    pub media_type: Mime,

    /// The actual image data
    pub image_data: Vec<u8>,
}

#[allow(unused)]
fn parse_media_type(media_type: &str) -> Result<Mime> {
    media_type
        .parse()
        .map_err(anyhow::Error::from)
        .map_err(Into::into)
}

pub fn load_embedded_artwork_image_from_file_path(
    file_path: &Path,
) -> Result<Option<LoadedArtworkImage>> {
    let tag = {
        let mut tagged_file = lofty::read_from_path(file_path).map_err(anyhow::Error::from)?;
        crate::fmt::lofty::take_primary_or_first_tag(&mut tagged_file)
    };
    if let Some((apic_type, media_type, image_data)) = tag
        .as_ref()
        .and_then(crate::fmt::lofty::find_embedded_artwork_image)
    {
        let media_type = parse_media_type(media_type)?;
        let loaded_artwork_image = LoadedArtworkImage {
            apic_type: Some(apic_type),
            media_type,
            image_data: image_data.to_owned(),
        };
        Ok(Some(loaded_artwork_image))
    } else {
        Ok(None)
    }
}

pub fn try_import_plain_tag<'a>(
    label: impl Into<Option<TagLabel<'a>>>,
    score_value: impl Into<ScoreValue>,
) -> StdResult<PlainTag, PlainTag> {
    let label = label.into().map(TagLabel::into_owned);
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
    fn message_str(self) -> &'static str {
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
    NonFractional(TempoBpm),
}

impl ImportedTempoBpm {
    #[must_use]
    pub(crate) const fn is_non_fractional(&self) -> bool {
        matches!(self, Self::NonFractional(_))
    }
}

impl From<ImportedTempoBpm> for TempoBpm {
    fn from(from: ImportedTempoBpm) -> Self {
        match from {
            ImportedTempoBpm::Fractional(into) | ImportedTempoBpm::NonFractional(into) => into,
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
                "Discarded {} duplicate {} titles",
                titles_len - titles.len(),
                scope.message_str(),
            ));
        }
        Canonical::tie(titles)
    }

    #[must_use]
    pub(crate) fn finish_import_of_actors(
        &mut self,
        scope: TrackScope,
        actors: Vec<Actor>,
    ) -> Canonical<Vec<Actor>> {
        let actors_len = actors.len();
        let actors = actors.canonicalize_into();
        if actors.len() < actors_len {
            self.issues.add_message(format!(
                "Discarded {} duplicate {} actors",
                actors_len - actors.len(),
                scope.message_str(),
            ));
        }
        Canonical::tie(actors)
    }

    #[must_use]
    pub(crate) fn import_tempo_bpm(&mut self, input: &str) -> Option<ImportedTempoBpm> {
        let input = trim_readable(input);
        if input.is_empty() {
            return None;
        }
        match input.parse() {
            Ok(bpm) => {
                let tempo_bpm = TempoBpm::from_inner(bpm);
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
                    ImportedTempoBpm::NonFractional(tempo_bpm)
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
                        "Unexpected remainder '{remainder}' after parsing replay gain input '{input}'"
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
                "Failed to parse musical key signature from input '{input}' (UTF-8 bytes: {input_bytes:X?})",
            ));
        }
        key_signature
    }

    pub(crate) fn import_faceted_tags_from_label_values<'a>(
        &mut self,
        tags_map: &mut TagsMap<'_>,
        faceted_tag_mapping_config: &FacetedTagMappingConfig,
        facet_id: &TagFacetId<'_>,
        label_values: impl IntoIterator<Item = Cow<'a, str>>,
    ) -> usize {
        let tag_mapping_config = faceted_tag_mapping_config.get(facet_id.as_str());
        let mut total_import_count = 0;
        let mut plain_tags = Vec::with_capacity(8);
        let mut next_score_value = TagScore::default_value();
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
                "Discarded {} duplicate tag labels for facet '{facet_id}'",
                total_import_count - count,
            ));
        }
        // TODO: Avoid cloning `facet_id`.
        let facet_id: TagFacetId<'static> = facet_id.clone().into_owned();
        tags_map.update_faceted_plain_tags_by_label_ordering(&facet_id, plain_tags);
        count
    }

    pub(crate) fn import_plain_tags_from_joined_label_value<'a>(
        &mut self,
        tag_mapping_config: Option<&TagMappingConfig>,
        next_score_value: &mut ScoreValue,
        plain_tags: &mut Vec<PlainTag>,
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
                        let label = TagLabel::clamp_from(split);
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
