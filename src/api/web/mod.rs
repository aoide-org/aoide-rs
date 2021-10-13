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

use std::{convert::Infallible, error::Error as StdError, result::Result as StdResult};

use reject::{InvalidHeader, InvalidQuery, MethodNotAllowed};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use warp::{
    body::BodyDeserializeError,
    http::StatusCode,
    reject::{self, Reject, Rejection},
    Reply,
};

use aoide_media::Error as MediaError;

use aoide_repo::prelude::{Pagination, PaginationLimit, PaginationOffset, RepoError};

use aoide_core_serde::entity::EntityRevision;

use crate::usecases as uc;

use super::*;

mod _repo {
    pub use aoide_repo::prelude::*;
}

pub mod collection;
pub mod media;
pub mod playlist;
pub mod track;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    BadRequest(anyhow::Error),

    #[error("timeout: {reason}")]
    Timeout { reason: String },

    #[error(transparent)]
    TaskScheduling(#[from] tokio::task::JoinError),

    #[error(transparent)]
    Media(#[from] MediaError),

    #[error(transparent)]
    Database(#[from] diesel::result::Error),

    #[error(transparent)]
    DatabaseConnection(#[from] r2d2::Error),

    #[error(transparent)]
    Repository(#[from] RepoError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<uc::Error> for Error {
    fn from(err: uc::Error) -> Self {
        use uc::Error::*;
        match err {
            Input(err) => Self::BadRequest(err),
            Media(err) => Self::Media(err),
            Database(err) => Self::Database(err),
            DatabaseMigration(err) => Self::Other(err.into()), // does not occur for the web API
            DatabaseConnection(err) => Self::DatabaseConnection(err),
            Repository(err) => Self::Repository(err),
            Io(err) => Self::Io(err),
            Other(err) => Self::Other(err),
        }
    }
}

impl From<aoide_usecases::Error> for Error {
    fn from(err: aoide_usecases::Error) -> Self {
        uc::Error::from(err).into()
    }
}

pub type Result<T> = StdResult<T, Error>;

impl Reject for Error {}

pub fn reject_on_error(err: impl Into<Error>) -> Rejection {
    reject::custom(err.into())
}

pub fn reject_from_anyhow(err: impl Into<anyhow::Error>) -> Rejection {
    reject_on_error(err.into())
}

pub fn reject_from_repo_error(err: RepoError) -> Rejection {
    reject_on_error(err)
}

#[derive(Debug)]
struct CustomReject {
    code: StatusCode,
    message: String,
}

impl Reject for CustomReject {}

pub fn reject_status_code_message(code: StatusCode, message: String) -> Rejection {
    warp::reject::custom(CustomReject { code, message })
}

/// An API error serializable to JSON.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorResponseBody {
    code: u16,
    message: String,
}

fn status_code_to_string(code: StatusCode) -> String {
    code.canonical_reason()
        .unwrap_or_else(|| code.as_str())
        .to_string()
}

pub async fn handle_rejection(reject: Rejection) -> StdResult<impl Reply, Infallible> {
    let code;
    let message;

    if reject.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = status_code_to_string(code);
    } else if let Some(CustomReject {
        code: custom_code,
        message: custom_message,
    }) = reject.find()
    {
        code = custom_code.to_owned();
        message = custom_message.to_owned();
    } else if let Some(err) = reject.find::<InvalidHeader>() {
        code = StatusCode::BAD_REQUEST;
        message = err
            .source()
            .map(ToString::to_string)
            .unwrap_or_else(|| err.to_string());
    } else if let Some(err) = reject.find::<InvalidQuery>() {
        code = StatusCode::BAD_REQUEST;
        message = err
            .source()
            .map(ToString::to_string)
            .unwrap_or_else(|| err.to_string());
    } else if let Some(err) = reject.find::<BodyDeserializeError>() {
        code = StatusCode::BAD_REQUEST;
        message = err
            .source()
            .map(ToString::to_string)
            .unwrap_or_else(|| err.to_string());
    } else if let Some(err) = reject.find::<Error>() {
        match err {
            Error::BadRequest(err) => {
                code = StatusCode::BAD_REQUEST;
                message = err.to_string();
            }
            Error::Timeout { .. } => {
                code = StatusCode::SERVICE_UNAVAILABLE;
                message = err.to_string();
            }
            Error::TaskScheduling(err) => {
                code = StatusCode::INTERNAL_SERVER_ERROR;
                message = err.to_string();
            }
            Error::Media(err) => {
                code = StatusCode::INTERNAL_SERVER_ERROR;
                message = err.to_string();
            }
            Error::Database(err) => {
                code = StatusCode::INTERNAL_SERVER_ERROR;
                message = err.to_string();
            }
            Error::DatabaseConnection(err) => {
                code = StatusCode::INTERNAL_SERVER_ERROR;
                message = err.to_string();
            }
            Error::Repository(err) => match err {
                RepoError::NotFound => {
                    code = StatusCode::NOT_FOUND;
                    message = status_code_to_string(code);
                }
                RepoError::Conflict => {
                    code = StatusCode::CONFLICT;
                    message = status_code_to_string(code);
                }
                err => {
                    code = StatusCode::INTERNAL_SERVER_ERROR;
                    message = err.to_string();
                }
            },
            Error::Io(err) => {
                code = StatusCode::INTERNAL_SERVER_ERROR;
                message = err.to_string();
            }
            Error::Other(err) => {
                code = StatusCode::INTERNAL_SERVER_ERROR;
                message = err.to_string();
            }
        }
    } else if let Some(err) = reject.find::<Error>() {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = err.to_string();
    } else if let Some(err) = reject.find::<MethodNotAllowed>() {
        // This must have the least priority, because most rejections
        // contain a MethodNotAllowed element!
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = err.to_string();
    } else {
        tracing::error!("Unhandled rejection {:?}", reject);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = format!("{:?}", reject);
    }

    let json_reply = warp::reply::json(&ErrorResponseBody {
        code: code.as_u16(),
        message,
    });

    Ok(warp::reply::with_status(json_reply, code))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityRevQueryParams {
    pub rev: EntityRevision,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationQueryParams {
    pub limit: Option<PaginationLimit>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<PaginationOffset>,
}

impl From<PaginationQueryParams> for Option<Pagination> {
    fn from(from: PaginationQueryParams) -> Self {
        let PaginationQueryParams { limit, offset } = from;
        if let Some(limit) = limit {
            Some(Pagination { limit, offset })
        } else {
            if let Some(offset) = offset {
                tracing::warn!("Ignoring pagination offset = {} without limit", offset);
            }
            None
        }
    }
}

fn new_request_id() -> Uuid {
    Uuid::new_v4()
}
