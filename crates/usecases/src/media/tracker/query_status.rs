// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_api::media::tracker::{query_status::Params, Status};

use aoide_repo::{
    collection::EntityRepo as CollectionRepo, media::tracker::Repo as MediaTrackerRepo,
};

use crate::collection::vfs::RepoContext;

use super::*;

pub fn query_status<Repo>(
    repo: &mut Repo,
    collection_uid: &CollectionUid,
    params: &Params,
) -> Result<Status>
where
    Repo: CollectionRepo + MediaTrackerRepo,
{
    let Params { root_url } = params;
    let collection_ctx = RepoContext::resolve(repo, collection_uid, root_url.as_ref())?;
    let Some(vfs_ctx) = &collection_ctx.content_path.vfs else {
        let path_kind = collection_ctx.content_path.kind;
        return Err(anyhow::anyhow!("Unsupported path kind: {path_kind:?}").into());
    };
    let collection_id = collection_ctx.record_id;
    let directories = repo
        .media_tracker_aggregate_directories_tracking_status(collection_id, &vfs_ctx.root_path)?;
    let status = Status { directories };
    Ok(status)
}
