// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub(crate) mod lofty;

#[cfg(feature = "fmt-flac")]
pub(crate) mod flac;

#[cfg(any(feature = "fmt-mp3"))]
pub(crate) mod id3;

#[cfg(feature = "fmt-mp3")]
pub(crate) mod mp3;

#[cfg(feature = "fmt-mp4")]
pub(crate) mod mp4;

#[cfg(any(feature = "fmt-flac"))]
pub(crate) mod vorbis;
