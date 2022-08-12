// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_repo::collection::EntityRepo as _;

use aoide_usecases::track::resolve as uc;

use super::*;

pub fn resolve_by_media_source_content_paths(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    media_content_paths: Vec<String>,
) -> Result<Vec<(String, EntityHeader)>> {
    let mut repo = RepoConnection::new(connection);
    let collection_id = repo.resolve_collection_id(collection_uid)?;
    uc::resolve_by_media_source_content_paths(&mut repo, collection_id, media_content_paths)
        .map_err(Into::into)
}
