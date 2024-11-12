// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::anyhow;

use aoide_core::{media::content::resolver::vfs::RemappingVfsResolver, CollectionUid};
use aoide_core_api::media::tracker::untrack_directories::{Outcome, Params, PathsParam, Summary};
use aoide_repo::{
    collection::EntityRepo as CollectionRepo, media::tracker::Repo as MediaTrackerRepo,
};

use crate::{collection::vfs::RepoContext, Error, Result};

#[allow(clippy::missing_panics_doc)] // Never panics
pub fn untrack_directories<Repo>(
    repo: &mut Repo,
    collection_uid: &CollectionUid,
    params: &Params,
) -> Result<Outcome>
where
    Repo: CollectionRepo + MediaTrackerRepo,
{
    let Params {
        root_url,
        paths,
        status,
    } = params;
    let collection_ctx = RepoContext::resolve(repo, collection_uid, root_url.as_ref())?;
    let Some(resolver) = &collection_ctx.content_path.resolver else {
        let path_kind = collection_ctx.content_path.kind;
        return Err(Error::Other(anyhow!(
            "unsupported path kind: {path_kind:?}"
        )));
    };
    let collection_id = collection_ctx.record_id;
    let mut untracked = 0;
    match paths {
        PathsParam::RootDirectory => {
            untracked += repo.media_tracker_untrack_directories(
                collection_id,
                resolver.root_path(),
                *status,
            )?;
        }
        PathsParam::SubDirectories(paths) => {
            for path in paths {
                untracked +=
                    repo.media_tracker_untrack_directories(collection_id, path, *status)?;
            }
        }
    }
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
