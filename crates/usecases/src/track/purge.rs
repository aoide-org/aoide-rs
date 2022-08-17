// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_repo::{
    collection::RecordId as CollectionId,
    media::source::CollectionRepo as MediaSourceCollectionRepo, track::CollectionRepo,
};

use super::*;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PurgeByMediaContentPathPredicatesSummary {
    pub purged_media_sources: usize,
    pub purged_tracks: usize,
}

pub fn purge_by_media_source_content_path_predicates<Repo>(
    repo: &mut Repo,
    collection_id: CollectionId,
    path_predicates: impl IntoIterator<Item = StringPredicate>,
) -> RepoResult<PurgeByMediaContentPathPredicatesSummary>
where
    Repo: CollectionRepo + MediaSourceCollectionRepo,
{
    let mut summary = PurgeByMediaContentPathPredicatesSummary::default();
    for path_predicate in path_predicates {
        // 1st step: Delete the tracks, leaving the corresponding media sources orphaned
        let purged_tracks = repo.purge_tracks_by_media_source_content_path_predicate(
            collection_id,
            path_predicate.borrow(),
        )?;
        // 2nd step: Delete all orphaned media sources
        let purged_media_sources = repo.purge_orphaned_media_sources_by_content_path_predicate(
            collection_id,
            path_predicate.borrow(),
        )?;
        debug_assert!(purged_tracks <= purged_media_sources);
        summary.purged_tracks += purged_tracks;
        summary.purged_media_sources += purged_media_sources;
    }
    Ok(summary)
}
