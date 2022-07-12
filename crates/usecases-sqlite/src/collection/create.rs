// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use uc::collection::{create_entity, store_created_entity};

use super::*;

pub fn create(connection: &SqliteConnection, new_collection: Collection) -> Result<Entity> {
    let created_entity = create_entity(new_collection)?;
    let repo = RepoConnection::new(connection);
    store_created_entity(&repo, &created_entity)?;
    Ok(created_entity)
}
