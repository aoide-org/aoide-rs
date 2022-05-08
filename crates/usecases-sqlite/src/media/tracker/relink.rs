// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use std::sync::atomic::AtomicBool;

use aoide_usecases::media::tracker::relink as uc;

use super::*;

pub fn relink_tracks_with_untracked_media_sources<ReportProgressFn: FnMut(&uc::Progress)>(
    connection: &SqliteConnection,
    collection_uid: &CollectionUid,
    find_candidate_params: uc::FindCandidateParams,
    report_progress_fn: &mut ReportProgressFn,
    abort_flag: &AtomicBool,
) -> Result<Vec<uc::RelocatedMediaSource>> {
    let repo = RepoConnection::new(connection);
    uc::relink_tracks_with_untracked_media_sources(
        &repo,
        collection_uid,
        find_candidate_params,
        report_progress_fn,
        abort_flag,
    )
    .map_err(Into::into)
}
