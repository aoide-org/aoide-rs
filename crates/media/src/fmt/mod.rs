// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

#[cfg(feature = "fmt-aiff")]
pub mod aiff;

#[cfg(feature = "fmt-flac")]
pub mod flac;

#[cfg(feature = "fmt-mp3")]
pub mod mp3;

#[cfg(feature = "fmt-mp4")]
pub mod mp4;

#[cfg(feature = "fmt-ogg")]
pub mod ogg;

#[cfg(feature = "fmt-opus")]
pub mod opus;

#[cfg(feature = "fmt-wav")]
pub mod wav;

#[cfg(any(feature = "fmt-mp3", feature = "fmt-aiff", feature = "fmt-wav"))]
pub mod id3;

#[cfg(any(feature = "fmt-flac", feature = "fmt-ogg", feature = "fmt-opus"))]
pub mod vorbis;

const ENCODER_FIELD_SEPARATOR: &str = "|";
