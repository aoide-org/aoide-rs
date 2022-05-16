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

use mime::Mime;
use num_derive::{FromPrimitive, ToPrimitive};

use crate::prelude::*;

/// The `APIC` picture type code as defined by `ID3v2`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum ApicType {
    Other = 0x00,
    Icon = 0x01,
    OtherIcon = 0x02,
    CoverFront = 0x03,
    CoverBack = 0x04,
    Leaflet = 0x05,
    Media = 0x06,
    LeadArtist = 0x07,
    Artist = 0x08,
    Conductor = 0x09,
    Band = 0x0A,
    Composer = 0x0B,
    Lyricist = 0x0C,
    RecordingLocation = 0x0D,
    DuringRecording = 0x0E,
    DuringPerformance = 0x0F,
    ScreenCapture = 0x10,
    BrightFish = 0x11,
    Illustration = 0x12,
    BandLogo = 0x13,
    PublisherLogo = 0x14,
}

pub type ImageDimension = u16;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ImageSize {
    pub width: ImageDimension,
    pub height: ImageDimension,
}

impl ImageSize {
    #[must_use]
    pub const fn is_empty(self) -> bool {
        !(self.width > 0 && self.height > 0)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ImageSizeInvalidity {
    Empty,
}

impl Validate for ImageSize {
    type Invalidity = ImageSizeInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.is_empty(), Self::Invalidity::Empty)
            .into()
    }
}

pub type Digest = [u8; 32];

pub type Thumbnail4x4Rgb8 = [u8; 4 * 4 * 3];

/// Artwork image properties
///
/// All properties are optional for maximum flexibility.
/// Properties could be missing or are yet unknown at some point
/// in time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtworkImage {
    pub media_type: Mime,

    pub apic_type: ApicType,

    /// The dimensions of the image (if known).
    pub size: Option<ImageSize>,

    /// Identifies the actual content, e.g. for cache lookup or to detect
    /// modifications.
    pub digest: Option<Digest>,

    /// A 4x4 R8G8B8 thumbnail image.
    pub thumbnail: Option<Thumbnail4x4Rgb8>,
}

#[derive(Copy, Clone, Debug)]
pub enum ArtworkImageInvalidity {
    MediaTypeEmpty,
    Size(ImageSizeInvalidity),
}

impl Validate for ArtworkImage {
    type Invalidity = ArtworkImageInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                self.media_type.essence_str().is_empty(),
                Self::Invalidity::MediaTypeEmpty,
            )
            .validate_with(&self.size, Self::Invalidity::Size)
            .into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedArtwork {
    pub image: ArtworkImage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkedArtwork {
    /// Absolute or relative URI/URL that links to the image.
    pub uri: String,

    pub image: ArtworkImage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Artwork {
    /// Artwork has been looked up at least once but nothing has been found.
    Missing,

    /// Artwork has been looked up at least once but the media type was not supported (yet).
    Unsupported,

    /// Artwork has been looked up at least once but the import failed unexpectedly.
    Irregular,

    /// The artwork is embedded in the media source.
    Embedded(EmbeddedArtwork),

    /// The artwork references an external image.
    Linked(LinkedArtwork),
}

#[derive(Copy, Clone, Debug)]
pub enum ArtworkInvalidity {
    Image(ArtworkImageInvalidity),
}

impl Validate for Artwork {
    type Invalidity = ArtworkInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let mut context = ValidationContext::new();
        match self {
            Self::Missing | Self::Unsupported | Self::Irregular => (),
            Self::Embedded(embedded) => {
                context = context.validate_with(&embedded.image, Self::Invalidity::Image);
            }
            Self::Linked(linked) => {
                // TODO: Validate uri
                context = context.validate_with(&linked.image, Self::Invalidity::Image);
            }
        }
        context.into()
    }
}
