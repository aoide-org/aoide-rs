// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{CollectionUid, media::content::ContentPath, util::clock::UtcDateTimeMs};
use aoide_repo::{collection::EntityRepo as _, media::source::CollectionRepo as _};
use aoide_repo_sqlite::DbConnection;

use crate::{RepoConnection, Result};

pub fn relocate(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    old_content_path_prefix: &ContentPath<'_>,
    new_content_path_prefix: &ContentPath<'_>,
) -> Result<usize> {
    let mut repo = RepoConnection::new(connection);
    let collection_id = repo.resolve_collection_id(collection_uid)?;
    let updated_at = UtcDateTimeMs::now();
    repo.relocate_media_sources_by_content_path_prefix(
        collection_id,
        updated_at,
        old_content_path_prefix,
        new_content_path_prefix,
    )
    .map_err(Into::into)
}
