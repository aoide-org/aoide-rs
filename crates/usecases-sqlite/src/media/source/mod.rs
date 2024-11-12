// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;

use anyhow::anyhow;

use aoide_core::{media::content::ContentPath, CollectionUid};
use aoide_repo::CollectionId;
use aoide_usecases::collection::vfs::RepoContext;

use crate::{Error, RepoConnection, Result};

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
        return Err(Error::Other(anyhow!(
            "unsupported path kind: {path_kind:?}"
        )));
    };
    let file_path = resolver.build_file_path(content_path);
    Ok((collection_ctx.record_id, file_path))
}
