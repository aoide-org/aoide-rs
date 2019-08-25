// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use super::*;

mod _core {
    pub use aoide_core::track::source::*;
}

use crate::audio::AudioContent;

///////////////////////////////////////////////////////////////////////
// MediaContent
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub enum MediaContent {
    #[serde(rename = "a")]
    Audio(AudioContent),
}

impl From<MediaContent> for _core::MediaContent {
    fn from(from: MediaContent) -> Self {
        use _core::MediaContent::*;
        match from {
            MediaContent::Audio(audio_content) => Audio(audio_content.into()),
        }
    }
}

impl From<_core::MediaContent> for MediaContent {
    fn from(from: _core::MediaContent) -> Self {
        use _core::MediaContent::*;
        match from {
            Audio(audio_content) => MediaContent::Audio(audio_content.into()),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Source
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(deny_unknown_fields)]
pub struct MediaSource(String, String, MediaContent);

impl From<_core::MediaSource> for MediaSource {
    fn from(from: _core::MediaSource) -> Self {
        let _core::MediaSource {
            uri,
            content_type,
            content,
        } = from;
        Self(uri, content_type, content.into())
    }
}

impl From<MediaSource> for _core::MediaSource {
    fn from(from: MediaSource) -> Self {
        let MediaSource(uri, content_type, content) = from;
        Self {
            uri,
            content_type,
            content: content.into(),
        }
    }
}
