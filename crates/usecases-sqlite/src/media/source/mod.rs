// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;

use aoide_core::media::content::ContentPath;
use aoide_repo::collection::RecordId as CollectionId;
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
