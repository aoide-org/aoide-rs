// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::borrow::Cow;

use anyhow::anyhow;
use semval::prelude::*;

use aoide_core::{
    CollectionUid, Playlist, PlaylistEntity, PlaylistHeader, PlaylistUid,
    playlist::EntityWithEntries, util::clock::OffsetDateTimeMs,
};
use aoide_core_api::{Pagination, playlist::EntityWithEntriesSummary};
use aoide_repo::{
    RepoResult, ReservableRecordCollector,
    collection::EntityRepo as CollectionRepo,
    playlist::{CollectionFilter as RepoCollectionFilter, EntityRepo, KindFilter, RecordHeader},
};

use crate::{Error, InputResult, Result};

pub mod entries;

#[derive(Debug)]
pub struct ValidatedInput(Playlist);

pub fn validate_input(playlist: Playlist) -> InputResult<ValidatedInput> {
    if let Err(err) = playlist.validate() {
        return Err(anyhow::anyhow!("invalid playlist input: {:?}", err).into());
    }
    Ok(ValidatedInput(playlist))
}

pub fn create_entity(new_playlist: Playlist) -> Result<PlaylistEntity> {
    let ValidatedInput(playlist) = validate_input(new_playlist)?;
    let header = PlaylistHeader::initial_random();
    let entity = PlaylistEntity::new(header, playlist);
    Ok(entity)
}

pub fn store_created_entity<Repo>(
    repo: &mut Repo,
    collection_uid: Option<&CollectionUid>,
    entity: &PlaylistEntity,
) -> RepoResult<()>
where
    Repo: CollectionRepo + EntityRepo,
{
    let collection_id = collection_uid
        .map(|uid| repo.resolve_collection_id(uid))
        .transpose()?;
    let created_at = OffsetDateTimeMs::now_utc();
    repo.insert_playlist_entity(collection_id, &created_at, entity)?;
    Ok(())
}

pub fn update_entity(hdr: PlaylistHeader, modified_playlist: Playlist) -> Result<PlaylistEntity> {
    let ValidatedInput(playlist) = validate_input(modified_playlist)?;
    let next_hdr = hdr
        .next_rev()
        .ok_or_else(|| Error::Other(anyhow!("no next revision")))?;
    let updated_entity = PlaylistEntity::new(next_hdr, playlist);
    Ok(updated_entity)
}

pub fn store_updated_entity<Repo>(
    repo: &mut Repo,
    updated_entity: &PlaylistEntity,
) -> RepoResult<()>
where
    Repo: EntityRepo,
{
    let updated_at = OffsetDateTimeMs::now_utc();
    repo.update_playlist_entity_revision(&updated_at, updated_entity)?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionFilter<'a> {
    pub uid: Option<Cow<'a, CollectionUid>>,
}

pub fn load_one_with_entries<Repo>(
    repo: &mut Repo,
    playlist_uid: &PlaylistUid,
) -> Result<EntityWithEntries>
where
    Repo: CollectionRepo + EntityRepo,
{
    let id = repo.resolve_playlist_id(playlist_uid)?;
    repo.load_playlist_entity_with_entries(id)
        .map(|(_, entity_with_entries)| entity_with_entries)
        .map_err(Into::into)
}

pub fn load_all_with_entries_summary<Repo>(
    repo: &mut Repo,
    collection_filter: Option<CollectionFilter<'_>>,
    kind_filter: Option<KindFilter<'_>>,
    pagination: Option<&Pagination>,
    collector: &mut impl ReservableRecordCollector<
        Header = RecordHeader,
        Record = EntityWithEntriesSummary,
    >,
) -> Result<()>
where
    Repo: CollectionRepo + EntityRepo,
{
    let collection_filter = collection_filter
        .map(|CollectionFilter { uid }| {
            uid.as_ref()
                .map(|uid| repo.resolve_collection_id(uid))
                .transpose()
        })
        .transpose()?
        .map(|id| RepoCollectionFilter { id });
    repo.load_playlist_entities_with_entries_summary(
        collection_filter,
        kind_filter,
        pagination,
        collector,
    )
    .map_err(Into::into)
}
