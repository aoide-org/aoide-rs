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

use aoide_repo::prelude::{Pagination, PaginationLimit, PaginationOffset, RepoResult};

use aoide_core_serde::entity::EntityRevision;

use serde::{Deserialize, Serialize};

use warp::reject::{self, Reject, Rejection};

use std::{error::Error as StdError, fmt};

///////////////////////////////////////////////////////////////////////

pub mod collections;
pub mod playlists;
pub mod tracks;

#[derive(Debug)]
struct RejectAnyhowError(anyhow::Error);

impl fmt::Display for RejectAnyhowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Reject for RejectAnyhowError {}

impl StdError for RejectAnyhowError {}

impl From<anyhow::Error> for RejectAnyhowError {
    fn from(err: anyhow::Error) -> Self {
        RejectAnyhowError(err)
    }
}

pub fn reject_from_anyhow(err: anyhow::Error) -> Rejection {
    reject::custom(RejectAnyhowError(err))
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
