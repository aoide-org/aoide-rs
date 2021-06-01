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

use aoide_core::usecases::media::tracker::Status;
use aoide_repo::{collection::RecordId as CollectionId, media::tracker::Repo as MediaTrackerRepo};

use url::Url;

pub fn query_status<Repo>(
    repo: &Repo,
    collection_id: CollectionId,
    source_path_resolver: &VirtualFilePathResolver,
    root_dir_url: Option<&Url>,
) -> Result<Status>
where
    Repo: MediaTrackerRepo,
{
    let path_prefix = root_dir_url
        .map(|url| resolve_path_prefix_from_url(source_path_resolver, url))
        .transpose()?
        .unwrap_or_default();
    let directories =
        repo.media_tracker_aggregate_directories_tracking_status(collection_id, &path_prefix)?;
    let status = Status { directories };
    Ok(status)
}
