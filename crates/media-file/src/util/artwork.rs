// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::convert::TryFrom as _;

use aoide_core::{
    media::artwork::{
        ApicType, Artwork, ArtworkImage, EmbeddedArtwork, ImageDimension, ImageSize,
        Thumbnail4x4Rgb8, THUMBNAIL_HEIGHT, THUMBNAIL_WIDTH,
    },
    util::color::RgbColor,
};
use image::{
    guess_format, load_from_memory, load_from_memory_with_format, DynamicImage, GenericImageView,
    ImageError, ImageFormat,
};
use mime::{Mime, IMAGE_BMP, IMAGE_GIF, IMAGE_JPEG, IMAGE_PNG, IMAGE_STAR};
use thiserror::Error;

use super::digest::MediaDigest;
use crate::Result;

#[derive(Debug, Error)]
pub enum ArtworkImageError {
    #[error("unsupported format {0:?}")]
    UnsupportedFormat(ImageFormat),

    #[error(transparent)]
    Image(#[from] ImageError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<ArtworkImageError> for crate::Error {
    fn from(err: ArtworkImageError) -> crate::Error {
        match err {
            ArtworkImageError::UnsupportedFormat(image_format) => Self::Metadata(anyhow::anyhow!(
                "Unsupported artwork image format: {image_format:?}"
            )),
            ArtworkImageError::Image(err) => Self::Metadata(err.into()),
            ArtworkImageError::Other(err) => Self::Metadata(err),
        }
    }
}

#[derive(Debug)]
pub struct LoadedArtworkPicture {
    pub media_type: Mime,
    pub picture: DynamicImage,
    pub recoverable_errors: Vec<anyhow::Error>,
}

pub type LoadArtworkPictureResult = std::result::Result<LoadedArtworkPicture, ArtworkImageError>;

#[allow(clippy::missing_panics_doc)] // Never panics
pub fn media_type_from_image_format(
    image_format: ImageFormat,
) -> std::result::Result<Mime, ArtworkImageError> {
    let media_type = match image_format {
        ImageFormat::Jpeg => IMAGE_JPEG,
        ImageFormat::Png => IMAGE_PNG,
        ImageFormat::Gif => IMAGE_GIF,
        ImageFormat::Bmp => IMAGE_BMP,
        ImageFormat::WebP => "image/webp".parse().expect("valid MIME type"),
        ImageFormat::Tiff => "image/tiff".parse().expect("valid MIME type"),
        ImageFormat::Tga => "image/tga".parse().expect("valid MIME type"),
        unsupported_format => {
            return Err(ArtworkImageError::UnsupportedFormat(unsupported_format));
        }
    };
    Ok(media_type)
}

pub fn load_artwork_picture(
    image_data: &[u8],
    image_format_hint: Option<ImageFormat>,
    media_type_hint: Option<&str>,
) -> LoadArtworkPictureResult {
    let image_format = image_format_hint.or_else(|| guess_format(image_data).ok());
    let mut recoverable_errors = Vec::new();
    let picture = if let Some(image_format) = image_format {
        load_from_memory_with_format(image_data, image_format)
    } else {
        load_from_memory(image_data)
    }?;
    let media_type = media_type_hint
        .and_then(|media_type_hint| {
            media_type_hint
                .parse::<Mime>()
                .map_err(|err| {
                    recoverable_errors.push(anyhow::anyhow!(
                        "Failed to parse MIME type from '{media_type_hint}': {err}"
                    ));
                    err
                })
                // Ignore and continue
                .ok()
        })
        .map(Ok)
        .or_else(|| image_format.map(media_type_from_image_format))
        .transpose()?
        .unwrap_or(IMAGE_STAR);
    Ok(LoadedArtworkPicture {
        media_type,
        picture,
        recoverable_errors,
    })
}

#[derive(Debug)]
pub struct IngestedArtworkImage {
    pub artwork_image: ArtworkImage,
    pub picture: DynamicImage,
    pub recoverable_errors: Vec<anyhow::Error>,
}

type IngestArtworkImageResult = std::result::Result<IngestedArtworkImage, ArtworkImageError>;

// [1..10], 1 = visit each pixel (maximum quality)
const COLOR_THIEF_QUALITY: u8 = 7;

// [2..255], copied from getColor() in <https://github.com/lokesh/color-thief>
const COLOR_THIEF_PALETTE_MAX_COLORS: u8 = 5;

fn ingest_artwork_image(
    apic_type: ApicType,
    image_data: &[u8],
    image_format_hint: Option<ImageFormat>,
    media_type_hint: Option<&str>,
    image_digest: &mut MediaDigest,
) -> IngestArtworkImageResult {
    let LoadedArtworkPicture {
        media_type,
        picture,
        recoverable_errors,
    } = load_artwork_picture(image_data, image_format_hint, media_type_hint)?;
    let (width, height) = picture.dimensions();
    let width = ImageDimension::try_from(width)
        .map_err(|_| anyhow::anyhow!("Unsupported image size: {width}x{height}"))?;
    let height = ImageDimension::try_from(height)
        .map_err(|_| anyhow::anyhow!("Unsupported image size: {width}x{height}"))?;
    let size = ImageSize { width, height };
    let digest = image_digest.digest_content(image_data).finalize_reset();
    let color_thief_format = match picture.color() {
        image::ColorType::Rgb8 => Some(color_thief::ColorFormat::Rgb),
        image::ColorType::Rgba8 => Some(color_thief::ColorFormat::Rgba),
        _ => {
            log::warn!(
                "Unsupported color type {color_type:?} for extracting the predominant color from \
                 artwork image",
                color_type = picture.color()
            );
            None
        }
    };
    let color = color_thief_format
        .and_then(|color_format| {
            color_thief::get_palette(
                picture.as_bytes(),
                color_format,
                COLOR_THIEF_QUALITY,
                COLOR_THIEF_PALETTE_MAX_COLORS,
            )
            .map_err(|err| {
                log::warn!("Failed to extract the predominant color from artwork image: {err}");
            })
            .ok()
        })
        .and_then(|palette| palette.first().copied())
        .map(|rgb| RgbColor::new(rgb.r, rgb.g, rgb.b));
    let thumbnail_picture = picture.resize_exact(
        THUMBNAIL_WIDTH.try_into().expect("infallible"),
        THUMBNAIL_HEIGHT.try_into().expect("infallible"),
        image::imageops::FilterType::Lanczos3,
    );
    let thumbnail = Thumbnail4x4Rgb8::try_from(thumbnail_picture.to_rgb8().into_raw()).ok();
    debug_assert!(thumbnail.is_some());
    let artwork_image = ArtworkImage {
        media_type,
        apic_type,
        size: Some(size),
        digest,
        color,
        thumbnail,
    };
    Ok(IngestedArtworkImage {
        artwork_image,
        picture,
        recoverable_errors,
    })
}

pub fn try_ingest_embedded_artwork_image(
    apic_type: ApicType,
    image_data: &[u8],
    image_format_hint: Option<ImageFormat>,
    media_type_hint: Option<&str>,
    image_digest: &mut MediaDigest,
) -> (Artwork, Option<DynamicImage>, Vec<String>) {
    ingest_artwork_image(
        apic_type,
        image_data,
        image_format_hint,
        media_type_hint,
        image_digest,
    )
    .map_or_else(
        |err| match err {
            ArtworkImageError::UnsupportedFormat(unsupported_format) => {
                let issue = format!("Unsupported image format: {unsupported_format:?}");
                (Artwork::Unsupported, None, vec![issue])
            }
            ArtworkImageError::Image(err) => {
                let issue = format!("Failed to load embedded artwork image: {err}");
                (Artwork::Irregular, None, vec![issue])
            }
            ArtworkImageError::Other(err) => {
                let issue = format!("Failed to load embedded artwork image: {err}");
                (Artwork::Irregular, None, vec![issue])
            }
        },
        |IngestedArtworkImage {
             artwork_image,
             picture,
             recoverable_errors,
         }| {
            let issues = recoverable_errors
                .into_iter()
                .map(|err| {
                    format!(
                        "Recoverable error while loading embedded {apic_type:?} artwork image: \
                         {err}"
                    )
                })
                .collect();
            let artwork = Artwork::Embedded(EmbeddedArtwork {
                image: artwork_image,
            });
            (artwork, Some(picture), issues)
        },
    )
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum EditEmbeddedArtworkImage {
    Replace(ReplaceEmbeddedArtworkImage),
    Remove(RemoveEmbeddedArtworkImage),
}

#[derive(Debug, Clone, Copy)]
pub enum EditOtherEmbeddedArtworkImages {
    Keep,
    Remove,
}

#[derive(Debug, Clone)]
pub struct ReplaceEmbeddedArtworkImage {
    pub artwork_image: ArtworkImage,
    pub image_data: Vec<u8>,
    pub others: EditOtherEmbeddedArtworkImages,
}

impl ReplaceEmbeddedArtworkImage {
    #[must_use]
    pub fn from_ingested_artwork_image(
        ingested_artwork_image: IngestedArtworkImage,
        others: EditOtherEmbeddedArtworkImages,
    ) -> Self {
        let IngestedArtworkImage {
            artwork_image,
            picture,
            recoverable_errors: _,
        } = ingested_artwork_image;
        let image_data = picture.into_bytes();
        Self {
            artwork_image,
            image_data,
            others,
        }
    }

    pub fn reingest_artwork_image(&self, image_digest: &mut MediaDigest) -> Result<ArtworkImage> {
        let ArtworkImage {
            apic_type,
            digest: _,
            media_type,
            size: _,
            color: _,
            thumbnail: _,
        } = &self.artwork_image;
        let image_format_hint = None;
        let ingested_artwork_image = ingest_artwork_image(
            *apic_type,
            &self.image_data,
            image_format_hint,
            Some(media_type.essence_str()),
            image_digest,
        )?;
        let IngestedArtworkImage {
            artwork_image,
            picture: _,
            recoverable_errors: _,
        } = ingested_artwork_image;
        Ok(artwork_image)
    }
}

#[derive(Debug, Clone)]
pub struct RemoveEmbeddedArtworkImage {
    pub apic_type: ApicType,
    pub others: EditOtherEmbeddedArtworkImages,
}
