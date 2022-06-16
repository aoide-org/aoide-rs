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

use aoide_core_api::track::replace::Summary;

use aoide_repo::{
    collection::RecordId as CollectionId,
    media::source::RecordId as MediaSourceId,
    track::{CollectionRepo as TrackCollectionRepo, ReplaceMode, ReplaceOutcome, ReplaceParams},
};

#[cfg(not(target_family = "wasm"))]
use aoide_core::media::content::resolver::ContentPathResolver as _;

#[cfg(not(target_family = "wasm"))]
use crate::collection::vfs::{ContentPathContext, RepoContext};

use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Params {
    pub mode: ReplaceMode,

    /// Consider the `path` as an URL and resolve it according
    /// the collection's media source configuration.
    pub resolve_path_from_url: bool,

    /// Preserve the `collected_at` property of existing media
    /// sources and don't update it.
    pub preserve_collected_at: bool,

    /// Set or update the synchronized revision if the media source
    /// has a synchronization time stamp
    pub update_last_synchronized_rev: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Completion {
    Finished,
    Aborted,
}

pub fn replace_collected_track_by_media_source_content_path<Repo>(
    summary: &mut Summary,
    repo: &Repo,
    collection_id: CollectionId,
    params: ReplaceParams,
    track: ValidatedInput,
) -> Result<Option<MediaSourceId>>
where
    Repo: TrackCollectionRepo,
{
    let ValidatedInput(track) = track;
    let media_content_path = track.media_source.content_link.path.clone();
    let outcome = repo
        .replace_track_by_media_source_content_path(collection_id, params, track)
        .map_err(|err| {
            log::warn!("Failed to replace track by URI '{media_content_path}': {err}");
            err
        })?;
    let media_source_id = match outcome {
        ReplaceOutcome::Created(media_source_id, _, entity) => {
            debug_assert_ne!(ReplaceMode::UpdateOnly, params.mode);
            log::trace!(
                "Created {}: {:?}",
                entity.body.track.media_source.content_link.path,
                entity.hdr
            );
            summary.created.push(entity);
            media_source_id
        }
        ReplaceOutcome::Updated(media_source_id, _, entity) => {
            debug_assert_ne!(ReplaceMode::CreateOnly, params.mode);
            log::trace!(
                "Updated {}: {:?}",
                entity.body.track.media_source.content_link.path,
                entity.hdr
            );
            summary.updated.push(entity);
            media_source_id
        }
        ReplaceOutcome::Unchanged(media_source_id, _, entity) => {
            log::trace!("Unchanged: {entity:?}");
            summary
                .unchanged
                .push(entity.raw.body.track.media_source.content_link.path);
            media_source_id
        }
        ReplaceOutcome::NotCreated(track) => {
            debug_assert_eq!(ReplaceMode::UpdateOnly, params.mode);
            log::trace!("Not created: {track:?}");
            summary.not_created.push(track);
            return Ok(None);
        }
        ReplaceOutcome::NotUpdated(media_source_id, _, track) => {
            debug_assert_eq!(ReplaceMode::CreateOnly, params.mode);
            log::trace!("Not updated: {track:?}");
            summary.not_updated.push(track);
            media_source_id
        }
    };
    Ok(Some(media_source_id))
}

#[cfg(not(target_family = "wasm"))]
pub fn replace_many_by_media_source_content_path<Repo>(
    repo: &Repo,
    collection_uid: &CollectionUid,
    params: &Params,
    validated_track_iter: impl IntoIterator<Item = ValidatedInput>,
) -> Result<Summary>
where
    Repo: aoide_repo::collection::EntityRepo + TrackCollectionRepo,
{
    let Params {
        mode: replace_mode,
        resolve_path_from_url,
        preserve_collected_at,
        update_last_synchronized_rev,
    } = params;
    let (collection_id, content_path_resolver) = if *resolve_path_from_url {
        let RepoContext {
            record_id,
            content_path: ContentPathContext { kind: _, vfs },
        } = RepoContext::resolve(repo, collection_uid, None)?;
        (record_id, vfs.map(|vfs| vfs.path_resolver))
    } else {
        let collection_id = repo.resolve_collection_id(collection_uid)?;
        (collection_id, None)
    };
    let mut summary = Summary::default();
    for validated_track in validated_track_iter {
        let ValidatedInput(mut track) = validated_track;
        if let Some(content_path_resolver) = content_path_resolver.as_ref() {
            let url = track
                .media_source
                .content_link
                .path
                .parse()
                .map_err(|err| {
                    anyhow::anyhow!(
                        "Failed to parse URL from path '{}': {err}",
                        track.media_source.content_link.path,
                    )
                })
                .map_err(Error::from)?;
            track.media_source.content_link.path = content_path_resolver
                .resolve_path_from_url(&url)
                .map_err(|err| {
                    anyhow::anyhow!("Failed to resolve local file path from URL '{url}': {err}")
                })
                .map_err(Error::from)?;
        }
        replace_collected_track_by_media_source_content_path(
            &mut summary,
            repo,
            collection_id,
            ReplaceParams {
                mode: *replace_mode,
                preserve_collected_at: *preserve_collected_at,
                update_last_synchronized_rev: *update_last_synchronized_rev,
            },
            ValidatedInput(track),
        )?;
    }
    Ok(summary)
}
