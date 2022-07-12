// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use semval::Validate as _;

use aoide_core::{
    collection::EntityUid as CollectionUid,
    playlist::{Entity, EntityHeader as PlaylistHeader, Playlist},
    util::clock::DateTime,
};

use aoide_repo::{
    collection::EntityRepo as CollectionRepo, playlist::CollectionRepo as PlaylistCollectionRepo,
};

use super::*;

#[derive(Debug)]
pub struct ValidatedInput(Playlist);

pub fn validate_input(playlist: Playlist) -> InputResult<ValidatedInput> {
    if let Err(err) = playlist.validate() {
        return Err(anyhow::anyhow!("Invalid playlist input: {:?}", err).into());
    }
    Ok(ValidatedInput(playlist))
}

pub fn create_entity(new_playlist: Playlist) -> Result<Entity> {
    let ValidatedInput(playlist) = validate_input(new_playlist)?;
    let header = PlaylistHeader::initial_random();
    let entity = Entity::new(header, playlist);
    Ok(entity)
}

pub fn store_created_entity<Repo>(
    repo: &Repo,
    collection_uid: &CollectionUid,
    entity: &Entity,
) -> RepoResult<()>
where
    Repo: CollectionRepo + PlaylistCollectionRepo,
{
    let collection_id = repo.resolve_collection_id(collection_uid)?;
    let created_at = DateTime::now_utc();
    repo.insert_playlist_entity(collection_id, created_at, entity)?;
    Ok(())
}

pub fn update_entity(hdr: PlaylistHeader, modified_playlist: Playlist) -> Result<Entity> {
    let ValidatedInput(playlist) = validate_input(modified_playlist)?;
    let next_hdr = hdr
        .next_rev()
        .ok_or_else(|| anyhow::anyhow!("no next revision"))?;
    let updated_entity = Entity::new(next_hdr, playlist);
    Ok(updated_entity)
}
