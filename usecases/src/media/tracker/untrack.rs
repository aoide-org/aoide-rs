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

use aoide_repo::{collection::RecordId as CollectionId, media::tracker::Repo as MediaTrackerRepo};

use url::Url;

pub fn untrack<Repo>(
    repo: &Repo,
    collection_id: CollectionId,
    root_url: &Url,
    source_path_resolver: &impl SourcePathResolver,
    status: Option<DirTrackingStatus>,
) -> Result<usize>
where
    Repo: MediaTrackerRepo,
{
    let root_path_prefix = resolve_path_prefix_from_url(source_path_resolver, root_url)?;
    Ok(repo.media_tracker_untrack(collection_id, &root_path_prefix, status)?)
}
