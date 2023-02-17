// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;

use aoide_media::io::import::{load_embedded_artwork_image_from_file_path, LoadedArtworkImage};

use aoide_repo::collection::RecordId as CollectionId;

use aoide_core::media::content::ContentPath;

use uc::collection::vfs::RepoContext;

use super::*;

pub mod purge_orphaned;
pub mod purge_untracked;
pub mod relocate;

pub fn resolve_file_path(
    repo: &mut RepoConnection<'_>,
    collection_uid: &CollectionUid,
    content_path: &ContentPath<'_>,
) -> Result<(CollectionId, PathBuf)> {
    let collection_ctx = RepoContext::resolve(repo, collection_uid, None)?;
    let Some(resolver) = &collection_ctx.content_path.resolver else {
        let path_kind = collection_ctx.content_path.kind;
        return Err(anyhow::anyhow!("Unsupported path kind: {path_kind:?}").into());
    };
    let file_path = resolver.build_file_path(content_path);
    Ok((collection_ctx.record_id, file_path))
}

pub fn load_embedded_artwork_image(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    content_path: &ContentPath<'_>,
) -> Result<(CollectionId, Option<LoadedArtworkImage>)> {
    let mut repo = RepoConnection::new(connection);
    resolve_file_path(&mut repo, collection_uid, content_path).and_then(
        |(collection_id, file_path)| {
            let loaded_artwork_image = load_embedded_artwork_image_from_file_path(&file_path)?;
            Ok((collection_id, loaded_artwork_image))
        },
    )
}
