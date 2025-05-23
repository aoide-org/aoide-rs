// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{convert::Infallible, error::Error as StdError, result::Result as StdResult};

use serde::Serialize;
use thiserror::Error;
use warp::{
    Reply,
    body::BodyDeserializeError,
    http::StatusCode,
    reject::{self, InvalidHeader, InvalidQuery, MethodNotAllowed, Reject, Rejection},
};

use aoide_backend_webapi_json as api;
use aoide_repo::RepoError;
use aoide_storage_sqlite::{
    self as db,
    connection::pool::{PooledConnection, gatekeeper::Gatekeeper as DatabaseConnectionGatekeeper},
};
use aoide_usecases_sqlite as uc;

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
    Other(anyhow::Error),
}

impl From<api::Error> for Error {
    fn from(err: api::Error) -> Self {
        use api::Error as From;
        match err {
            From::BadRequest(err) => Self::BadRequest(err),
            From::UseCase(err) => err.into(),
            From::DatabaseTransaction(err) => Self::Other(err.into()),
            From::Other(err) => Self::Other(err),
        }
    }
}

impl From<uc::Error> for Error {
    fn from(err: uc::Error) -> Self {
        use uc::Error as From;
        match err {
            From::Input(err) => Self::BadRequest(err),
            From::Io(err) => Self::Other(err.into()),
            From::MediaFile(err) => Self::Other(err.into()),
            From::Storage(err) => err.into(),
            From::Repository(err) => match err {
                RepoError::NotFound => Self::NotFound,
                RepoError::Conflict => Self::Conflict,
                RepoError::Aborted => Self::ServiceUnavailable,
                RepoError::Other(err) => Self::Other(err),
            },
            From::DatabaseMigration(err) | From::Other(err) => Self::Other(err),
        }
    }
}

impl From<db::Error> for Error {
    fn from(err: db::Error) -> Self {
        use db::Error as From;
        match err {
            From::Database(err) => Self::Other(err.into()),
            From::DatabaseConnection(err) => Self::Other(err.into()),
            From::DatabaseConnectionPool(err) => Self::Other(err.into()),
            From::TaskScheduling(err) => Self::Other(err.into()),
            From::TaskTimeout { reason } => Self::Timeout { reason },
            From::Other(err) => Self::Other(err),
        }
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
    gatekeeper: &DatabaseConnectionGatekeeper,
    handler: H,
) -> std::result::Result<T, Rejection>
where
    H: FnOnce(PooledConnection) -> std::result::Result<T, E> + Send + 'static,
    T: Send + 'static,
    E: Into<Error> + Send + 'static,
{
    after_blocking_task_finished(gatekeeper.spawn_blocking_write_task(handler).await)
}

pub async fn spawn_blocking_read_task<H, T, E>(
    gatekeeper: &DatabaseConnectionGatekeeper,
    handler: H,
) -> std::result::Result<T, Rejection>
where
    H: FnOnce(PooledConnection) -> std::result::Result<T, E> + Send + 'static,
    T: Send + 'static,
    E: Into<Error> + Send + 'static,
{
    after_blocking_task_finished(gatekeeper.spawn_blocking_read_task(handler).await)
}

#[derive(Debug)]
#[expect(dead_code, reason = "False positive.")]
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

#[expect(clippy::unused_async)] // async needed for warp filter
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
        code = *custom_code;
        message = custom_message.clone();
    } else if let Some(err) = reject.find::<InvalidHeader>() {
        code = StatusCode::BAD_REQUEST;
        message = err
            .source()
            .map_or_else(|| err.to_string(), ToString::to_string);
    } else if let Some(err) = reject.find::<InvalidQuery>() {
        code = StatusCode::BAD_REQUEST;
        message = err
            .source()
            .map_or_else(|| err.to_string(), ToString::to_string);
    } else if let Some(err) = reject.find::<BodyDeserializeError>() {
        code = StatusCode::BAD_REQUEST;
        message = err
            .source()
            .map_or_else(|| err.to_string(), ToString::to_string);
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
                message = reason.clone();
            }
            Error::Other(err) => {
                code = StatusCode::INTERNAL_SERVER_ERROR;
                message = err.to_string();
            }
        }
    } else if let Some(err) = reject.find::<MethodNotAllowed>() {
        // This must have the least priority, because most rejections
        // contain a MethodNotAllowed element!
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = err.to_string();
    } else {
        log::error!("Unhandled rejection {reject:?}");
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = format!("{reject:?}");
    }

    let json_reply = warp::reply::json(&ErrorResponseBody {
        code: code.as_u16(),
        message,
    });

    Ok(warp::reply::with_status(json_reply, code))
}
