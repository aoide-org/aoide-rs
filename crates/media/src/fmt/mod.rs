// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub(crate) mod lofty;

#[cfg(feature = "fmt-flac")]
pub(crate) mod flac;

#[cfg(any(feature = "fmt-flac"))]
pub(crate) mod vorbis;
