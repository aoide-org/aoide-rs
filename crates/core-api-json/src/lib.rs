// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#![deny(missing_debug_implementations)]
#![deny(clippy::clone_on_ref_ptr)]
#![warn(rust_2018_idioms)]

// Common imports
mod prelude {
    pub use aoide_core_api::{PaginationLimit, PaginationOffset};
    pub use serde::{Deserialize, Serialize};

    pub(crate) use aoide_core_api as _inner;
}

use crate::prelude::*;

pub mod collection;
pub mod filtering;
pub mod media;
pub mod sorting;
pub mod tag;
pub mod track;

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
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
