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

use serde::Serialize;
use thiserror::Error;
use warp::{
    body::BodyDeserializeError,
    http::StatusCode,
    reject::{self, InvalidHeader, InvalidQuery, MethodNotAllowed, Reject, Rejection},
    Reply,
};

use aoide_repo::prelude::RepoError;

use aoide_jsonapi_sqlite as api;

use aoide_usecases_sqlite as uc;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    BadRequest(anyhow::Error),

    #[error(transparent)]
    UseCase(#[from] uc::Error),

    #[error("timeout: {reason}")]
    Timeout { reason: String },

    #[error(transparent)]
    TaskScheduling(#[from] tokio::task::JoinError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<api::Error> for Error {
    fn from(err: api::Error) -> Self {
        match err {
            api::Error::BadRequest(err) => Self::BadRequest(err),
            api::Error::UseCase(err) => Self::UseCase(err),
            api::Error::Other(err) => Self::Other(err),
        }
    }
}

pub type Result<T> = StdResult<T, Error>;

impl Reject for Error {}

pub fn reject_on_error(err: impl Into<Error>) -> Rejection {
    reject::custom(err.into())
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
            Error::UseCase(err) => match err {
                uc::Error::Input(err) => {
                    code = StatusCode::BAD_REQUEST;
                    message = err.to_string();
                }
                uc::Error::Media(err) => {
                    // Using StatusCode::UNSUPPORTED_MEDIA_TYPE for some
                    // of the error variants would be wrong, because they
                    // don't affect the media type of the request!
                    code = StatusCode::INTERNAL_SERVER_ERROR;
                    message = err.to_string();
                }
                uc::Error::Database(err) => {
                    code = StatusCode::INTERNAL_SERVER_ERROR;
                    message = err.to_string();
                }
                uc::Error::DatabaseMigration(err) => {
                    code = StatusCode::INTERNAL_SERVER_ERROR;
                    message = err.to_string();
                }
                uc::Error::DatabaseConnection(err) => {
                    code = StatusCode::INTERNAL_SERVER_ERROR;
                    message = err.to_string();
                }
                uc::Error::Repository(err) => match err {
                    RepoError::NotFound => {
                        code = StatusCode::NOT_FOUND;
                        message = status_code_to_string(code);
                    }
                    RepoError::Conflict => {
                        code = StatusCode::CONFLICT;
                        message = status_code_to_string(code);
                    }
                    RepoError::Aborted => {
                        code = StatusCode::SERVICE_UNAVAILABLE;
                        message = status_code_to_string(code);
                    }
                    RepoError::Other(err) => {
                        code = StatusCode::INTERNAL_SERVER_ERROR;
                        message = err.to_string();
                    }
                },
                uc::Error::Io(err) => {
                    code = StatusCode::INTERNAL_SERVER_ERROR;
                    message = err.to_string();
                }
                uc::Error::Other(err) => {
                    code = StatusCode::INTERNAL_SERVER_ERROR;
                    message = err.to_string();
                }
            },
            Error::Timeout { reason } => {
                code = StatusCode::REQUEST_TIMEOUT;
                message = reason.to_owned();
            }
            Error::TaskScheduling(err) => {
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
