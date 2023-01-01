// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

// rustflags
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
// rustflags (clippy)
#![warn(clippy::all)]
#![warn(clippy::explicit_deref_methods)]
#![warn(clippy::explicit_into_iter_loop)]
#![warn(clippy::explicit_iter_loop)]
#![warn(clippy::must_use_candidate)]
// rustdocflags
#![warn(rustdoc::broken_intra_doc_links)]
#![cfg_attr(not(test), deny(clippy::panic_in_result_fn))]
#![cfg_attr(not(debug_assertions), deny(clippy::used_underscore_binding))]

pub mod collection;
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
