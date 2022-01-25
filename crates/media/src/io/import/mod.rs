// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::{borrow::Cow, result::Result as StdResult};

use semval::IsValid as _;

use aoide_core::{
    audio::signal::LoudnessLufs,
    media::{ApicType, Content, Source, SourcePath},
    music::{key::KeySignature, tempo::TempoBpm},
    tag::{
        CowLabel, FacetId as TagFacetId, Label as TagLabel, LabelValue, PlainTag,
        Score as TagScore, ScoreValue, TagsMap,
    },
    track::{actor::Actor, index::Index, title::Title, Track},
    util::{
        canonical::{Canonical, CanonicalizeInto as _},
        clock::{DateOrDateTime, DateTime},
    },
};

use crate::{
    util::{
        db2lufs,
        digest::MediaDigest,
        guess_mime_from_path, media_type_from_image_format, parse_index_numbers,
        parse_key_signature, parse_replay_gain_db, parse_year_tag,
        tag::{FacetedTagMappingConfig, TagMappingConfig},
        trim_readable,
    },
    Error, Result,
};

use bitflags::bitflags;
use mime::Mime;
use std::{
    fs::File,
    io::{BufReader, Read, Seek},
    path::Path,
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
        /// Use the mapping for grouping and movement/WORK fields as introduced
        /// by iTunes v12.5.4.
        const COMPATIBILITY_ID3V2_ITUNES_GROUPING_MOVEMENT_WORK = 0b0000_0001_0000_0000;

        /// Use the `TDRC` and `TDOR` frames for the release and recording date
        /// respectively.
        ///
        /// iTunes and most other applications don't strictly follow the ID3v2.4
        /// standard that provides `TDRL` (release date), `TDRC` (recording date),
        /// `TDOR` (original release date), and more date frames for various purposes.
        /// Instead they simply (mis)use `TDRC` for the release date and `TDOR` for
        /// both the original release and recording date without further distinction.
        ///
        /// See also https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html
        const COMPATIBILITY_ID3V2_MUSICBRAINZ_PICARD_TDRC_TDOR  = 0b0000_0010_0000_0000;

        /// Import aoide faceted tags
        ///
        /// Implies METADATA.
        const CUSTOM_AOIDE_TAGS                                 = 0b0001_0000_0000_0001;

        /// Import metadata (cue points, loops, track color) from Serato file tags
        ///
        /// Implies METADATA.
        const CUSTOM_SERATO_MARKERS                             = 0b0010_0000_0000_0001;
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewTrackInput {
    pub collected_at: DateTime,
    pub synchronized_at: DateTime,
}

impl NewTrackInput {
    #[must_use]
    pub fn into_new_track(self, path: SourcePath, content_type: Mime) -> Track {
        let Self {
            collected_at,
            synchronized_at,
        } = self;
        let media_source = Source {
            collected_at,
            synchronized_at: Some(synchronized_at),
            path,
            content_type,
            advisory_rating: None,
            content_digest: None,
            content_metadata_flags: Default::default(),
            content: Content::Audio(Default::default()),
            artwork: Default::default(),
        };
        Track::new_from_media_source(media_source)
    }
}

pub trait Reader: Read + Seek + 'static {}

impl<T> Reader for T where T: Read + Seek + 'static {}

/// Recoverable errors and warnings
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Issues {
    messages: Vec<String>,
}

impl Issues {
    #[must_use]
    pub const fn new() -> Self {
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
    let mut importer = Importer::new();
    match track.media_source.content_type.essence_str() {
        #[cfg(feature = "fmt-flac")]
        "audio/flac" => crate::fmt::flac::Metadata::read_from(reader)
            .and_then(|metadata| metadata.import_into_track(&mut importer, config, track)),
        #[cfg(feature = "fmt-mp3")]
        "audio/mpeg" => crate::fmt::mp3::MetadataExt::read_from(reader)
            .and_then(|metadata_ext| metadata_ext.import_into_track(&mut importer, config, track)),
        #[cfg(feature = "fmt-mp4")]
        "audio/m4a" | "video/mp4" => crate::fmt::mp4::Metadata::read_from(reader)
            .and_then(|metadata| metadata.import_into_track(&mut importer, config, track)),
        #[cfg(feature = "fmt-ogg")]
        "audio/ogg" | "audio/vorbis" => crate::fmt::ogg::Metadata::read_from(reader)
            .and_then(|metadata| metadata.import_into_track(&mut importer, config, track)),
        #[cfg(feature = "fmt-opus")]
        "audio/opus" => crate::fmt::opus::Metadata::read_from(reader)
            .and_then(|metadata| metadata.import_into_track(&mut importer, config, track)),
        _ => Err(Error::UnsupportedContentType(
            track.media_source.content_type.to_owned(),
        )),
    }
    .map(move |()| importer.finish())
    .map_err(|err| {
        log::warn!(
            "Failed to parse metadata from media source '{}': {}",
            track.media_source.path,
            err
        );
        err
    })
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

fn parse_media_type(media_type: &str) -> Result<Mime> {
    media_type
        .parse()
        .map_err(anyhow::Error::from)
        .map_err(Into::into)
}

pub fn load_embedded_artwork_image_from_file_path(
    importer: &mut Importer,
    file_path: &Path,
) -> Result<Option<LoadedArtworkImage>> {
    let file = File::open(file_path)?;
    let mut reader: Box<dyn Reader> = Box::new(BufReader::new(file));
    let mime = guess_mime_from_path(&file_path)?;
    match mime.as_ref() {
        #[cfg(feature = "fmt-flac")]
        "audio/flac" => crate::fmt::flac::Metadata::read_from(&mut reader).and_then(|metadata| {
            metadata
                .find_embedded_artwork_image()
                .map(|(apic_type, media_type, image_data)| {
                    Ok(LoadedArtworkImage {
                        apic_type: Some(apic_type),
                        media_type: parse_media_type(media_type)?,
                        image_data: image_data.to_owned(),
                    })
                })
                .transpose()
        }),
        #[cfg(feature = "fmt-mp3")]
        "audio/mpeg" => crate::fmt::mp3::Metadata::read_from(&mut reader).and_then(|metadata| {
            metadata
                .as_ref()
                .and_then(crate::fmt::mp3::Metadata::find_embedded_artwork_image)
                .map(|(apic_type, media_type, image_data)| {
                    Ok(LoadedArtworkImage {
                        apic_type: Some(apic_type),
                        media_type: parse_media_type(media_type)?,
                        image_data: image_data.to_owned(),
                    })
                })
                .transpose()
        }),
        #[cfg(feature = "fmt-mp4")]
        "audio/m4a" | "video/mp4" => {
            crate::fmt::mp4::Metadata::read_from(&mut reader).and_then(|metadata| {
                metadata
                    .find_embedded_artwork_image()
                    .map(|(apic_type, image_format, image_data)| {
                        Ok(LoadedArtworkImage {
                            apic_type: Some(apic_type),
                            media_type: media_type_from_image_format(image_format)?,
                            image_data: image_data.to_owned(),
                        })
                    })
                    .transpose()
            })
        }
        #[cfg(feature = "fmt-ogg")]
        "audio/ogg" | "audio/vorbis" => {
            crate::fmt::ogg::Metadata::read_from(&mut reader).and_then(|metadata| {
                metadata
                    .find_embedded_artwork_image(importer)
                    .map(|(apic_type, media_type, image_data)| {
                        Ok(LoadedArtworkImage {
                            apic_type: Some(apic_type),
                            media_type: parse_media_type(&media_type)?,
                            image_data,
                        })
                    })
                    .transpose()
            })
        }
        #[cfg(feature = "fmt-opus")]
        "audio/opus" => crate::fmt::opus::Metadata::read_from(&mut reader).and_then(|metadata| {
            metadata
                .find_embedded_artwork_image(importer)
                .map(|(apic_type, media_type, image_data)| {
                    Ok(LoadedArtworkImage {
                        apic_type: Some(apic_type),
                        media_type: parse_media_type(&media_type)?,
                        image_data,
                    })
                })
                .transpose()
        }),
        _ => Err(Error::UnsupportedContentType(mime)),
    }
}

pub fn try_import_plain_tag<'a>(
    label: impl Into<Option<CowLabel<'a>>>,
    score_value: impl Into<ScoreValue>,
) -> StdResult<PlainTag, PlainTag> {
    let label = label.into().map(Into::into);
    let score = TagScore::clamp_from(score_value);
    let plain_tag = PlainTag { label, score };
    if plain_tag.is_valid() {
        Ok(plain_tag)
    } else {
        Err(plain_tag)
    }
}

#[derive(Debug, Default)]
pub struct Importer {
    issues: Issues,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackScope {
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

impl Importer {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            issues: Issues::new(),
        }
    }

    pub fn add_issue(&mut self, message: impl Into<String>) {
        self.issues.add_message(message)
    }

    #[must_use]
    pub fn finish(self) -> Issues {
        let Self { issues } = self;
        issues
    }

    #[must_use]
    pub fn import_year_tag_from_field(
        &mut self,
        field: &str,
        input: &str,
    ) -> Option<DateOrDateTime> {
        let parsed = parse_year_tag(input);
        if parsed.is_none() {
            self.issues.messages.push(format!(
                "Failed to parse year tag from input '{}' in field '{}'",
                input, field
            ));
        }
        parsed
    }

    #[must_use]
    pub fn finish_import_of_titles(
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
    pub fn finish_import_of_actors(
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
    pub fn import_tempo_bpm(&mut self, input: &str) -> Option<TempoBpm> {
        let input = trim_readable(input);
        if input.is_empty() {
            return None;
        }
        match input.parse() {
            Ok(bpm) => {
                let tempo_bpm = TempoBpm::from_raw(bpm);
                if !tempo_bpm.is_valid() {
                    // The value 0 is often used for an unknown bpm.
                    // Silently ignore this special value to prevent log spam.
                    if bpm != 0.0 {
                        self.add_issue(format!(
                            "Invalid tempo parsed from input '{}': {}",
                            input, tempo_bpm
                        ));
                    }
                    return None;
                }
                log::debug!("Parsed tempo from input '{}': {}", input, tempo_bpm);
                Some(tempo_bpm)
            }
            Err(err) => {
                self.add_issue(format!(
                    "Failed to parse tempo (BPM) from input '{}': {}",
                    input, err
                ));
                None
            }
        }
    }

    #[must_use]
    pub fn import_replay_gain(&mut self, input: &str) -> Option<LoudnessLufs> {
        let input = trim_readable(input);
        if input.is_empty() {
            return None;
        }
        match parse_replay_gain_db(input) {
            Ok((remainder, relative_gain_db)) => {
                if !remainder.is_empty() {
                    self.add_issue(format!(
                        "Unexpected remainder '{}' after parsing replay gain input '{}'",
                        remainder, input
                    ));
                }
                let loudness_lufs = db2lufs(relative_gain_db);
                if !loudness_lufs.is_valid() {
                    self.add_issue(format!(
                        "Invalid loudness parsed from replay gain input '{}': {}",
                        input, loudness_lufs
                    ));
                    return None;
                }
                log::debug!(
                    "Parsed loudness from replay gain input '{}': {}",
                    input,
                    loudness_lufs
                );
                Some(loudness_lufs)
            }
            Err(err) => {
                // Silently ignore any 0 values
                if input.parse().ok() == Some(0.0) {
                    log::debug!(
                        "Ignoring invalid replay gain (dB) from input '{}': {}",
                        input,
                        err
                    );
                } else {
                    self.add_issue(format!(
                        "Failed to parse replay gain (dB) from input '{}': {}",
                        input, err
                    ));
                }
                None
            }
        }
    }

    pub fn import_key_signature(&mut self, input: &str) -> Option<KeySignature> {
        let key_signature = parse_key_signature(input);
        if key_signature.is_none() {
            self.add_issue(format!(
                "Failed to parse musical key signature from input '{}' (UTF-8 bytes: {:X?})",
                input,
                input.as_bytes()
            ));
        }
        key_signature
    }

    #[must_use]
    pub fn import_index_numbers_from_field(&mut self, field: &str, input: &str) -> Option<Index> {
        let index = parse_index_numbers(input);
        if index.is_none() {
            self.add_issue(format!(
                "Failed to parse index numbers from input '{}' in field '{}'",
                input, field,
            ));
        }
        index
    }

    pub fn import_faceted_tags_from_label_values(
        &mut self,
        tags_map: &mut TagsMap,
        faceted_tag_mapping_config: &FacetedTagMappingConfig,
        facet_id: &TagFacetId,
        label_values: impl IntoIterator<Item = LabelValue>,
    ) -> usize {
        let tag_mapping_config = faceted_tag_mapping_config.get(facet_id.value());
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
                "Discarded {} duplicate tag labels for facet '{}'",
                total_import_count - count,
                facet_id,
            ));
        }
        if plain_tags.is_empty() {
            return 0;
        }
        tags_map.update_faceted_plain_tags_by_label_ordering(facet_id, plain_tags);
        count
    }

    pub fn import_plain_tags_from_joined_label_value<'a>(
        &mut self,
        tag_mapping_config: Option<&TagMappingConfig>,
        next_score_value: &mut ScoreValue,
        plain_tags: &mut Vec<PlainTag>,
        joined_label_value: impl Into<Cow<'a, str>>,
    ) -> usize {
        if let Some(joined_label_value) = TagLabel::clamp_value(joined_label_value) {
            debug_assert!(!joined_label_value.is_empty());
            let mut import_count = 0;
            if let Some(tag_mapping_config) = tag_mapping_config {
                if !tag_mapping_config.label_separator.is_empty() {
                    for label_value in joined_label_value.split(&tag_mapping_config.label_separator)
                    {
                        let label = TagLabel::clamp_value(label_value);
                        match try_import_plain_tag(label, *next_score_value) {
                            Ok(plain_tag) => {
                                plain_tags.push(plain_tag);
                                import_count += 1;
                                *next_score_value =
                                    tag_mapping_config.next_score_value(*next_score_value);
                            }
                            Err(plain_tag) => {
                                self.add_issue(format!(
                                    "Failed to import plain tag: {:?}",
                                    plain_tag
                                ));
                            }
                        }
                    }
                }
            }
            if import_count == 0 {
                // Try to import the whole string as a single tag label
                match try_import_plain_tag(joined_label_value, *next_score_value) {
                    Ok(plain_tag) => {
                        plain_tags.push(plain_tag);
                        import_count += 1;
                        if let Some(tag_mapping_config) = tag_mapping_config {
                            *next_score_value =
                                tag_mapping_config.next_score_value(*next_score_value);
                        }
                    }
                    Err(plain_tag) => {
                        self.add_issue(format!("Failed to import plain tag: {:?}", plain_tag));
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