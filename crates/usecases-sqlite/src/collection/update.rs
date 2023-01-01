// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::collection::EntityHeader;

use uc::collection::{store_updated_entity, update_entity};

use super::*;

pub fn update(
    connection: &mut DbConnection,
    entity_header: EntityHeader,
    modified_collection: Collection,
) -> Result<Entity> {
    let updated_entity = update_entity(entity_header, modified_collection)?;
    let mut repo = RepoConnection::new(connection);
    store_updated_entity(&mut repo, &updated_entity)?;
    Ok(updated_entity)
}
