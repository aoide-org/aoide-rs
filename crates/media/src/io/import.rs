// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
    media::{ApicType, Content, Source, SourcePath},
    tag::{
        CowLabel, FacetId as TagFacetId, Label as TagLabel, LabelValue, PlainTag,
        Score as TagScore, ScoreValue, TagsMap,
    },
    track::{actor::Actor, title::Title, Track},
    util::{
        canonical::{Canonical, CanonicalizeInto as _},
        clock::DateTime,
    },
};

use crate::{
    util::{
        digest::MediaDigest,
        guess_mime_from_path, media_type_from_image_format,
        tag::{FacetedTagMappingConfig, TagMappingConfig},
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
    pub struct ImportTrackFlags: u16 {
        const METADATA                            = 0b0000_0000_0000_0001;
        const EMBEDDED_ARTWORK                    = 0b0000_0000_0000_0011; // implies METADATA
        const ARTWORK_DIGEST                      = 0b0000_0000_0000_0111; // Hash cover image, implies EMBEDDED_ARTWORK
        // Custom application metadata
        const ITUNES_ID3V2_GROUPING_MOVEMENT_WORK = 0b0000_0001_0000_0000; // ID3v2 with iTunes v12.5.4 and newer
        const AOIDE_TAGS                          = 0b0000_0010_0000_0001; // implies METADATA
        const SERATO_MARKERS                      = 0b0000_0100_0000_0001; // implies METADATA
    }
}

impl ImportTrackFlags {
    #[must_use]
    pub fn is_valid(self) -> bool {
        Self::all().contains(self)
    }

    #[must_use]
    pub fn new_artwork_digest(self) -> MediaDigest {
        if self.contains(Self::ARTWORK_DIGEST) {
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

pub fn import_into_track(
    reader: &mut Box<dyn Reader>,
    config: &ImportTrackConfig,
    track: &mut Track,
) -> Result<()> {
    match track.media_source.content_type.essence_str() {
        #[cfg(feature = "fmt-flac")]
        "audio/flac" => crate::fmt::flac::Metadata::read_from(reader)
            .and_then(|metadata| metadata.import_into_track(config, track)),
        #[cfg(feature = "fmt-mp3")]
        "audio/mpeg" => crate::fmt::mp3::MetadataExt::read_from(reader)
            .and_then(|metadata_ext| metadata_ext.import_into_track(config, track)),
        #[cfg(feature = "fmt-mp4")]
        "audio/m4a" | "video/mp4" => crate::fmt::mp4::Metadata::read_from(reader)
            .and_then(|metadata| metadata.import_into_track(config, track)),
        #[cfg(feature = "fmt-ogg")]
        "audio/ogg" | "audio/vorbis" => crate::fmt::ogg::Metadata::read_from(reader)
            .and_then(|metadata| metadata.import_into_track(config, track)),
        #[cfg(feature = "fmt-opus")]
        "audio/opus" => crate::fmt::opus::Metadata::read_from(reader)
            .and_then(|metadata| metadata.import_into_track(config, track)),
        _ => Err(Error::UnsupportedContentType(
            track.media_source.content_type.to_owned(),
        )),
    }
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
                    .find_embedded_artwork_image()
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
                .find_embedded_artwork_image()
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

#[must_use]
pub fn finish_import_of_titles(
    source_path: &SourcePath,
    titles: Vec<Title>,
) -> Canonical<Vec<Title>> {
    let titles_len = titles.len();
    let titles = titles.canonicalize_into();
    if titles.len() < titles_len {
        log::warn!(
            "Discarded {} duplicate track titles imported from {}",
            titles_len - titles.len(),
            source_path
        );
    }
    Canonical::tie(titles)
}

#[must_use]
pub fn finish_import_of_actors(
    source_path: &SourcePath,
    actors: Vec<Actor>,
) -> Canonical<Vec<Actor>> {
    let actors_len = actors.len();
    let actors = actors.canonicalize_into();
    if actors.len() < actors_len {
        log::warn!(
            "Discarded {} duplicate track actors imported from {}",
            actors_len - actors.len(),
            source_path
        );
    }
    Canonical::tie(actors)
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

pub fn import_plain_tags_from_joined_label_value<'a>(
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
                for label_value in joined_label_value.split(&tag_mapping_config.label_separator) {
                    let label = TagLabel::clamp_value(label_value);
                    match try_import_plain_tag(label, *next_score_value) {
                        Ok(plain_tag) => {
                            plain_tags.push(plain_tag);
                            import_count += 1;
                            *next_score_value =
                                tag_mapping_config.next_score_value(*next_score_value);
                        }
                        Err(plain_tag) => {
                            log::warn!("Failed to import plain tag: {:?}", plain_tag,);
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
                        *next_score_value = tag_mapping_config.next_score_value(*next_score_value);
                    }
                }
                Err(plain_tag) => {
                    log::warn!("Failed to import plain tag: {:?}", plain_tag,);
                }
            }
        }
        import_count
    } else {
        log::debug!("Skipping empty tag label");
        0
    }
}

pub fn import_faceted_tags_from_label_values(
    source_path: &SourcePath,
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
        total_import_count += import_plain_tags_from_joined_label_value(
            tag_mapping_config,
            &mut next_score_value,
            &mut plain_tags,
            label_value,
        );
    }
    let count = plain_tags.len();
    if count < total_import_count {
        log::warn!(
            "Discarded {} duplicate tag labels for facet {} imported from {}",
            total_import_count - count,
            facet_id,
            source_path
        );
    }
    if plain_tags.is_empty() {
        return 0;
    }
    tags_map.update_faceted_plain_tags_by_label_ordering(facet_id, plain_tags);
    count
}
