// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod batch;
pub mod collection;
pub mod media;
pub mod playlist;
pub mod storage;
pub mod track;

pub type Error = aoide_usecases_sqlite::Error;
pub type Result<T> = std::result::Result<T, Error>;

pub mod prelude {
    pub use aoide_core::CollectionUid;

    pub use super::{Error, Result};
}
