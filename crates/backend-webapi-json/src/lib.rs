// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(rustdoc::broken_intra_doc_links)]
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

use std::result::Result as StdResult;

use aoide_core::CollectionUid;
use aoide_core_json::entity::EntityRevision;
use aoide_repo::prelude::{Pagination, PaginationLimit, PaginationOffset};
use aoide_repo_sqlite::DbConnection;
use aoide_usecases_sqlite as uc;
use diesel::Connection as _;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub mod collection;
pub mod media;
pub mod playlist;
pub mod track;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    BadRequest(anyhow::Error),

    #[error(transparent)]
    UseCase(uc::Error),

    #[error(transparent)]
    DatabaseTransaction(#[from] diesel::result::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<uc::Error> for Error {
    fn from(err: uc::Error) -> Self {
        Self::UseCase(err)
    }
}

impl From<aoide_usecases::Error> for Error {
    fn from(err: aoide_usecases::Error) -> Self {
        uc::Error::from(err).into()
    }
}

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct EntityRevQueryParams {
    pub rev: EntityRevision,
}

fn new_request_id() -> Uuid {
    Uuid::new_v4()
}
