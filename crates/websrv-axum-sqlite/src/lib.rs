// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{result::Result as StdResult, str::FromStr};

use axum::{
    Json,
    body::Bytes,
    extract::{
        FromRequest, FromRequestParts, Path, Query, Request,
        rejection::{BytesRejection, PathRejection, QueryRejection},
    },
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use serde::{Serialize, de::DeserializeOwned};
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

impl From<PathRejection> for Error {
    fn from(rejection: PathRejection) -> Self {
        Self::Rejected {
            status: rejection.status(),
            message: rejection.body_text(),
        }
    }
}

impl From<QueryRejection> for Error {
    fn from(rejection: QueryRejection) -> Self {
        Self::Rejected {
            status: rejection.status(),
            message: rejection.body_text(),
        }
    }
}

impl From<BytesRejection> for Error {
    fn from(rejection: BytesRejection) -> Self {
        Self::Rejected {
            status: rejection.status(),
            message: rejection.body_text(),
        }
    }
}

pub type Result<T> = StdResult<T, Error>;

pub fn after_blocking_task_finished<T, E1, E2>(res: StdResult<StdResult<T, E1>, E2>) -> Result<T>
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
        let body = Json(ErrorResponseBody {
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

/// A single path parameter that is parsed with [`FromStr`].
///
/// Entity UIDs only implement [`FromStr`] but not [`serde::Deserialize`].
#[derive(Debug)]
pub struct PathParam<T>(pub T);

impl<S, T> FromRequestParts<S> for PathParam<T>
where
    S: Send + Sync,
    T: FromStr,
    T::Err: Into<anyhow::Error>,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
        let Path(param) = Path::<String>::from_request_parts(parts, state).await?;
        param
            .parse()
            .map(Self)
            .map_err(|err: T::Err| Error::BadRequest(err.into()))
    }
}

/// Query parameters with a JSON error response on rejection.
#[derive(Debug)]
pub struct QueryParams<T>(pub T);

impl<S, T> FromRequestParts<S> for QueryParams<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
        let Query(params) = Query::<T>::from_request_parts(parts, state).await?;
        Ok(Self(params))
    }
}

/// A JSON request body.
///
/// In contrast to [`axum::Json`] the `Content-Type` header is not checked.
#[derive(Debug)]
pub struct JsonBody<T>(pub T);

impl<S, T> FromRequest<S> for JsonBody<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = Error;

    async fn from_request(req: Request, state: &S) -> Result<Self> {
        let body = Bytes::from_request(req, state).await?;
        serde_json::from_slice(&body)
            .map(Self)
            .map_err(|err| Error::BadRequest(err.into()))
    }
}
