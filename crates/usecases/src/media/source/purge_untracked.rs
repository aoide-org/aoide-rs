// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_api::media::source::purge_untracked::{Outcome, Params, Summary};

use aoide_repo::{
    collection::EntityRepo as CollectionRepo,
    media::source::CollectionRepo as MediaSourceCollectionRepo,
};

use crate::collection::vfs::RepoContext;

use super::*;

pub fn purge_untracked<Repo>(
    repo: &mut Repo,
    collection_uid: &CollectionUid,
    params: &Params,
) -> Result<Outcome>
where
    Repo: CollectionRepo + MediaSourceCollectionRepo,
{
    let Params { root_url } = params;
    let collection_ctx = RepoContext::resolve(repo, collection_uid, root_url.as_ref())?;
    let vfs_ctx = if let Some(vfs_ctx) = &collection_ctx.content_path.vfs {
        vfs_ctx
    } else {
        let path_kind = collection_ctx.content_path.kind;
        return Err(anyhow::anyhow!("Unsupported path kind: {path_kind:?}",).into());
    };
    let collection_id = collection_ctx.record_id;
    let purged = if vfs_ctx.root_path.is_empty() {
        repo.purge_untracked_media_sources(collection_id)
    } else {
        let root_path_predicate = StringPredicateBorrowed::Prefix(&vfs_ctx.root_path);
        repo.purge_untracked_media_sources_by_content_path_predicate(
            collection_id,
            root_path_predicate,
        )
    }?;
    let (root_url, root_path) = collection_ctx
        .content_path
        .vfs
        .map(|vfs_context| (vfs_context.root_url, vfs_context.root_path))
        .expect("collection with path kind VFS");
    let summary = Summary { purged };
    let outcome = Outcome {
        root_url,
        root_path,
        summary,
    };
    Ok(outcome)
}
