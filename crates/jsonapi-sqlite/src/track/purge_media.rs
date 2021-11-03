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

use aoide_core_ext_serde::filtering::StringPredicate;
use aoide_usecases::track::purge::PurgeByMediaSourcePathPredicatesSummary;
use aoide_usecases_sqlite::SqlitePooledConnection;

use super::*;

mod uc {
    pub use aoide_repo::prelude::StringPredicate;
    pub use aoide_usecases_sqlite::track::purge::*;
}

pub type RequestBody = Vec<StringPredicate>;

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ResponseBody {
    purged_media_sources: u64,
    purged_tracks: u64,
}

impl From<PurgeByMediaSourcePathPredicatesSummary> for ResponseBody {
    fn from(from: PurgeByMediaSourcePathPredicatesSummary) -> Self {
        let PurgeByMediaSourcePathPredicatesSummary {
            purged_media_sources,
            purged_tracks,
        } = from;
        Self {
            purged_media_sources: purged_media_sources as u64,
            purged_tracks: purged_tracks as u64,
        }
    }
}

pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &_core::EntityUid,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    uc::purge_by_media_source_path_predicates(
        &pooled_connection,
        collection_uid,
        request_body.into_iter().map(Into::into).collect(),
    )
    .map(Into::into)
    .map_err(Into::into)
}