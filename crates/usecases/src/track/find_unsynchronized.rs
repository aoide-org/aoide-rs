// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::anyhow;

use aoide_core::{CollectionUid, media::content::resolver::ContentPathResolver};
use aoide_core_api::{
    Pagination,
    filtering::StringPredicate,
    media::source::ResolveUrlFromContentPath,
    track::find_unsynchronized::{Params, UnsynchronizedTrack, UnsynchronizedTrackEntity},
};
use aoide_repo::{
    CollectionId, RepoResult,
    collection::EntityRepo as CollectionRepo,
    track::{CollectionRepo as TrackCollectionRepo, RecordTrail},
};

use crate::{Error, Result, collection::vfs::RepoContext};

#[expect(clippy::missing_panics_doc)] // Never panics
pub fn find_unsynchronized<Repo, Resolver>(
    repo: &mut Repo,
    collection_id: CollectionId,
    pagination: &Pagination,
    content_path_predicate: Option<StringPredicate<'_>>,
    content_path_resolver: Option<&Resolver>,
) -> RepoResult<Vec<UnsynchronizedTrackEntity>>
where
    Repo: TrackCollectionRepo,
    Resolver: ContentPathResolver,
{
    repo.find_unsynchronized_tracks(collection_id, pagination, content_path_predicate)
        .map(|v| {
            v.into_iter()
                .map(|(entity_header, _record_id, record_trail)| {
                    let RecordTrail {
                        collection_id: _,
                        media_source_id: _,
                        content_link,
                        last_synchronized_rev,
                    } = record_trail;
                    let mut content_link = content_link;
                    if let Some(content_path_resolver) = content_path_resolver {
                        // FIXME: Handle errors
                        let url = content_path_resolver
                            .resolve_url_from_path(&content_link.path)
                            .expect("valid content path");
                        content_link.path = url.to_string().into();
                    }
                    let track = UnsynchronizedTrack {
                        content_link,
                        last_synchronized_rev,
                    };
                    UnsynchronizedTrackEntity::new(entity_header, track)
                })
                .collect()
        })
}

pub fn find_unsynchronized_with_params<Repo>(
    repo: &mut Repo,
    collection_uid: &CollectionUid,
    params: Params,
    pagination: &Pagination,
) -> Result<Vec<UnsynchronizedTrackEntity>>
where
    Repo: CollectionRepo + TrackCollectionRepo,
{
    let Params {
        resolve_url_from_content_path,
        content_path_predicate,
    } = params;
    let collection_ctx = RepoContext::resolve_override(
        repo,
        collection_uid,
        None,
        resolve_url_from_content_path
            .as_ref()
            .and_then(ResolveUrlFromContentPath::override_root_url)
            .map(ToOwned::to_owned),
    )?;
    let collection_id = collection_ctx.record_id;
    let content_path_resolver = if resolve_url_from_content_path.is_some() {
        if let Some(resolver) = collection_ctx.content_path.resolver {
            Some(resolver)
        } else {
            let path_kind = collection_ctx.content_path.kind;
            return Err(Error::Other(anyhow!(
                "unsupported path kind: {path_kind:?}"
            )));
        }
    } else {
        None
    };
    find_unsynchronized(
        repo,
        collection_id,
        pagination,
        content_path_predicate,
        content_path_resolver.as_ref(),
    )
    .map_err(Into::into)
}
