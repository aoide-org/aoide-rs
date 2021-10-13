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

use aoide_core::util::url::BaseUrl;

use aoide_core_ext::media::tracker::{
    untrack::{Outcome, Summary},
    DirTrackingStatus,
};

use aoide_repo::{collection::RecordId as CollectionId, media::tracker::Repo as MediaTrackerRepo};

use super::*;

pub fn untrack<Repo>(
    repo: &Repo,
    collection_id: CollectionId,
    root_url: BaseUrl,
    source_path_resolver: &impl SourcePathResolver,
    status: Option<DirTrackingStatus>,
) -> Result<Outcome>
where
    Repo: MediaTrackerRepo,
{
    let root_path_prefix = resolve_path_prefix_from_base_url(source_path_resolver, &root_url)?;
    let root_url = source_path_resolver
        .resolve_url_from_path(&root_path_prefix)
        .map_err(anyhow::Error::from)?;
    let untracked = repo.media_tracker_untrack(collection_id, &root_path_prefix, status)?;
    let summary = Summary { untracked };
    Ok(Outcome { root_url, summary })
}
