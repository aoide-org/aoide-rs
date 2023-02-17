// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_api::media::source::purge_orphaned::{Outcome, Params, Summary};

use aoide_repo::{
    collection::EntityRepo as CollectionRepo,
    media::source::CollectionRepo as MediaSourceCollectionRepo,
};

use crate::collection::vfs::RepoContext;

use super::*;

/// Purge orphaned media sources that don't belong to any track
pub fn purge_orphaned<Repo>(
    repo: &mut Repo,
    collection_uid: &CollectionUid,
    params: &Params,
) -> Result<Outcome>
where
    Repo: CollectionRepo + MediaSourceCollectionRepo,
{
    let Params { root_url } = params;
    let collection_ctx = RepoContext::resolve(repo, collection_uid, root_url.as_ref())?;
    let collection_id = collection_ctx.record_id;
    let root_path_prefix = collection_ctx.root_path_prefix_str(root_url.as_ref());
    let purged = if let Some(root_path_prefix) = root_path_prefix {
        repo.purge_orphaned_media_sources_by_content_path_predicate(
            collection_id,
            StringPredicate::Prefix(root_path_prefix.into()),
        )
    } else {
        repo.purge_orphaned_media_sources(collection_id)
    }?;
    let (root_url, root_path) =
        collection_ctx
            .content_path
            .resolver
            .map_or((None, None), |resolver| {
                let (root_url, root_path) = resolver.dismantle();
                (Some(root_url), Some(root_path))
            });
    let summary = Summary { purged };
    let outcome = Outcome {
        root_url,
        root_path,
        summary,
    };
    Ok(outcome)
}
