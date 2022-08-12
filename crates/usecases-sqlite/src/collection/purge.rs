// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

pub fn purge(connection: &mut DbConnection, entity_uid: &EntityUid) -> Result<()> {
    let mut repo = RepoConnection::new(connection);
    let id = repo.resolve_collection_id(entity_uid)?;
    repo.purge_collection_entity(id).map_err(Into::into)
}
