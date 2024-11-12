// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use url::Url;

use aoide_core::{
    media::content::{resolver::ContentPathResolver as _, ContentPath},
    util::url::BaseUrl,
    Collection, CollectionEntity, CollectionHeader, CollectionUid,
};
use aoide_core_api::{
    collection::{EntityWithSummary, LoadScope},
    Pagination,
};
use aoide_repo::{
    collection::{EntityRepo as _, KindFilter, MediaSourceRootUrlFilter, RecordHeader},
    ReservableRecordCollector,
};
use aoide_usecases::collection::{self as uc, vfs::RepoContext};

use crate::{DbConnection, Error, RepoConnection, Result};

pub fn create(
    connection: &mut DbConnection,
    new_collection: Collection,
) -> Result<CollectionEntity> {
    let created_entity = uc::create_entity(new_collection)?;
    let mut repo = RepoConnection::new(connection);
    uc::store_created_entity(&mut repo, &created_entity)?;
    Ok(created_entity)
}

pub fn update(
    connection: &mut DbConnection,
    entity_header: CollectionHeader,
    modified_collection: Collection,
) -> Result<CollectionEntity> {
    let updated_entity = uc::update_entity(entity_header, modified_collection)?;
    let mut repo = RepoConnection::new(connection);
    uc::store_updated_entity(&mut repo, &updated_entity)?;
    Ok(updated_entity)
}

pub fn purge(connection: &mut DbConnection, entity_uid: &CollectionUid) -> Result<()> {
    let mut repo = RepoConnection::new(connection);
    uc::purge(&mut repo, entity_uid).map_err(Into::into)
}

pub fn load_one(
    connection: &mut DbConnection,
    entity_uid: &CollectionUid,
    scope: LoadScope,
) -> Result<(RecordHeader, EntityWithSummary)> {
    let mut repo = RepoConnection::new(connection);
    uc::load_one(&mut repo, entity_uid, scope).map_err(Into::into)
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

pub fn resolve_content_path_from_url(
    connection: &mut DbConnection,
    entity_uid: &CollectionUid,
    content_url: &Url,
) -> Result<Option<ContentPath<'static>>> {
    let mut repo = RepoConnection::new(connection);
    let repo_ctx = RepoContext::resolve_override(&mut repo, entity_uid, None, None)?;
    let Some(content_path_resolver) = &repo_ctx.content_path.resolver else {
        return Ok(None);
    };
    content_path_resolver
        .resolve_path_from_url(content_url)
        .map_err(Into::into)
        .map_err(Error::Other)
}

pub fn resolve_url_from_content_path(
    connection: &mut DbConnection,
    entity_uid: &CollectionUid,
    content_path: &ContentPath<'_>,
    override_root_url: Option<BaseUrl>,
) -> Result<Option<Url>> {
    let mut repo = RepoConnection::new(connection);
    let repo_ctx = RepoContext::resolve_override(&mut repo, entity_uid, None, override_root_url)?;
    let Some(content_path_resolver) = &repo_ctx.content_path.resolver else {
        return Ok(None);
    };
    content_path_resolver
        .resolve_url_from_path(content_path)
        .map(Some)
        .map_err(Into::into)
        .map_err(Error::Other)
}

#[cfg(test)]
mod tests;
