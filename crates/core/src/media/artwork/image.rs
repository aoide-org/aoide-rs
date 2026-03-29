// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{convert::TryFrom as _, rc::Rc};

use anyhow::anyhow;
use derive_more::{Display, Error, From};
use image::{
    DynamicImage, GenericImageView, ImageError, ImageFormat, guess_format, load_from_memory,
    load_from_memory_with_format,
};
use mime::{IMAGE_BMP, IMAGE_GIF, IMAGE_JPEG, IMAGE_PNG, IMAGE_STAR, Mime};
use okmain::InputImage;

use crate::{media::MediaDigest, util::color::RgbColor};

use super::{
    ApicType, Artwork, ArtworkImage, EmbeddedArtwork, ImageDimension, ImageSize, THUMBNAIL_HEIGHT,
    THUMBNAIL_WIDTH, Thumbnail4x4Rgb8,
};

#[derive(Debug, Display, Error, From)]
pub enum ArtworkImageError {
    #[display("unsupported format {_0:?}")]
    UnsupportedFormat(#[error(not(source))] ImageFormat),

    #[from]
    Image(ImageError),

    #[from]
    Other(anyhow::Error),
}

#[derive(Debug)]
pub struct LoadedArtworkPicture {
    pub media_type: Mime,
    pub picture: DynamicImage,
    pub recoverable_errors: Vec<anyhow::Error>,
}

pub type LoadArtworkPictureResult = std::result::Result<LoadedArtworkPicture, ArtworkImageError>;

#[expect(clippy::missing_panics_doc)] // Never panics
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
                        "failed to parse MIME type from '{media_type_hint}': {err}"
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

const MAIN_COLORS_IMAGE_MAX_SIZE: u16 = 128;

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
    let width = ImageDimension::try_from(width).map_err(|_| {
        ArtworkImageError::Other(anyhow!("unsupported image size: {width}x{height}"))
    })?;
    let height = ImageDimension::try_from(height).map_err(|_| {
        ArtworkImageError::Other(anyhow!("unsupported image size: {width}x{height}"))
    })?;
    let data_size = image_data.len() as u64;
    let image_size = ImageSize { width, height };
    let digest = image_digest
        .digest_content_data(image_data)
        .finalize_reset();
    let picture = Rc::new(picture);
    let scaled_picture =
        if width > MAIN_COLORS_IMAGE_MAX_SIZE || height > MAIN_COLORS_IMAGE_MAX_SIZE {
            Rc::new(picture.resize(
                MAIN_COLORS_IMAGE_MAX_SIZE.into(),
                MAIN_COLORS_IMAGE_MAX_SIZE.into(),
                image::imageops::FilterType::Lanczos3,
            ))
        } else {
            Rc::clone(&picture)
        };
    let color = {
        let rgb8_image_buffer;
        let rgb8_image_buffer = if let Some(rgb8_image_buffer) = scaled_picture.as_rgb8() {
            rgb8_image_buffer
        } else {
            // Format conversion requires a temporary allocation.
            rgb8_image_buffer = scaled_picture.to_rgb8();
            &rgb8_image_buffer
        };
        let input_image = InputImage::try_from(rgb8_image_buffer).map_err(|err| {
            ArtworkImageError::Other(
                anyhow::Error::from(err).context("convert image to okmain input"),
            )
        })?;
        let main_colors = okmain::colors(input_image);
        main_colors
            .first()
            .map(|rgb| RgbColor::rgb(rgb.r, rgb.g, rgb.b))
    };
    let thumbnail_picture = scaled_picture.resize_exact(
        THUMBNAIL_WIDTH.into(),
        THUMBNAIL_HEIGHT.into(),
        image::imageops::FilterType::Lanczos3,
    );
    let thumbnail = Thumbnail4x4Rgb8::try_from(thumbnail_picture.to_rgb8().into_raw()).ok();
    debug_assert!(thumbnail.is_some());
    let artwork_image = ArtworkImage {
        media_type,
        apic_type,
        data_size,
        image_size: Some(image_size),
        digest,
        color,
        thumbnail,
    };
    drop(scaled_picture);
    debug_assert_eq!(Rc::strong_count(&picture), 1);
    let picture = Rc::into_inner(picture).expect("infallible");
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
#[expect(clippy::large_enum_variant)]
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

    pub fn reingest_artwork_image(
        &self,
        image_digest: &mut MediaDigest,
    ) -> Result<ArtworkImage, ArtworkImageError> {
        let ArtworkImage {
            media_type,
            apic_type,
            data_size: _,
            digest: _,
            image_size: _,
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
