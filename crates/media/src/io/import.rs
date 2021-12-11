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
    fmt::{flac, mp3, mp4, ogg},
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
    pub fn into_new_track(self, path: SourcePath, mime: &Mime) -> Track {
        let Self {
            collected_at,
            synchronized_at,
        } = self;
        let media_source = Source {
            collected_at,
            synchronized_at: Some(synchronized_at),
            path,
            content_type: mime.to_string(),
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
    mime: Mime,
    reader: &mut Box<dyn Reader>,
    config: &ImportTrackConfig,
    track: &mut Track,
) -> Result<()> {
    match mime.essence_str() {
        "audio/flac" => flac::Metadata::read_from(reader)
            .and_then(|metadata| metadata.import_into_track(config, track)),
        "audio/mpeg" => mp3::MetadataExt::read_from(reader)
            .and_then(|metadata_ext| metadata_ext.import_into_track(config, track)),
        "audio/m4a" | "video/mp4" => mp4::Metadata::read_from(reader)
            .and_then(|metadata| metadata.import_into_track(config, track)),
        "audio/ogg" => ogg::Metadata::read_from(reader)
            .and_then(|metadata| metadata.import_into_track(config, track)),
        _ => Err(Error::UnsupportedContentType(mime)),
    }
    .map_err(|err| {
        tracing::warn!(
            "Failed to parse metadata from media source '{}': {}",
            track.media_source.path,
            err
        );
        err
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedArtworkImageData {
    pub apic_type: ApicType,
    pub media_type: String,
    pub image_data: Vec<u8>,
}

pub fn load_embedded_artwork_image_data_from_file_path(
    file_path: &Path,
) -> Result<Option<EmbeddedArtworkImageData>> {
    let file = File::open(file_path)?;
    let mut reader: Box<dyn Reader> = Box::new(BufReader::new(file));
    let mime = guess_mime_from_path(&file_path)?;
    match mime.as_ref() {
        "audio/flac" => flac::Metadata::read_from(&mut reader).map(|metadata| {
            metadata
                .find_embedded_artwork_image()
                .map(
                    |(apic_type, media_type, image_data)| EmbeddedArtworkImageData {
                        apic_type,
                        media_type: media_type.to_owned(),
                        image_data: image_data.to_owned(),
                    },
                )
        }),
        "audio/mpeg" => mp3::Metadata::read_from(&mut reader).map(|metadata| {
            metadata
                .find_embedded_artwork_image()
                .map(
                    |(apic_type, media_type, image_data)| EmbeddedArtworkImageData {
                        apic_type,
                        media_type: media_type.to_owned(),
                        image_data: image_data.to_owned(),
                    },
                )
        }),
        "audio/m4a" | "video/mp4" => mp4::Metadata::read_from(&mut reader).and_then(|metadata| {
            metadata
                .find_embedded_artwork_image()
                .map(|(apic_type, image_format, image_data)| {
                    Ok(EmbeddedArtworkImageData {
                        apic_type,
                        media_type: media_type_from_image_format(image_format)?,
                        image_data: image_data.to_owned(),
                    })
                })
                .transpose()
        }),
        "audio/ogg" => ogg::Metadata::read_from(&mut reader).map(|metadata| {
            metadata
                .find_embedded_artwork_image()
                .map(
                    |(apic_type, media_type, image_data)| EmbeddedArtworkImageData {
                        apic_type,
                        media_type,
                        image_data,
                    },
                )
        }),
        _ => Err(Error::UnsupportedContentType(mime)),
    }
}
