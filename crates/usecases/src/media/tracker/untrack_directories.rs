// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::media::content::resolver::vfs::RemappingVfsResolver;
use aoide_core_api::media::tracker::untrack_directories::{Outcome, Params, Summary};
use aoide_repo::{
    collection::EntityRepo as CollectionRepo, media::tracker::Repo as MediaTrackerRepo,
};

use super::*;
use crate::collection::vfs::RepoContext;

#[allow(clippy::missing_panics_doc)] // Never panics
pub fn untrack_directories<Repo>(
    repo: &mut Repo,
    collection_uid: &CollectionUid,
    params: &Params,
) -> Result<Outcome>
where
    Repo: CollectionRepo + MediaTrackerRepo,
{
    let Params { root_url, status } = params;
    let collection_ctx = RepoContext::resolve(repo, collection_uid, root_url.as_ref())?;
    let Some(resolver) = &collection_ctx.content_path.resolver else {
        let path_kind = collection_ctx.content_path.kind;
        return Err(anyhow::anyhow!("unsupported path kind: {path_kind:?}").into());
    };
    let collection_id = collection_ctx.record_id;
    let untracked =
        repo.media_tracker_untrack_directories(collection_id, resolver.root_path(), *status)?;
    let (root_url, root_path) = collection_ctx
        .content_path
        .resolver
        .map(RemappingVfsResolver::dismantle)
        .expect("collection with path kind VFS");
    let summary = Summary { untracked };
    Ok(Outcome {
        root_url,
        root_path,
        summary,
    })
}
