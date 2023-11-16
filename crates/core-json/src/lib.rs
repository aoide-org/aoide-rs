// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

// TODO: Remove temporary workaround.
// <https://github.com/rust-lang/rust-clippy/issues/11237>
#![allow(clippy::wildcard_imports)]

pub mod prelude {
    pub(crate) use serde::{Deserialize, Serialize};
    pub(crate) use serde_repr::*;

    pub(crate) use crate::util::{clock::*, color::*};
}

pub mod audio;
pub mod collection;
pub mod entity;
pub mod media;
pub mod music;
pub mod playlist;
pub mod tag;
pub mod track;
pub mod util;
