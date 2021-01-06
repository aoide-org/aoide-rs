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

use super::*;

mod _repo {
    pub use aoide_repo::prelude::*;
}

use aoide_repo::prelude::{Pagination, PaginationLimit, PaginationOffset, RepoError, RepoResult};

use aoide_core_serde::entity::EntityRevision;

use reject::MethodNotAllowed;
use serde::{Deserialize, Serialize};

use warp::{
    body::BodyDeserializeError,
    http::StatusCode,
    reject::{self, Reject, Rejection},
    Reply,
};

use std::{convert::Infallible, error::Error as StdError};

///////////////////////////////////////////////////////////////////////

pub mod collections;
pub mod playlists;
pub mod tracks;

#[derive(Debug)]
struct RejectAnyhowError(anyhow::Error);

impl Reject for RejectAnyhowError {}

pub fn reject_from_anyhow(err: anyhow::Error) -> Rejection {
    reject::custom(RejectAnyhowError(err))
}

#[derive(Debug)]
struct RejectConflictError;

impl Reject for RejectConflictError {}

pub fn reject_from_repo_error(err: RepoError) -> Rejection {
    match err {
        RepoError::NotFound => warp::reject::not_found(),
        RepoError::Conflict => warp::reject::custom(RejectConflictError),
        err => reject_from_anyhow(err.into()),
    }
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

pub async fn handle_rejection(reject: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if reject.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = status_code_to_string(code);
    } else if let Some(RejectConflictError) = reject.find() {
        code = StatusCode::CONFLICT;
        message = status_code_to_string(code);
    } else if let Some(CustomReject {
        code: custom_code,
        message: custom_message,
    }) = reject.find()
    {
        code = custom_code.to_owned();
        message = custom_message.to_owned();
    } else if let Some(err) = reject.find::<BodyDeserializeError>() {
        code = StatusCode::BAD_REQUEST;
        message = err
            .source()
            .map(ToString::to_string)
            .unwrap_or_else(|| err.to_string());
    } else if let Some(err) = reject.find::<MethodNotAllowed>() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = err.to_string();
    } else if let Some(RejectAnyhowError(err)) = reject.find() {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = err.to_string();
    } else if let Some(err) = reject.find::<Error>() {
        code = StatusCode::INTERNAL_SERVER_ERROR;
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
                log::warn!("Ignoring pagination offset = {} without limit", offset);
            }
            None
        }
    }
}

/// Predicates for matching strings
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StringPredicate {
    StartsWith(String),
    StartsNotWith(String),
    EndsWith(String),
    EndsNotWith(String),
    Contains(String),
    ContainsNot(String),
    Matches(String),
    MatchesNot(String),
    Equals(String),
    EqualsNot(String),
}

impl From<StringPredicate> for _repo::StringPredicate {
    fn from(from: StringPredicate) -> Self {
        use StringPredicate::*;
        match from {
            StartsWith(s) => Self::StartsWith(s),
            StartsNotWith(s) => Self::StartsNotWith(s),
            EndsWith(s) => Self::EndsWith(s),
            EndsNotWith(s) => Self::EndsNotWith(s),
            Contains(s) => Self::Contains(s),
            ContainsNot(s) => Self::ContainsNot(s),
            Matches(s) => Self::Matches(s),
            MatchesNot(s) => Self::MatchesNot(s),
            Equals(s) => Self::Equals(s),
            EqualsNot(s) => Self::EqualsNot(s),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct WithTokensQueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    with: Option<String>,
}

impl WithTokensQueryParams {
    pub fn try_with_token(&self, with_token: &str) -> bool {
        match self.with {
            Some(ref with) => with.split(',').any(|token| token == with_token),
            None => false,
        }
    }
}
