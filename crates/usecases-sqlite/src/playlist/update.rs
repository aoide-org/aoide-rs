// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::util::clock::DateTime;
use uc::playlist::update_entity;

use super::*;

pub fn update(
    connection: &mut DbConnection,
    entity_header: EntityHeader,
    modified_playlist: Playlist,
) -> Result<Entity> {
    let updated_entity = update_entity(entity_header, modified_playlist)?;
    let updated_at = DateTime::now_utc();
    let mut repo = RepoConnection::new(connection);
    repo.update_playlist_entity_revision(updated_at, &updated_entity)?;
    Ok(updated_entity)
}
