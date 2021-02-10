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
    media::{Content, Source},
    track::Track,
    util::clock::DateTime,
};

use bitflags::bitflags;
use mime::Mime;
use std::io::{Read, Seek};
use url::Url;

#[rustfmt::skip]
bitflags! {
    pub struct ImportTrackFlags: u16 {
        const METADATA                            = 0b0000000000000001;
        const ARTWORK                             = 0b0000000000000010;
        const CONTENT_DIGEST                      = 0b0000000000000100;
        const ARTWORK_DIGEST                      = 0b0000000000001010; // implies ARTWORK
        const ARTWORK_DIGEST_SHA256               = 0b0000000000011010; // Use SHA-256 instead of BLAKE3 (e.g. for Mixxx)
        // Custom application metadata
        const ITUNES_ID3V2_GROUPING_MOVEMENT_WORK = 0b0000000100000000; // ID3v2 with iTunes v12.5.4 and newer
        const MIXXX_CUSTOM_TAGS                   = 0b0000001000000001; // implies METADATA
        const MIXXX_KEEP_CUSTOM_GENRE_TAGS        = 0b0000010000000000;
        const MIXXX_KEEP_CUSTOM_MOOD_TAGS         = 0b0000100000000000;
        const SERATO_TAGS                         = 0b0001000000000001; // implies METADATA
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
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NewTrackInput {
    pub collected_at: DateTime,
    pub synchronized_at: DateTime,
}

impl NewTrackInput {
    pub fn try_from_url_into_new_track(self, url: &Url, mime: &Mime) -> Result<Track> {
        let Self {
            collected_at,
            synchronized_at,
        } = self;
        let media_source = Source {
            collected_at,
            synchronized_at: Some(synchronized_at),
            uri: url.to_string(),
            content_type: mime.to_string(),
            content_digest: None,
            content_metadata_flags: Default::default(),
            content: Content::Audio(Default::default()),
            artwork: Default::default(),
        };
        Ok(Track::new_from_media_source(media_source))
    }
}

pub trait Reader: Read + Seek + 'static {}

impl<T> Reader for T where T: Read + Seek + 'static {}

pub trait ImportTrack {
    fn import_track(
        &self,
        config: &ImportTrackConfig,
        flags: ImportTrackFlags,
        track: Track,
        reader: &mut Box<dyn Reader>,
    ) -> Result<Track>;
}
