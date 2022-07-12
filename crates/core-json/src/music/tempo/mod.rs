// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::prelude::*;

mod _core {
    pub(super) use aoide_core::music::tempo::*;
}

///////////////////////////////////////////////////////////////////////
// Tempo
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct TempoBpm(_core::Bpm);

impl From<_core::TempoBpm> for TempoBpm {
    fn from(from: _core::TempoBpm) -> Self {
        Self(from.to_raw())
    }
}

impl From<TempoBpm> for _core::TempoBpm {
    fn from(from: TempoBpm) -> Self {
        Self::from_raw(from.0)
    }
}
