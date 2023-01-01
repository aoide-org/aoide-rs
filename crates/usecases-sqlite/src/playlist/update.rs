// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use uc::playlist::{store_updated_entity, update_entity};

use super::*;

pub fn update(
    connection: &mut DbConnection,
    entity_header: EntityHeader,
    modified_playlist: Playlist,
) -> Result<Entity> {
    let updated_entity = update_entity(entity_header, modified_playlist)?;
    let mut repo = RepoConnection::new(connection);
    store_updated_entity(&mut repo, &updated_entity)?;
    Ok(updated_entity)
}
