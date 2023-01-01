// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

pub fn load_one(connection: &mut DbConnection, entity_uid: &EntityUid) -> Result<Entity> {
    let mut repo = RepoConnection::new(connection);
    let (_, entity) = repo.load_track_entity_by_uid(entity_uid)?;
    Ok(entity)
}

pub fn load_many(
    connection: &mut DbConnection,
    entity_uids: impl IntoIterator<Item = EntityUid>,
    collector: &mut impl RecordCollector<Header = RecordHeader, Record = Entity>,
) -> Result<()> {
    let mut repo = RepoConnection::new(connection);
    for entity_uid in entity_uids {
        match repo.load_track_entity_by_uid(&entity_uid) {
            Ok((record_header, entity)) => {
                collector.collect(record_header, entity);
            }
            Err(RepoError::NotFound) => {
                log::debug!("Track with UID '{entity_uid}' not found");
                continue;
            }
            Err(err) => {
                return Err(err.into());
            }
        }
    }
    Ok(())
}
