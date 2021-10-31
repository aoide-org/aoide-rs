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

use aoide_core_ext::filtering::StringPredicate;

use aoide_repo::{
    collection::RecordId as CollectionId, media::source::Repo as MediaSourceRepo,
    prelude::RepoResult,
};

pub fn purge_orphaned_by_path_predicates<Repo>(
    repo: &Repo,
    collection_id: CollectionId,
    path_predicates: Vec<StringPredicate>,
) -> RepoResult<usize>
where
    Repo: MediaSourceRepo,
{
    let mut total_count = 0;
    for path_predicate in path_predicates {
        total_count += repo.purge_orphaned_media_sources_by_path_predicate(
            collection_id,
            path_predicate.borrow(),
        )?
    }
    Ok(total_count)
}
