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
    pub use aoide_core::media::*;
}

use crate::{audio::AudioContent, util::color::ColorRgb};

///////////////////////////////////////////////////////////////////////
// Content
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Content {
    #[serde(rename = "a")]
    Audio(AudioContent),
}

impl From<Content> for _core::Content {
    fn from(from: Content) -> Self {
        use _core::Content::*;
        match from {
            Content::Audio(audio_content) => Audio(audio_content.into()),
        }
    }
}

impl From<_core::Content> for Content {
    fn from(from: _core::Content) -> Self {
        use _core::Content::*;
        match from {
            Audio(audio_content) => Content::Audio(audio_content.into()),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Artwork
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(deny_unknown_fields)]
pub struct ImageSize(u16, u16);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(deny_unknown_fields)]
pub struct Artwork {
    #[serde(rename = "s")]
    size: ImageSize,

    #[serde(rename = "f", skip_serializing_if = "Option::is_none")]
    fingerprint: Option<String>,

    #[serde(rename = "u", skip_serializing_if = "Option::is_none")]
    uri: Option<String>,

    #[serde(rename = "b", skip_serializing_if = "Option::is_none")]
    background_color: Option<ColorRgb>,
}

impl From<_core::Artwork> for Artwork {
    fn from(from: _core::Artwork) -> Self {
        let _core::Artwork { size: _core::ImageSize { width, height }, fingerprint, uri, background_color } = from;
        Self { size: ImageSize(width, height), fingerprint, uri, background_color: background_color.map(Into::into) }
    }
}

impl From<Artwork> for _core::Artwork {
    fn from(from: Artwork) -> Self {
        let Artwork { size: ImageSize(width, height), fingerprint, uri, background_color } = from;
        Self { size: _core::ImageSize { width, height }, fingerprint, uri, background_color: background_color.map(Into::into) }
    }
}

///////////////////////////////////////////////////////////////////////
// Source
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(deny_unknown_fields)]
pub struct Source {
    #[serde(rename = "u")]
    uri: String,

    #[serde(rename = "t")]
    content_type: String,

    #[serde(rename = "c")]
    content: Content,

    #[serde(rename = "a", skip_serializing_if = "Option::is_none")]
    cover_art: Option<Artwork>,
}

impl From<_core::Source> for Source {
    fn from(from: _core::Source) -> Self {
        let _core::Source {
            uri,
            content_type,
            content,
            cover_art,
        } = from;
        Self {
            uri,
            content_type,
            content: content.into(),
            cover_art: cover_art.map(Into::into),
        }
    }
}

impl From<Source> for _core::Source {
    fn from(from: Source) -> Self {
        let Source {
            uri,
            content_type,
            content,
            cover_art
        } = from;
        Self {
            uri,
            content_type,
            content: content.into(),
            cover_art: cover_art.map(Into::into),
        }
    }
}
