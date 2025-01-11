// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

#[cfg(not(any(feature = "frontend", feature = "backend")))]
compile_error!("at least one of the features \"frontend\" or \"backend\" must be enabled");

// Common imports
mod prelude {
    pub(crate) use aoide_core_api as _inner;
    pub(crate) use aoide_core_api::{PaginationLimit, PaginationOffset};
    pub(crate) use serde::{Deserialize, Serialize};
}
use self::prelude::*;

mod sorting;
pub use self::sorting::SortDirection;

pub mod collection;
pub mod filtering;
pub mod media;
pub mod playlist;
pub mod tag;
pub mod track;

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Pagination {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<PaginationLimit>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<PaginationOffset>,
}

#[cfg(feature = "frontend")]
impl From<_inner::Pagination> for Pagination {
    fn from(from: _inner::Pagination) -> Self {
        let _inner::Pagination { limit, offset } = from;
        Self { limit, offset }
    }
}

#[cfg(feature = "backend")]
impl From<Pagination> for _inner::Pagination {
    fn from(from: Pagination) -> Self {
        let Pagination { limit, offset } = from;
        Self { limit, offset }
    }
}
