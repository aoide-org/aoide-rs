// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Content {
    #[serde(rename = "aud")]
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageSize(u16, u16);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artwork {
    #[serde(rename = "dim", skip_serializing_if = "Option::is_none")]
    size: Option<ImageSize>,

    #[serde(rename = "col", skip_serializing_if = "Option::is_none")]
    color: Option<ColorRgb>,

    #[serde(rename = "dig", skip_serializing_if = "Option::is_none")]
    digest: Option<String>,

    #[serde(rename = "uri", skip_serializing_if = "Option::is_none")]
    uri: Option<String>,
}

impl From<_core::Artwork> for Artwork {
    fn from(from: _core::Artwork) -> Self {
        let _core::Artwork {
            size,
            color,
            digest,
            uri,
        } = from;
        let size = size.map(|size| {
            let _core::ImageSize { width, height } = size;
            ImageSize(width, height)
        });
        Self {
            size,
            color: color.map(Into::into),
            digest,
            uri,
        }
    }
}

impl From<Artwork> for _core::Artwork {
    fn from(from: Artwork) -> Self {
        let Artwork {
            size,
            color,
            digest,
            uri,
        } = from;
        let size = size.map(|size| {
            let ImageSize(width, height) = size;
            _core::ImageSize { width, height }
        });
        Self {
            size,
            color: color.map(Into::into),
            digest,
            uri,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Source
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    #[serde(rename = "uri")]
    uri: String,

    #[serde(rename = "typ")]
    content_type: String,

    #[serde(flatten)]
    content: Content,

    #[serde(rename = "art", skip_serializing_if = "Option::is_none")]
    artwork: Option<Artwork>,
}

impl From<_core::Source> for Source {
    fn from(from: _core::Source) -> Self {
        let _core::Source {
            uri,
            content_type,
            content,
            artwork,
        } = from;
        Self {
            uri,
            content_type,
            content: content.into(),
            artwork: artwork.map(Into::into),
        }
    }
}

impl From<Source> for _core::Source {
    fn from(from: Source) -> Self {
        let Source {
            uri,
            content_type,
            content,
            artwork,
        } = from;
        Self {
            uri,
            content_type,
            content: content.into(),
            artwork: artwork.map(Into::into),
        }
    }
}
