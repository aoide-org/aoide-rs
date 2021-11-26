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

///////////////////////////////////////////////////////////////////////

use crate::{util::tag::FacetedTagMappingConfig, Result};

use aoide_core::{
    media::{Content, Source, SourcePath},
    track::Track,
    util::clock::DateTime,
};

use bitflags::bitflags;
use mime::Mime;
use std::io::{Read, Seek};

#[rustfmt::skip]
bitflags! {
    pub struct ImportTrackFlags: u16 {
        const METADATA                            = 0b0000_0000_0000_0001;
        const EMBEDDED_ARTWORK                    = 0b0000_0000_0000_0010;
        const CONTENT_DIGEST                      = 0b0000_0000_0000_0100;
        const ARTWORK_DIGEST                      = 0b0000_0000_0000_1010; // implies ARTWORK
        const ARTWORK_DIGEST_SHA256               = 0b0000_0000_0001_1010; // Use SHA-256 instead of BLAKE3 (e.g. for Mixxx)
        // Custom application metadata
        const ITUNES_ID3V2_GROUPING_MOVEMENT_WORK = 0b0000_0001_0000_0000; // ID3v2 with iTunes v12.5.4 and newer
        const MIXXX_CUSTOM_TAGS                   = 0b0000_0010_0000_0001; // implies METADATA
        const MIXXX_KEEP_CUSTOM_GENRE_TAGS        = 0b0000_0100_0000_0000;
        const MIXXX_KEEP_CUSTOM_MOOD_TAGS         = 0b0000_1000_0000_0000;
        const SERATO_TAGS                         = 0b0001_0000_0000_0001; // implies METADATA
    }
}

impl ImportTrackFlags {
    pub fn is_valid(self) -> bool {
        Self::all().contains(self)
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

pub trait ImportTrack {
    fn import_track(
        &self,
        reader: &mut Box<dyn Reader>,
        config: &ImportTrackConfig,
        track: &mut Track,
    ) -> Result<()>;
}
