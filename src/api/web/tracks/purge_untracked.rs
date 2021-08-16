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

use url::Url;

use aoide_core::util::url::BaseUrl;

pub use aoide_core_serde::{
    entity::EntityHeader,
    track::{Entity, Track},
    usecases::filtering::StringPredicate,
};

mod uc {
    pub use crate::usecases::tracks::purge::*;
    pub use aoide_repo::prelude::StringPredicate;
}

use super::*;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    root_url: Option<Url>,

    #[serde(skip_serializing_if = "Option::is_none")]
    untrack_orphaned_directories: Option<bool>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ResponseBody {
    untracked_directories: u64,
    purged_tracks: u64,
}

impl From<uc::PurgeByUntrackedMediaSourcesSummary> for ResponseBody {
    fn from(from: uc::PurgeByUntrackedMediaSourcesSummary) -> Self {
        let uc::PurgeByUntrackedMediaSourcesSummary {
            untracked_directories,
            purged_tracks,
        } = from;
        Self {
            untracked_directories: untracked_directories as u64,
            purged_tracks: purged_tracks as u64,
        }
    }
}

pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &_core::EntityUid,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let RequestBody {
        root_url,
        untrack_orphaned_directories,
    } = request_body;
    let root_url = root_url
        .map(BaseUrl::try_autocomplete_from)
        .transpose()
        .map_err(anyhow::Error::from)
        .map_err(Error::BadRequest)?;
    uc::purge_by_untracked_media_sources(
        &pooled_connection,
        collection_uid,
        root_url.as_ref(),
        untrack_orphaned_directories.unwrap_or(false),
    )
    .map(Into::into)
    .map_err(Into::into)
}
