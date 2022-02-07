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

use aoide_repo::{
    collection::RecordId as CollectionId,
    media::source::CollectionRepo as MediaSourceCollectionRepo, track::CollectionRepo,
};

use super::*;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PurgeByMediaSourcePathPredicatesSummary {
    pub purged_media_sources: usize,
    pub purged_tracks: usize,
}

pub fn purge_by_media_source_path_predicates<Repo>(
    repo: &Repo,
    collection_id: CollectionId,
    path_predicates: Vec<StringPredicate>,
) -> RepoResult<PurgeByMediaSourcePathPredicatesSummary>
where
    Repo: CollectionRepo + MediaSourceCollectionRepo,
{
    let mut summary = PurgeByMediaSourcePathPredicatesSummary::default();
    for path_predicate in path_predicates {
        // 1st step: Delete the tracks, leaving the correpsonding media sources orphaned
        let purged_tracks = repo
            .purge_tracks_by_media_source_path_predicate(collection_id, path_predicate.borrow())?;
        // 2nd step: Delete all orphaned media sources
        let purged_media_sources = repo.purge_orphaned_media_sources_by_path_predicate(
            collection_id,
            path_predicate.borrow(),
        )?;
        debug_assert!(purged_tracks <= purged_media_sources);
        summary.purged_tracks += purged_tracks;
        summary.purged_media_sources += purged_media_sources;
    }
    Ok(summary)
}
