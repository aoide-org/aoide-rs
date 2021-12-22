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

use aoide_core_api::media::tracker::{query_status::Params, Status};

use aoide_repo::{collection::RecordId as CollectionId, media::tracker::Repo as MediaTrackerRepo};

use super::*;

pub fn query_status<Repo>(
    repo: &Repo,
    collection_id: CollectionId,
    source_path_resolver: &VirtualFilePathResolver,
    params: &Params,
) -> Result<Status>
where
    Repo: MediaTrackerRepo,
{
    let Params { root_url } = params;
    let root_path_prefix = root_url
        .as_ref()
        .map(|url| resolve_path_prefix_from_base_url(source_path_resolver, url))
        .transpose()?
        .unwrap_or_default();
    let directories =
        repo.media_tracker_aggregate_directories_tracking_status(collection_id, &root_path_prefix)?;
    let status = Status { directories };
    Ok(status)
}
