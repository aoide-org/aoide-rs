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
#![deny(rust_2018_idioms)]

pub mod collection;
pub mod filtering;
pub mod media;
pub mod sorting;
pub mod tag;
pub mod track;

pub type PaginationOffset = u64;

pub type PaginationLimit = u64;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Pagination {
    pub limit: Option<PaginationLimit>,
    pub offset: Option<PaginationOffset>,
}

impl Pagination {
    pub const fn has_offset(&self) -> bool {
        self.offset.is_some()
    }

    pub const fn is_limited(&self) -> bool {
        self.limit.is_some()
    }

    pub const fn is_paginated(&self) -> bool {
        self.has_offset() || self.is_limited()
    }

    /// Mandatory offset
    ///
    /// Returns the offset if specified or 0 otherwise.
    pub fn mandatory_offset(&self) -> PaginationOffset {
        self.offset.unwrap_or(0)
    }

    /// Mandatory limit
    ///
    /// Returns the limit if specified or the maximum value otherwise.
    pub fn mandatory_limit(&self) -> PaginationLimit {
        self.limit.unwrap_or(PaginationLimit::MAX)
    }
}
