// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

// Opt-in for allowed-by-default lints (in alphabetical order)
// See also: <https://doc.rust-lang.org/rustc/lints>
#![warn(future_incompatible)]
#![warn(let_underscore)]
#![warn(missing_debug_implementations)]
//#![warn(missing_docs)] // TODO
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(unused)]
// Clippy lints
#![warn(clippy::pedantic)]
// Additional restrictions
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::missing_const_for_fn)]
#![warn(clippy::self_named_module_files)]
// Repetitions of module/type names occur frequently when using many
// modules for keeping the size of the source files handy. Often
// types have the same name as their parent module.
#![allow(clippy::module_name_repetitions)]
// Repeating the type name in `..Default::default()` expressions
// is not needed since the context is obvious.
#![allow(clippy::default_trait_access)]
// Using wildcard imports consciously is acceptable.
#![allow(clippy::wildcard_imports)]
// Importing all enum variants into a narrow, local scope is acceptable.
#![allow(clippy::enum_glob_use)]
// TODO: Add missing docs
#![allow(clippy::missing_errors_doc)]

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
