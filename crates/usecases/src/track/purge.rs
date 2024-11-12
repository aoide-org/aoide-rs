// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_api::filtering::StringPredicate;
use aoide_repo::{
    media::source::CollectionRepo as MediaSourceCollectionRepo, track::CollectionRepo,
    CollectionId, RepoResult,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PurgeByMediaContentPathPredicatesSummary {
    pub purged_media_sources: usize,
    pub purged_tracks: usize,
}

pub fn purge_by_media_source_content_path_predicates<'a, Repo>(
    repo: &mut Repo,
    collection_id: CollectionId,
    path_predicates: impl IntoIterator<Item = StringPredicate<'a>>,
) -> RepoResult<PurgeByMediaContentPathPredicatesSummary>
where
    Repo: CollectionRepo + MediaSourceCollectionRepo,
{
    let mut summary = PurgeByMediaContentPathPredicatesSummary::default();
    for path_predicate in path_predicates {
        // 1st step: Delete the tracks, leaving the corresponding media sources orphaned
        let purged_tracks = repo.purge_tracks_by_media_source_content_path_predicate(
            collection_id,
            path_predicate.to_borrowed(),
        )?;
        // 2nd step: Delete all orphaned media sources
        let purged_media_sources = repo.purge_orphaned_media_sources_by_content_path_predicate(
            collection_id,
            path_predicate,
        )?;
        debug_assert!(purged_tracks <= purged_media_sources);
        summary.purged_tracks += purged_tracks;
        summary.purged_media_sources += purged_media_sources;
    }
    Ok(summary)
}
