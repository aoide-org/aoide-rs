// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::prelude::*;

mod _inner {
    pub(super) use crate::_inner::sorting::*;
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum SortDirection {
    #[serde(rename = "asc")]
    Ascending,

    #[serde(rename = "desc")]
    Descending,
}

#[cfg(feature = "backend")]
impl From<SortDirection> for _inner::SortDirection {
    fn from(from: SortDirection) -> Self {
        use SortDirection::*;
        match from {
            Ascending => Self::Ascending,
            Descending => Self::Descending,
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::SortDirection> for SortDirection {
    fn from(from: _inner::SortDirection) -> Self {
        use _inner::SortDirection::*;
        match from {
            Ascending => Self::Ascending,
            Descending => Self::Descending,
        }
    }
}
