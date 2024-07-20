// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use egui::{Color32, ColorImage};

use aoide::{
    media::artwork::{Artwork, ArtworkImage, EmbeddedArtwork},
    util::color::RgbColor,
};

const ARTWORK_THUMBNAIL_SIZE: usize = 4;
const ARTWORK_THUMBNAIL_BORDER_SIZE: usize = 1;

pub const ARTWORK_THUMBNAIL_IMAGE_SIZE: usize =
    ARTWORK_THUMBNAIL_SIZE + 2 * ARTWORK_THUMBNAIL_BORDER_SIZE;

#[must_use]
const fn solid_rgb_color(color: RgbColor) -> Color32 {
    Color32::from_rgb(color.red(), color.green(), color.blue())
}

#[must_use]
fn artwork_thumbnail_image_with_solid_color(color: Color32) -> ColorImage {
    ColorImage {
        size: [ARTWORK_THUMBNAIL_IMAGE_SIZE, ARTWORK_THUMBNAIL_IMAGE_SIZE],
        pixels: [color; ARTWORK_THUMBNAIL_IMAGE_SIZE * ARTWORK_THUMBNAIL_IMAGE_SIZE].to_vec(),
    }
}

#[must_use]
pub(super) fn artwork_thumbnail_image_placeholder() -> ColorImage {
    artwork_thumbnail_image_with_solid_color(Color32::TRANSPARENT)
}

#[must_use]
fn artwork_thumbnail_image_from_rgb_pixels(
    thumbnail: &[u8; ARTWORK_THUMBNAIL_SIZE * ARTWORK_THUMBNAIL_SIZE * 3],
    border_color: Color32,
) -> ColorImage {
    let pixels = thumbnail
        .chunks_exact(3)
        .map(|rgb| Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
    artwork_thumbnail_image_from_pixels(pixels, border_color)
}

#[must_use]
#[allow(clippy::similar_names)]
fn artwork_thumbnail_image_from_pixels(
    pixels: impl IntoIterator<Item = Color32>,
    border_color: Color32,
) -> ColorImage {
    // TODO: Avoid temporary allocation.
    let pixels = pixels.into_iter().collect::<Vec<_>>();
    let mut pixels_rows = pixels.chunks_exact(4);
    let pixels_row0 = pixels_rows.next().unwrap();
    let pixels_row1 = pixels_rows.next().unwrap();
    let pixels_row2 = pixels_rows.next().unwrap();
    let pixels_row3 = pixels_rows.next().unwrap();
    let pixels = std::iter::repeat(border_color)
        .take(
            ARTWORK_THUMBNAIL_IMAGE_SIZE * ARTWORK_THUMBNAIL_BORDER_SIZE
                + ARTWORK_THUMBNAIL_BORDER_SIZE,
        )
        .chain(pixels_row0.iter().copied())
        .chain(std::iter::repeat(border_color).take(ARTWORK_THUMBNAIL_BORDER_SIZE * 2))
        .chain(pixels_row1.iter().copied())
        .chain(std::iter::repeat(border_color).take(ARTWORK_THUMBNAIL_BORDER_SIZE * 2))
        .chain(pixels_row2.iter().copied())
        .chain(std::iter::repeat(border_color).take(ARTWORK_THUMBNAIL_BORDER_SIZE * 2))
        .chain(pixels_row3.iter().copied())
        .chain(std::iter::repeat(border_color).take(
            ARTWORK_THUMBNAIL_IMAGE_SIZE * ARTWORK_THUMBNAIL_BORDER_SIZE
                + ARTWORK_THUMBNAIL_BORDER_SIZE,
        ))
        .collect::<Vec<_>>();
    debug_assert_eq!(
        pixels.len(),
        ARTWORK_THUMBNAIL_IMAGE_SIZE * ARTWORK_THUMBNAIL_IMAGE_SIZE
    );
    ColorImage {
        size: [ARTWORK_THUMBNAIL_IMAGE_SIZE, ARTWORK_THUMBNAIL_IMAGE_SIZE],
        pixels,
    }
}

#[must_use]
#[allow(clippy::similar_names)]
pub(super) fn artwork_thumbnail_image(
    artwork: &Artwork,
    default_color: Option<RgbColor>,
) -> Option<ColorImage> {
    let Artwork::Embedded(EmbeddedArtwork {
        image: ArtworkImage {
            thumbnail, color, ..
        },
        ..
    }) = artwork
    else {
        return None;
    };
    let color = color.or(default_color);
    let Some(thumbnail) = thumbnail else {
        return color
            .map(solid_rgb_color)
            .map(artwork_thumbnail_image_with_solid_color);
    };
    let color = color.map(solid_rgb_color);
    let border_color = color.unwrap_or(Color32::TRANSPARENT);
    Some(artwork_thumbnail_image_from_rgb_pixels(
        thumbnail,
        border_color,
    ))
}
