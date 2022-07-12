// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

pub fn purge(connection: &SqliteConnection, entity_uid: &EntityUid) -> Result<()> {
    let repo = RepoConnection::new(connection);
    let id = repo.resolve_playlist_id(entity_uid)?;
    repo.purge_playlist_entity(id).map_err(Into::into)
}
