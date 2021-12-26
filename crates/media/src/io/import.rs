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

use crate::{
    util::{
        digest::MediaDigest, guess_mime_from_path, media_type_from_image_format,
        tag::FacetedTagMappingConfig,
    },
    Error, Result,
};

use aoide_core::{
    media::{ApicType, Content, Source, SourcePath},
    track::Track,
    util::clock::DateTime,
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
    pub fn is_valid(self) -> bool {
        Self::all().contains(self)
    }

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
