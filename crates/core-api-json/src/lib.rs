// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

#[cfg(not(any(feature = "frontend", feature = "backend")))]
compile_error!("at least one of the features \"frontend\" or \"backend\" must be enabled");

// Common imports
mod prelude {
    pub(crate) use aoide_core_api::{PaginationLimit, PaginationOffset};

    pub(crate) use serde::{Deserialize, Serialize};

    pub(crate) use aoide_core_api as _inner;

    #[cfg(feature = "schemars")]
    pub(crate) use schemars::JsonSchema;
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
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
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
