// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_api::collection::{EntityWithSummary, LoadScope};
use aoide_repo::collection::MediaSourceRootUrlFilter;

use super::*;

pub fn load_one(
    connection: &mut DbConnection,
    entity_uid: &EntityUid,
    scope: LoadScope,
) -> Result<EntityWithSummary> {
    let mut repo = RepoConnection::new(connection);
    let id = repo.resolve_collection_id(entity_uid)?;
    let (record_hdr, entity) = repo.load_collection_entity(id)?;
    let summary = match scope {
        LoadScope::Entity => None,
        LoadScope::EntityWithSummary => Some(repo.load_collection_summary(record_hdr.id)?),
    };
    Ok(EntityWithSummary { entity, summary })
}

pub fn load_all(
    connection: &mut DbConnection,
    kind: Option<&str>,
    media_source_root_url: Option<&MediaSourceRootUrlFilter>,
    scope: LoadScope,
    pagination: Option<&Pagination>,
    collector: &mut impl ReservableRecordCollector<Header = RecordHeader, Record = EntityWithSummary>,
) -> Result<()> {
    let mut repo = RepoConnection::new(connection);
    let with_summary = match scope {
        LoadScope::Entity => false,
        LoadScope::EntityWithSummary => true,
    };
    repo.load_collection_entities(
        kind,
        media_source_root_url,
        with_summary,
        pagination,
        collector,
    )
    .map_err(Into::into)
}

pub fn load_all_kinds(connection: &mut DbConnection) -> Result<Vec<String>> {
    let mut repo = RepoConnection::new(connection);
    repo.load_all_kinds().map_err(Into::into)
}
