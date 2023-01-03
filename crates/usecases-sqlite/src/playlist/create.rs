// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

pub fn create(
    connection: &mut DbConnection,
    collection_uid: Option<&CollectionUid>,
    new_playlist: Playlist,
) -> Result<Entity> {
    let created_entity = uc::playlist::create_entity(new_playlist)?;
    let mut repo = RepoConnection::new(connection);
    uc::playlist::store_created_entity(&mut repo, collection_uid, &created_entity)?;
    Ok(created_entity)
}
