// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use std::result::Result as StdResult;

use diesel::{prelude::SqliteConnection, Connection as _};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;

use aoide_core::collection::EntityUid as CollectionUid;

use aoide_repo::prelude::{Pagination, PaginationLimit, PaginationOffset};

use aoide_core_json::entity::EntityRevision;

use aoide_usecases_sqlite as uc;

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
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct EntityRevQueryParams {
    pub rev: EntityRevision,
}

fn new_request_id() -> Uuid {
    Uuid::new_v4()
}
