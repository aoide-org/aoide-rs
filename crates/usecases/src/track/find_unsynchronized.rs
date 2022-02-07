// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use aoide_core::{entity::EntityUid, media::SourcePath};
use aoide_core_api::{media::source::ResolveUrlFromPath, track::find_unsynchronized::*};

use aoide_repo::{
    collection::{EntityRepo as CollectionRepo, RecordId as CollectionId},
    track::{CollectionRepo as TrackCollectionRepo, RecordTrail},
};

use crate::collection::vfs::RepoContext;

use super::*;

pub fn find_unsynchronized<Repo>(
    repo: &Repo,
    collection_id: CollectionId,
    pagination: &Pagination,
    media_source_path_predicate: Option<StringPredicateBorrowed<'_>>,
    source_path_resolver: Option<&VirtualFilePathResolver>,
) -> RepoResult<Vec<UnsynchronizedTrackEntity>>
where
    Repo: TrackCollectionRepo,
{
    repo.find_unsynchronized_tracks(collection_id, pagination, media_source_path_predicate)
        .map(|v| {
            v.into_iter()
                .map(|(entity_header, _record_id, record_trail)| {
                    let RecordTrail {
                        collection_id: _,
                        media_source_id: _,
                        media_source_path,
                        media_source_external_rev,
                        media_source_synchronized_rev,
                    } = record_trail;
                    let mut media_source_path = media_source_path;
                    if let Some(source_path_resolver) = source_path_resolver {
                        // FIXME: Handle errors
                        let url = source_path_resolver
                            .resolve_url_from_path(&media_source_path)
                            .unwrap();
                        media_source_path = SourcePath::new(url.to_string());
                    }
                    let media_source = UnsynchronizedMediaSource {
                        path: media_source_path,
                        external_rev: media_source_external_rev,
                    };
                    let track = UnsynchronizedTrack {
                        media_source,
                        media_source_synchronized_rev,
                    };
                    UnsynchronizedTrackEntity::new(entity_header, track)
                })
                .collect()
        })
}

pub fn find_unsynchronized_with_params<Repo>(
    repo: &Repo,
    collection_uid: &EntityUid,
    params: Params,
    pagination: &Pagination,
) -> Result<Vec<UnsynchronizedTrackEntity>>
where
    Repo: CollectionRepo + TrackCollectionRepo,
{
    let Params {
        resolve_url_from_path,
        media_source_path_predicate,
    } = params;
    let collection_ctx = RepoContext::resolve_ext(
        repo,
        collection_uid,
        None,
        resolve_url_from_path
            .as_ref()
            .and_then(ResolveUrlFromPath::override_root_url)
            .map(ToOwned::to_owned),
    )?;
    let collection_id = collection_ctx.record_id;
    let source_path_resolver = if resolve_url_from_path.is_some() {
        if let Some(vfs_ctx) = collection_ctx.source_path.vfs {
            Some(vfs_ctx.path_resolver)
        } else {
            return Err(anyhow::anyhow!(
                "Unsupported path kind: {:?}",
                collection_ctx.source_path.kind
            )
            .into());
        }
    } else {
        None
    };
    find_unsynchronized(
        repo,
        collection_id,
        pagination,
        media_source_path_predicate
            .as_ref()
            .map(StringPredicate::borrow),
        source_path_resolver.as_ref(),
    )
    .map_err(Into::into)
}
