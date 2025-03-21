// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::result::Result as StdResult;

use diesel::Connection as _;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use aoide_core::CollectionUid;
use aoide_core_api::{Pagination, PaginationLimit, PaginationOffset};
use aoide_core_json::entity::EntityRevision;
use aoide_repo_sqlite::DbConnection;
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
    Other(anyhow::Error),
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
