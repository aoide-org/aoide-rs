// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{collection::EntityHeader, util::clock::DateTime};

use uc::collection::update_entity;

use super::*;

pub fn update(
    connection: &SqliteConnection,
    entity_header: EntityHeader,
    modified_collection: Collection,
) -> Result<Entity> {
    let updated_entity = update_entity(entity_header, modified_collection)?;
    let updated_at = DateTime::now_utc();
    let repo = RepoConnection::new(connection);
    repo.update_collection_entity_revision(updated_at, &updated_entity)?;
    Ok(updated_entity)
}
