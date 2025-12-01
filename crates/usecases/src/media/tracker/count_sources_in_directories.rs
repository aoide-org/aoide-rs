// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::anyhow;

use aoide_core::{CollectionUid, media::content::ContentPath};
use aoide_core_api::media::tracker::count_sources_in_directories::Params;
use aoide_repo::{
    collection::EntityRepo as CollectionRepo, media::tracker::Repo as MediaTrackerRepo,
};

use crate::{Error, Result, collection::vfs::RepoContext};

pub fn count_sources_in_directories<Repo>(
    repo: &mut Repo,
    collection_uid: &CollectionUid,
    params: &Params,
) -> Result<Vec<(ContentPath<'static>, usize)>>
where
    Repo: CollectionRepo + MediaTrackerRepo,
{
    let Params {
        root_url,
        filter,
        order,
        pagination,
    } = params;
    let collection_ctx = RepoContext::resolve(repo, collection_uid, root_url.as_ref())?;
    let Some(resolver) = &collection_ctx.content_path.resolver else {
        let path_kind = collection_ctx.content_path.kind;
        return Err(Error::Other(anyhow!(
            "unsupported path kind: {path_kind:?}"
        )));
    };
    let collection_id = collection_ctx.record_id;
    repo.media_tracker_count_sources_in_directories(
        collection_id,
        resolver.root_path(),
        filter,
        *order,
        pagination,
    )
    .map_err(Into::into)
}
