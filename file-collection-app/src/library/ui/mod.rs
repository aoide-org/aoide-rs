// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! User interface components for rendering with `egui`.

mod artwork;
pub use self::artwork::ARTWORK_THUMBNAIL_IMAGE_SIZE;
use self::artwork::{artwork_thumbnail_image, artwork_thumbnail_image_placeholder};

mod track;
pub use self::track::{TrackListItem, TrackYear};
