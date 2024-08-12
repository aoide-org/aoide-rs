// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::anyhow;
use aoide_core::media::content::resolver::vfs::RemappingVfsResolver;
use aoide_core_api::media::source::purge_untracked::{Outcome, Params, Summary};
use aoide_repo::{
    collection::EntityRepo as CollectionRepo,
    media::source::CollectionRepo as MediaSourceCollectionRepo,
};

use super::*;
use crate::collection::vfs::RepoContext;

#[allow(clippy::missing_panics_doc)] // Never panics
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
    let Some(resolver) = &collection_ctx.content_path.resolver else {
        let path_kind = collection_ctx.content_path.kind;
        return Err(Error::Other(anyhow!(
            "unsupported path kind: {path_kind:?}",
        )));
    };
    let collection_id = collection_ctx.record_id;
    let purged = if resolver.root_path().is_empty() {
        repo.purge_untracked_media_sources(collection_id)
    } else {
        let root_path_predicate =
            StringPredicate::Prefix(resolver.root_path().to_borrowed().into_inner());
        repo.purge_untracked_media_sources_by_content_path_predicate(
            collection_id,
            root_path_predicate,
        )
    }?;
    let (root_url, root_path) = collection_ctx
        .content_path
        .resolver
        .map(RemappingVfsResolver::dismantle)
        .expect("collection with path kind VFS");
    let summary = Summary { purged };
    let outcome = Outcome {
        root_url,
        root_path,
        summary,
    };
    Ok(outcome)
}
