// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Integration of the JSON web API into [axum].
//!
//! Provides an [`Error`](enum@Error) type that is converted into a JSON error response
//! and the extractors [`Json`], [`Query`], and [`Path`] that reject requests
//! with this error type.

use std::result::Result as StdResult;

use axum::{
    extract::{
        FromRequest, FromRequestParts,
        rejection::{JsonRejection, PathRejection, QueryRejection},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

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

    /// Rejected by the web framework, e.g. while extracting the request.
    #[error("{message}")]
    Rejected { status: StatusCode, message: String },

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

macro_rules! impl_from_rejection {
    ($rejection:ty) => {
        impl From<$rejection> for Error {
            fn from(rejection: $rejection) -> Self {
                Self::Rejected {
                    status: rejection.status(),
                    message: rejection.body_text(),
                }
            }
        }
    };
}

impl_from_rejection!(JsonRejection);
impl_from_rejection!(QueryRejection);
impl_from_rejection!(PathRejection);

pub type Result<T> = StdResult<T, Error>;

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

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (code, message) = match self {
            Self::BadRequest(err) => (StatusCode::BAD_REQUEST, err.to_string()),
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                status_code_to_string(StatusCode::NOT_FOUND),
            ),
            Self::Conflict => (
                StatusCode::CONFLICT,
                status_code_to_string(StatusCode::CONFLICT),
            ),
            Self::ServiceUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                status_code_to_string(StatusCode::SERVICE_UNAVAILABLE),
            ),
            Self::Timeout { reason } => (StatusCode::REQUEST_TIMEOUT, reason),
            Self::Rejected { status, message } => (status, message),
            Self::Other(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        };
        let body = axum::Json(ErrorResponseBody {
            code: code.as_u16(),
            message,
        });
        (code, body).into_response()
    }
}

/// Fallback handler for unmatched routes.
#[expect(clippy::unused_async)] // async needed for axum handler
pub async fn handle_not_found() -> Error {
    Error::NotFound
}

/// Fallback handler for matched routes with an unsupported method.
#[expect(clippy::unused_async)] // async needed for axum handler
pub async fn handle_method_not_allowed() -> Error {
    let status = StatusCode::METHOD_NOT_ALLOWED;
    Error::Rejected {
        status,
        message: status_code_to_string(status),
    }
}

/// JSON request or response body.
///
/// Like [`axum::Json`], but rejects requests with an [`Error`](enum@Error).
#[derive(Debug, FromRequest)]
#[from_request(via(axum::Json), rejection(Error))]
pub struct Json<T>(pub T);

impl<T> IntoResponse for Json<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        let Self(value) = self;
        axum::Json(value).into_response()
    }
}

/// Query parameters.
///
/// Like [`axum::extract::Query`], but rejects requests with an [`Error`](enum@Error).
#[derive(Debug, FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(Error))]
pub struct Query<T>(pub T);

/// Path parameters.
///
/// Like [`axum::extract::Path`], but rejects requests with an [`Error`](enum@Error).
#[derive(Debug, FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(Error))]
pub struct Path<T>(pub T);

fn after_blocking_task_finished<T, E1, E2>(res: StdResult<StdResult<T, E1>, E2>) -> Result<T>
where
    E1: Into<Error>,
    E2: Into<Error>,
{
    res.map_err(Into::into)
        .and_then(|res| res.map_err(Into::into))
}

pub async fn spawn_blocking_write_task<H, T, E>(
    gatekeeper: &DatabaseConnectionGatekeeper,
    handler: H,
) -> Result<T>
where
    H: FnOnce(PooledConnection) -> StdResult<T, E> + Send + 'static,
    T: Send + 'static,
    E: Into<Error> + Send + 'static,
{
    after_blocking_task_finished(gatekeeper.spawn_blocking_write_task(handler).await)
}

pub async fn spawn_blocking_read_task<H, T, E>(
    gatekeeper: &DatabaseConnectionGatekeeper,
    handler: H,
) -> Result<T>
where
    H: FnOnce(PooledConnection) -> StdResult<T, E> + Send + 'static,
    T: Send + 'static,
    E: Into<Error> + Send + 'static,
{
    after_blocking_task_finished(gatekeeper.spawn_blocking_read_task(handler).await)
}
