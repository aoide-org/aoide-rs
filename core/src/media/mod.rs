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
    audio::{AudioContent, AudioContentInvalidity},
    prelude::*,
};

use num_derive::{FromPrimitive, ToPrimitive};

///////////////////////////////////////////////////////////////////////
// Content
///////////////////////////////////////////////////////////////////////

/// Reliability of content metadata.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, FromPrimitive, ToPrimitive)]
pub enum ContentMetadataStatus {
    Unknown = 0,

    /// Unreliable
    ///
    /// Example: Parsed from file tags which are considered inaccurate
    /// and are often imprecise.
    Unreliable = 1,

    /// Reliable
    ///
    /// Example: Reported by a decoder when opening an audio/video
    /// stream for reading. Nevertheless different decoders may report
    /// slightly differing values.
    Reliable = 2,

    /// Confirmed, acknowledged, or manually set.
    ///
    /// Locked metadata will not be updated automatically, neither when
    /// parsing file tags nor when decoding an audio/video stream.
    Locked = 3,
}

impl Default for ContentMetadataStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Content {
    Audio(AudioContent),
}

impl From<AudioContent> for Content {
    fn from(from: AudioContent) -> Self {
        Self::Audio(from)
    }
}

///////////////////////////////////////////////////////////////////////
// Artwork
///////////////////////////////////////////////////////////////////////

pub type ImageDimension = u16;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ImageSize {
    pub width: ImageDimension,
    pub height: ImageDimension,
}

impl ImageSize {
    pub const fn is_empty(self) -> bool {
        !(self.width > 0 && self.height > 0)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ImageSizeInvalidity {
    Empty,
}

impl Validate for ImageSize {
    type Invalidity = ImageSizeInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.is_empty(), ImageSizeInvalidity::Empty)
            .into()
    }
}

// All artwork properties are optional for maximum flexibility.
// Properties could be missing or are yet unknown at some point
// in time.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Artwork {
    /// The URI of an external resource
    pub uri: Option<String>,

    /// The media type (if known), e.g. "image/jpeg"
    pub media_type: Option<String>,

    /// Identifies the actual content for cache lookup and to decide
    /// about modifications, e.g. a base64-encoded SHA256 hash of the
    /// raw image data.
    pub digest: Option<Vec<u8>>,

    /// The dimensions of the image (if known).
    pub size: Option<ImageSize>,

    /// An optional (background) color can be used to quickly display
    /// a preliminary view before the actual image has been loaded and
    /// for selecting a matching color scheme.
    pub color_rgb: Option<RgbColor>,
}

impl Artwork {
    pub fn is_empty(&self) -> bool {
        let Self {
            digest,
            size,
            color_rgb,
            ..
        } = self;
        digest.is_none() && size.is_none() && color_rgb.is_none()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ArtworkInvalidity {
    MediaTypeEmpty,
    DigestEmpty,
    ImageSize(ImageSizeInvalidity),
    RgbColor(RgbColorInvalidity),
}

impl Validate for Artwork {
    type Invalidity = ArtworkInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                self.media_type
                    .as_ref()
                    .map(String::is_empty)
                    .unwrap_or(false),
                Self::Invalidity::MediaTypeEmpty,
            )
            .invalidate_if(
                self.digest.as_ref().map(Vec::is_empty).unwrap_or(false),
                Self::Invalidity::DigestEmpty,
            )
            .validate_with(&self.size, Self::Invalidity::ImageSize)
            .validate_with(&self.color_rgb, Self::Invalidity::RgbColor)
            .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Source
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    pub collected_at: DateTime,

    pub synchronized_at: Option<DateTime>,

    pub uri: String,

    pub content_type: String,

    pub content_digest: Option<Vec<u8>>,

    pub content_metadata_status: ContentMetadataStatus,

    pub content: Content,

    pub artwork: Artwork,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SourceInvalidity {
    UriEmpty,
    ContentTypeEmpty,
    AudioContent(AudioContentInvalidity),
    Artwork(ArtworkInvalidity),
}

impl Validate for Source {
    type Invalidity = SourceInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let context = ValidationContext::new()
            .invalidate_if(self.uri.trim().is_empty(), Self::Invalidity::UriEmpty)
            .invalidate_if(
                self.content_type.trim().is_empty(),
                Self::Invalidity::ContentTypeEmpty,
            )
            .validate_with(&self.artwork, Self::Invalidity::Artwork);
        // TODO: Validate MIME type
        match self.content {
            Content::Audio(ref audio_content) => {
                context.validate_with(audio_content, Self::Invalidity::AudioContent)
            }
        }
        .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceUri {
    pub uri: String,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SourceUriInvalidity {
    UriEmpty,
}

impl Validate for SourceUri {
    type Invalidity = SourceUriInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.uri.trim().is_empty(), Self::Invalidity::UriEmpty)
            .into()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SourceOrUri {
    Source(Source),
    Uri(SourceUri),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SourceOrUriInvalidity {
    Source(SourceInvalidity),
    Uri(SourceUriInvalidity),
}

impl Validate for SourceOrUri {
    type Invalidity = SourceOrUriInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let context = ValidationContext::new();
        use SourceOrUri::*;
        match self {
            Source(source) => context.validate_with(source, Self::Invalidity::Source),
            Uri(source_uri) => context.validate_with(source_uri, Self::Invalidity::Uri),
        }
        .into()
    }
}
