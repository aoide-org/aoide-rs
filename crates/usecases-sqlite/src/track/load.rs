// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::BTreeMap;

use aoide_core::{
    CollectionUid, TrackEntity, TrackUid,
    tag::{FacetId, Label},
    track::actor::ActorNamesSummarySplitter,
};
use aoide_repo::{
    RecordCollector,
    collection::EntityRepo as _,
    track::{ActorRepo as _, EntityRepo as _, RecordHeader, TagRepo as _},
};
use aoide_repo_sqlite::DbConnection;

use crate::{RepoConnection, Result};

pub fn load_one(connection: &mut DbConnection, entity_uid: &TrackUid) -> Result<TrackEntity> {
    let mut repo = RepoConnection::new(connection);
    let (_, entity) = repo.load_track_entity_by_uid(entity_uid)?;
    Ok(entity)
}

/// Load multiple tracks in the order of `entity_uids`.
///
/// Unknown tracks are skipped. Must be invoked within a transaction.
#[expect(clippy::missing_panics_doc)] // Never panics
pub fn load_many(
    connection: &mut DbConnection,
    entity_uids: impl IntoIterator<Item = TrackUid>,
    collector: &mut impl RecordCollector<Header = RecordHeader, Record = TrackEntity>,
) -> Result<()> {
    let mut repo = RepoConnection::new(connection);
    let entity_uids = entity_uids.into_iter().collect::<Vec<_>>();
    let ids = repo.resolve_track_ids(&entity_uids)?;
    // Each track is only loaded once, even if requested multiple times.
    let mut remaining_counts = BTreeMap::<_, usize>::new();
    for id in ids.iter().flatten() {
        *remaining_counts.entry(*id).or_default() += 1;
    }
    let unique_ids = remaining_counts.keys().copied().collect::<Vec<_>>();
    let mut loaded = repo
        .load_track_entities(&unique_ids)?
        .into_iter()
        .map(|(record_header, entity)| (record_header.id, (record_header, entity)))
        .collect::<BTreeMap<_, _>>();
    for (entity_uid, id) in entity_uids.iter().zip(ids) {
        let Some(id) = id else {
            log::debug!("Track with UID '{entity_uid}' not found");
            continue;
        };
        let remaining_count = remaining_counts.get_mut(&id).expect("counted");
        *remaining_count -= 1;
        let (record_header, entity) = if *remaining_count > 0 {
            loaded.get(&id).cloned()
        } else {
            loaded.remove(&id)
        }
        .expect("loaded");
        collector.collect(record_header, entity);
    }
    Ok(())
}

pub fn load_all_actor_names(
    connection: &mut DbConnection,
    collection_uid: Option<&CollectionUid>,
    summary_splitter: &ActorNamesSummarySplitter,
) -> Result<Vec<String>> {
    let mut repo = RepoConnection::new(connection);
    let collection_id = collection_uid
        .map(|uid| repo.resolve_collection_id(uid))
        .transpose()?;
    repo.load_all_actor_names(collection_id, summary_splitter)
        .map_err(Into::into)
}

/// Count the tracks per tag label.
///
/// See also: [`aoide_repo::track::TagRepo::count_track_tag_labels()`]
pub fn count_track_tag_labels(
    connection: &mut DbConnection,
    collection_uid: Option<&CollectionUid>,
    facet_id: Option<&FacetId>,
) -> Result<Vec<(Label<'static>, usize)>> {
    let mut repo = RepoConnection::new(connection);
    let collection_id = collection_uid
        .map(|uid| repo.resolve_collection_id(uid))
        .transpose()?;
    repo.count_track_tag_labels(collection_id, facet_id)
        .map_err(Into::into)
}
