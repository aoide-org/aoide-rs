// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{CollectionUid, TrackHeader, media::content::ContentPath};
use aoide_repo::collection::EntityRepo as _;
use aoide_repo_sqlite::DbConnection;
use aoide_usecases::track::resolve as uc;

use crate::{RepoConnection, Result};

pub fn resolve_by_media_source_content_paths(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    media_content_paths: Vec<ContentPath<'static>>,
) -> Result<Vec<(ContentPath<'static>, TrackHeader)>> {
    let mut repo = RepoConnection::new(connection);
    let collection_id = repo.resolve_collection_id(collection_uid)?;
    uc::resolve_by_media_source_content_paths(&mut repo, collection_id, media_content_paths)
        .map_err(Into::into)
}
