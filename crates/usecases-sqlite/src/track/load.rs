// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{CollectionUid, TrackEntity, TrackUid, track::actor::ActorNamesSummarySplitter};
use aoide_repo::{
    RecordCollector, RepoError,
    collection::EntityRepo as _,
    track::{ActorRepo as _, EntityRepo as _, RecordHeader},
};
use aoide_repo_sqlite::DbConnection;

use crate::{RepoConnection, Result};

pub fn load_one(connection: &mut DbConnection, entity_uid: &TrackUid) -> Result<TrackEntity> {
    let mut repo = RepoConnection::new(connection);
    let (_, entity) = repo.load_track_entity_by_uid(entity_uid)?;
    Ok(entity)
}

pub fn load_many(
    connection: &mut DbConnection,
    entity_uids: impl IntoIterator<Item = TrackUid>,
    collector: &mut impl RecordCollector<Header = RecordHeader, Record = TrackEntity>,
) -> Result<()> {
    let mut repo = RepoConnection::new(connection);
    for entity_uid in entity_uids {
        match repo.load_track_entity_by_uid(&entity_uid) {
            Ok((record_header, entity)) => {
                collector.collect(record_header, entity);
            }
            Err(RepoError::NotFound) => {
                log::debug!("Track with UID '{entity_uid}' not found");
            }
            Err(err) => {
                return Err(err.into());
            }
        }
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
