// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::collection::*;
use aoide_core_api::collection::{EntityWithSummary, LoadScope};
use aoide_repo::{
    collection::{EntityRepo as _, KindFilter, MediaSourceRootUrlFilter, RecordHeader},
    prelude::*,
};

use super::*;

pub fn create(connection: &mut DbConnection, new_collection: Collection) -> Result<Entity> {
    let created_entity = uc::collection::create_entity(new_collection)?;
    let mut repo = RepoConnection::new(connection);
    uc::collection::store_created_entity(&mut repo, &created_entity)?;
    Ok(created_entity)
}

pub fn update(
    connection: &mut DbConnection,
    entity_header: EntityHeader,
    modified_collection: Collection,
) -> Result<Entity> {
    let updated_entity = uc::collection::update_entity(entity_header, modified_collection)?;
    let mut repo = RepoConnection::new(connection);
    uc::collection::store_updated_entity(&mut repo, &updated_entity)?;
    Ok(updated_entity)
}

pub fn purge(connection: &mut DbConnection, entity_uid: &EntityUid) -> Result<()> {
    let mut repo = RepoConnection::new(connection);
    uc::collection::purge(&mut repo, entity_uid).map_err(Into::into)
}

pub fn load_one(
    connection: &mut DbConnection,
    entity_uid: &EntityUid,
    scope: LoadScope,
) -> Result<EntityWithSummary> {
    let mut repo = RepoConnection::new(connection);
    uc::collection::load_one(&mut repo, entity_uid, scope).map_err(Into::into)
}

pub fn load_all(
    connection: &mut DbConnection,
    kind_filter: Option<KindFilter<'_>>,
    media_source_root_url: Option<&MediaSourceRootUrlFilter>,
    scope: LoadScope,
    pagination: Option<&Pagination>,
    collector: &mut impl ReservableRecordCollector<Header = RecordHeader, Record = EntityWithSummary>,
) -> Result<()> {
    let mut repo = RepoConnection::new(connection);
    repo.load_collection_entities(
        kind_filter,
        media_source_root_url,
        scope,
        pagination,
        collector,
    )
    .map_err(Into::into)
}

pub fn load_all_kinds(connection: &mut DbConnection) -> Result<Vec<String>> {
    let mut repo = RepoConnection::new(connection);
    repo.load_all_kinds().map_err(Into::into)
}
