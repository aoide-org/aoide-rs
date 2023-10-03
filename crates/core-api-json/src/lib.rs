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
// Rustdoc lints
#![warn(rustdoc::broken_intra_doc_links)]
// Clippy lints
#![warn(clippy::pedantic)]
// Additional restrictions
#![warn(clippy::clone_on_ref_ptr)]
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

#[cfg(not(any(feature = "frontend", feature = "backend")))]
compile_error!("at least one of the features \"frontend\" or \"backend\" must be enabled");

// Common imports
mod prelude {
    pub(crate) use aoide_core_api as _inner;
    pub(crate) use aoide_core_api::{PaginationLimit, PaginationOffset};
    pub(crate) use serde::{Deserialize, Serialize};
}
use self::prelude::*;

pub mod collection;
pub mod filtering;
pub mod media;
pub mod playlist;
pub mod sorting;
pub mod tag;
pub mod track;

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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
