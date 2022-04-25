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

#![cfg_attr(not(debug_assertions), deny(warnings))]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(clippy::all)]
#![deny(clippy::explicit_deref_methods)]
#![deny(clippy::explicit_into_iter_loop)]
#![deny(clippy::explicit_iter_loop)]
#![deny(clippy::must_use_candidate)]
#![cfg_attr(not(test), deny(clippy::panic_in_result_fn))]
#![cfg_attr(not(debug_assertions), deny(clippy::used_underscore_binding))]

use std::{
    convert::Infallible,
    error::Error as StdError,
    result::Result as StdResult,
    sync::{atomic::AtomicBool, Arc},
};

use db::connection::pool::{
    gatekeeper::Gatekeeper as DatabaseConnectionGatekeeper, PooledConnection,
};
use serde::Serialize;
use thiserror::Error;
use tokio::task::JoinError;
use warp::{
    body::BodyDeserializeError,
    http::StatusCode,
    reject::{self, InvalidHeader, InvalidQuery, MethodNotAllowed, Reject, Rejection},
    Reply,
};

use aoide_repo::prelude::RepoError;

use aoide_storage_sqlite as db;

use aoide_usecases_sqlite as uc;

use aoide_backend_webapi_json as api;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    BadRequest(anyhow::Error),

    #[error("not found")]
    NotFound,

    #[error("conflict")]
    Conflict,

    #[error("service unavailable")]
    ServiceUnavailable,

    #[error("timeout: {reason}")]
    Timeout { reason: String },

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<api::Error> for Error {
    fn from(err: api::Error) -> Self {
        use api::Error::*;
        match err {
            BadRequest(err) => Self::BadRequest(err),
            UseCase(err) => err.into(),
            DatabaseTransaction(err) => Self::Other(err.into()),
            Other(err) => Self::Other(err),
        }
    }
}

impl From<uc::Error> for Error {
    fn from(err: uc::Error) -> Self {
        use uc::Error::*;
        match err {
            Input(err) => Self::BadRequest(err),
            Io(err) => Self::Other(err.into()),
            Media(err) => Self::Other(err.into()),
            Storage(err) => err.into(),
            DatabaseMigration(err) => Self::Other(err.into()),
            Repository(err) => match err {
                RepoError::NotFound => Self::NotFound,
                RepoError::Conflict => Self::Conflict,
                RepoError::Aborted => Self::ServiceUnavailable,
                RepoError::Other(err) => Self::Other(err),
            },
            Other(err) => Self::Other(err),
        }
    }
}

impl From<db::Error> for Error {
    fn from(err: db::Error) -> Self {
        use db::Error::*;
        match err {
            Database(err) => Self::Other(err.into()),
            DatabaseConnection(err) => Self::Other(err.into()),
            DatabaseConnectionPool(err) => Self::Other(err.into()),
            TaskScheduling(err) => Self::Other(err.into()),
            TaskTimeout { reason } => Self::Timeout { reason },
            Other(err) => Self::Other(err),
        }
    }
}

impl From<JoinError> for Error {
    fn from(err: JoinError) -> Self {
        Self::Other(err.into())
    }
}

pub type Result<T> = StdResult<T, Error>;

impl Reject for Error {}

fn reject_on_error(err: impl Into<Error>) -> Rejection {
    reject::custom(err.into())
}

pub fn after_blocking_task_finished<T, E1, E2>(
    res: std::result::Result<std::result::Result<T, E1>, E2>,
) -> std::result::Result<T, Rejection>
where
    E1: Into<Error>,
    E2: Into<Error>,
{
    res.map_err(reject_on_error)
        .and_then(|res| res.map_err(reject_on_error))
}

pub async fn spawn_blocking_write_task<H, T, E>(
    gatekeper: &DatabaseConnectionGatekeeper,
    handler: H,
) -> std::result::Result<T, Rejection>
where
    H: FnOnce(PooledConnection, Arc<AtomicBool>) -> std::result::Result<T, E> + Send + 'static,
    T: Send + 'static,
    E: Into<Error> + Send + 'static,
{
    after_blocking_task_finished(gatekeper.spawn_blocking_write_task(handler).await)
}

pub async fn spawn_blocking_read_task<H, T, E>(
    gatekeper: &DatabaseConnectionGatekeeper,
    handler: H,
) -> std::result::Result<T, Rejection>
where
    H: FnOnce(PooledConnection, Arc<AtomicBool>) -> std::result::Result<T, E> + Send + 'static,
    T: Send + 'static,
    E: Into<Error> + Send + 'static,
{
    after_blocking_task_finished(gatekeper.spawn_blocking_read_task(handler).await)
}

#[derive(Debug)]
struct CustomReject {
    code: StatusCode,
    message: String,
}

impl Reject for CustomReject {}

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
            Error::NotFound => {
                code = StatusCode::NOT_FOUND;
                message = status_code_to_string(code);
            }
            Error::Conflict => {
                code = StatusCode::CONFLICT;
                message = status_code_to_string(code);
            }
            Error::ServiceUnavailable => {
                code = StatusCode::SERVICE_UNAVAILABLE;
                message = status_code_to_string(code);
            }
            Error::Timeout { reason } => {
                code = StatusCode::REQUEST_TIMEOUT;
                message = reason.to_owned();
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
        log::error!("Unhandled rejection {:?}", reject);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = format!("{:?}", reject);
    }

    let json_reply = warp::reply::json(&ErrorResponseBody {
        code: code.as_u16(),
        message,
    });

    Ok(warp::reply::with_status(json_reply, code))
}
