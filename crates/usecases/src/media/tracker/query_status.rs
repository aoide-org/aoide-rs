// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::anyhow;

use aoide_core::CollectionUid;
use aoide_core_api::media::tracker::{Status, query_status::Params};
use aoide_repo::{
    collection::EntityRepo as CollectionRepo, media::tracker::Repo as MediaTrackerRepo,
};

use crate::{Error, Result, collection::vfs::RepoContext};

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
    let Some(resolver) = &collection_ctx.content_path.resolver else {
        let path_kind = collection_ctx.content_path.kind;
        return Err(Error::Other(anyhow!(
            "unsupported path kind: {path_kind:?}"
        )));
    };
    let collection_id = collection_ctx.record_id;
    let directories = repo
        .media_tracker_aggregate_directories_tracking_status(collection_id, resolver.root_path())?;
    let status = Status { directories };
    Ok(status)
}
