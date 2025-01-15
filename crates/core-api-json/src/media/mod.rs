// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::prelude::*;

pub mod source;
pub mod tracker;

mod _core {
    pub(super) use aoide_core_api::media::*;
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum SyncMode {
    Once,
    Modified,
    ModifiedResync,
    Always,
}

#[cfg(feature = "backend")]
impl From<SyncMode> for _core::SyncMode {
    fn from(from: SyncMode) -> Self {
        use SyncMode as From;
        match from {
            From::Once => Self::Once,
            From::Modified => Self::Modified,
            From::ModifiedResync => Self::ModifiedResync,
            From::Always => Self::Always,
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_core::SyncMode> for SyncMode {
    fn from(from: _core::SyncMode) -> Self {
        use _core::SyncMode as From;
        match from {
            From::Once => Self::Once,
            From::Modified => Self::Modified,
            From::ModifiedResync => Self::ModifiedResync,
            From::Always => Self::Always,
        }
    }
}
