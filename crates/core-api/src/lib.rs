// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

// TODO: Remove temporary workaround.
// <https://github.com/rust-lang/rust-clippy/issues/11237>
#![allow(clippy::wildcard_imports)]

pub mod collection;
pub use collection::Summary as CollectionSummary;

pub mod filtering;
pub mod media;
pub mod playlist;
pub mod sorting;
pub mod tag;
pub mod track;

pub type PaginationOffset = u64;

pub type PaginationLimit = u64;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pagination {
    pub limit: Option<PaginationLimit>,
    pub offset: Option<PaginationOffset>,
}

impl Pagination {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            limit: None,
            offset: None,
        }
    }

    #[must_use]
    pub const fn has_offset(&self) -> bool {
        self.offset.is_some()
    }

    #[must_use]
    pub const fn is_limited(&self) -> bool {
        self.limit.is_some()
    }

    #[must_use]
    pub const fn is_paginated(&self) -> bool {
        self.has_offset() || self.is_limited()
    }

    /// Mandatory offset
    ///
    /// Returns the offset if specified or 0 otherwise.
    #[must_use]
    pub fn mandatory_offset(&self) -> PaginationOffset {
        self.offset.unwrap_or(0)
    }

    /// Mandatory limit
    ///
    /// Returns the limit if specified or the maximum value otherwise.
    #[must_use]
    pub fn mandatory_limit(&self) -> PaginationLimit {
        self.limit.unwrap_or(PaginationLimit::MAX)
    }
}
