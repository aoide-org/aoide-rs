// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use data_encoding::BASE64_NOPAD;
use image::{codecs::png::PngEncoder, ImageEncoder as _};
use mime::Mime;
use strum::FromRepr;

use crate::prelude::*;

/// The `APIC` picture type code as defined by `ID3v2`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
#[repr(u8)]
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

pub const THUMBNAIL_WIDTH: ImageDimension = 4;

pub const THUMBNAIL_HEIGHT: ImageDimension = 4;

pub type Thumbnail4x4Rgb8 = [u8; (THUMBNAIL_WIDTH * THUMBNAIL_HEIGHT * 3) as _];

/// Create an image from thumbnail data
#[must_use]
#[allow(clippy::missing_panics_doc)] // Never panics
pub fn thumbnail_image(thumbnail: &Thumbnail4x4Rgb8) -> image::RgbImage {
    image::RgbImage::from_raw(
        THUMBNAIL_WIDTH.into(),
        THUMBNAIL_HEIGHT.into(),
        thumbnail.to_vec(),
    )
    .expect("Some")
}

/// Create an ICO [data URI](<https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/Data_URLs>)
/// from thumbnail data
#[must_use]
#[allow(clippy::missing_panics_doc)] // Never panics
pub fn thumbnail_png_data_uri(thumbnail: &Thumbnail4x4Rgb8) -> String {
    const DATA_URI_PREFIX: &str = "data:image/png;base64,";
    let mut png_data = Vec::with_capacity(192);
    let png_encoder = PngEncoder::new(&mut png_data);
    png_encoder
        .write_image(
            thumbnail,
            THUMBNAIL_WIDTH.into(),
            THUMBNAIL_HEIGHT.into(),
            image::ColorType::Rgb8,
        )
        .expect("infallible");
    debug_assert!(png_data.len() <= 192);
    let encoded_len = BASE64_NOPAD.encode_len(png_data.len());
    let data_uri_len = DATA_URI_PREFIX.len() + encoded_len;
    let mut data_uri = String::with_capacity(data_uri_len);
    data_uri.push_str(DATA_URI_PREFIX);
    BASE64_NOPAD
        .encode_write(&png_data, &mut data_uri)
        .expect("infallible");
    data_uri
}

/// Artwork image properties
///
/// All properties are optional for maximum flexibility.
/// Properties could be missing or are yet unknown at some point
/// in time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtworkImage {
    pub apic_type: ApicType,

    pub media_type: Mime,

    pub data_size: u64,

    /// Identifies the actual content, e.g. for cache lookup or to detect
    /// modifications.
    pub digest: Option<Digest>,

    /// The dimensions of the image (if known).
    pub image_size: Option<ImageSize>,

    /// The predominant color in the image.
    pub color: Option<RgbColor>,

    /// A 4x4 R8G8B8 thumbnail image.
    pub thumbnail: Option<Thumbnail4x4Rgb8>,
}

#[derive(Copy, Clone, Debug)]
pub enum ArtworkImageInvalidity {
    MediaTypeEmpty,
    ImageSize(ImageSizeInvalidity),
}

impl Validate for ArtworkImage {
    type Invalidity = ArtworkImageInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                self.media_type.essence_str().is_empty(),
                Self::Invalidity::MediaTypeEmpty,
            )
            .validate_with(&self.image_size, Self::Invalidity::ImageSize)
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

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use data_url::DataUrl;
    use image::{codecs::png::PngDecoder, ImageDecoder as _};

    use super::{THUMBNAIL_HEIGHT, THUMBNAIL_WIDTH};

    #[test]
    fn encode_and_decode_thumbnail_as_data_uri() {
        for r in [0x00u8, 0xffu8] {
            for g in [0x00u8, 0xffu8] {
                for b in [0x00u8, 0xffu8] {
                    let pixel = [r, g, b];
                    let thumbnail_data = std::iter::repeat(pixel)
                        .take((THUMBNAIL_WIDTH * THUMBNAIL_HEIGHT) as _)
                        .flatten()
                        .collect::<Vec<_>>();
                    let thumbnail = thumbnail_data.clone().try_into().unwrap();
                    let thumbnail_uri = super::thumbnail_png_data_uri(&thumbnail);
                    let data_url = DataUrl::process(&thumbnail_uri).unwrap();
                    let mime_type = data_url.mime_type();
                    assert_eq!("image", mime_type.type_);
                    assert_eq!("png", mime_type.subtype);
                    assert!(mime_type.parameters.is_empty());
                    let (png_data, fragment_identifier) = data_url.decode_to_vec().unwrap();
                    assert!(!png_data.is_empty());
                    assert!(fragment_identifier.is_none());
                    let png_data_cursor = Cursor::new(png_data);
                    let png_decoder = PngDecoder::new(png_data_cursor).unwrap();
                    let mut decoded_data = [0; (THUMBNAIL_WIDTH * THUMBNAIL_HEIGHT * 3) as _];
                    png_decoder.read_image(&mut decoded_data).unwrap();
                    assert_eq!(thumbnail_data, decoded_data);
                }
            }
        }
    }
}
