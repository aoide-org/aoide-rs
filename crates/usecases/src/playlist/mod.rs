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

use semval::Validate as _;

use aoide_core::{
    entity::{EntityHeader, EntityUid},
    playlist::{Entity, Playlist},
    util::clock::DateTime,
};

use aoide_repo::{collection::EntityRepo as CollectionRepo, playlist::EntityRepo as PlaylistRepo};

use super::*;

#[derive(Debug)]
pub struct ValidatedInput(Playlist);

pub fn validate_input(playlist: Playlist) -> InputResult<ValidatedInput> {
    if let Err(err) = playlist.validate() {
        return Err(anyhow::anyhow!("Invalid playlist input: {:?}", err).into());
    }
    Ok(ValidatedInput(playlist))
}

pub fn create_entity(playlist: Playlist) -> Result<Entity> {
    let ValidatedInput(playlist) = validate_input(playlist)?;
    let header = EntityHeader::initial_random();
    let entity = Entity::new(header, playlist);
    Ok(entity)
}

pub fn store_created_entity<Repo>(
    repo: &Repo,
    collection_uid: &EntityUid,
    entity: &Entity,
) -> RepoResult<()>
where
    Repo: CollectionRepo + PlaylistRepo,
{
    let collection_id = repo.resolve_collection_id(collection_uid)?;
    let created_at = DateTime::now_utc();
    repo.insert_collected_playlist_entity(collection_id, created_at, entity)?;
    Ok(())
}

pub fn update_entity(hdr: EntityHeader, playlist: Playlist) -> Result<Entity> {
    let ValidatedInput(playlist) = validate_input(playlist)?;
    let next_hdr = hdr
        .next_rev()
        .ok_or_else(|| anyhow::anyhow!("no next revision"))?;
    let updated_entity = Entity::new(next_hdr, playlist);
    Ok(updated_entity)
}
