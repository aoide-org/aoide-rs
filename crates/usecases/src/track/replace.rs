// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::track::Entity as TrackEntity;
use aoide_core_api::track::replace::Summary;
use aoide_repo::{
    collection::RecordId as CollectionId,
    media::source::RecordId as MediaSourceId,
    track::{CollectionRepo as TrackCollectionRepo, ReplaceMode, ReplaceOutcome, ReplaceParams},
};

use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
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

    /// Decode gig tags
    ///
    /// Decode all custom tags that are not supported as native file tags
    /// from the "cgrp" (content group/grouping) tag when creating/updating
    /// tags.
    ///
    /// This options is useful for interoperability with applications that
    /// only support the common file tags and for file synchronization.
    pub decode_gigtags: bool,
}

#[derive(Debug)]
pub enum Outcome {
    NotCreated(Track),
    NotUpdated(MediaSourceId, Track),
    Unchanged(MediaSourceId, TrackEntity),
    Created(MediaSourceId, TrackEntity),
    Updated(MediaSourceId, TrackEntity),
}

impl Outcome {
    pub fn update_summary(self, summary: &mut Summary) -> Option<MediaSourceId> {
        match self {
            Self::Unchanged(media_source_id, entity) => {
                log::trace!("Unchanged: {entity:?}");
                let (_, body) = entity.into();
                summary
                    .unchanged
                    .push(body.track.media_source.content.link.path);
                Some(media_source_id)
            }
            Self::Created(media_source_id, entity) => {
                log::trace!(
                    "Created {path}: {hdr:?}",
                    path = entity.body.track.media_source.content.link.path,
                    hdr = entity.hdr
                );
                summary.created.push(entity);
                Some(media_source_id)
            }
            Self::Updated(media_source_id, entity) => {
                log::trace!(
                    "Updated {path}: {hdr:?}",
                    path = entity.body.track.media_source.content.link.path,
                    hdr = entity.hdr
                );
                summary.updated.push(entity);
                Some(media_source_id)
            }
            Self::NotCreated(track) => {
                log::trace!("Not created: {track:?}");
                summary.not_created.push(track);
                None
            }
            Self::NotUpdated(media_source_id, track) => {
                log::trace!("Not updated: {track:?}");
                summary.not_updated.push(track);
                Some(media_source_id)
            }
        }
    }
}

pub fn replace_collected_track_by_media_source_content_path<Repo>(
    repo: &mut Repo,
    collection_id: CollectionId,
    params: ReplaceParams,
    track: ValidatedInput,
) -> Result<Outcome>
where
    Repo: TrackCollectionRepo,
{
    let ValidatedInput(track) = track;
    let media_content_path = track.media_source.content.link.path.clone();
    let outcome = repo
        .replace_track_by_media_source_content_path(collection_id, params, track)
        .map_err(|err| {
            log::warn!("Failed to replace track by URI '{media_content_path}': {err}");
            err
        })?;
    let completion = match outcome {
        ReplaceOutcome::Unchanged(media_source_id, _, entity) => {
            Outcome::Unchanged(media_source_id, entity)
        }
        ReplaceOutcome::Created(media_source_id, _, entity) => {
            debug_assert_ne!(ReplaceMode::UpdateOnly, params.mode);
            Outcome::Created(media_source_id, entity)
        }
        ReplaceOutcome::Updated(media_source_id, _, entity) => {
            debug_assert_ne!(ReplaceMode::CreateOnly, params.mode);
            Outcome::Updated(media_source_id, entity)
        }
        ReplaceOutcome::NotCreated(track) => {
            debug_assert_eq!(ReplaceMode::UpdateOnly, params.mode);
            Outcome::NotCreated(track)
        }
        ReplaceOutcome::NotUpdated(media_source_id, _, track) => {
            debug_assert_eq!(ReplaceMode::CreateOnly, params.mode);
            Outcome::NotUpdated(media_source_id, track)
        }
    };
    Ok(completion)
}

#[cfg(all(feature = "media-file", not(target_family = "wasm")))]
pub fn replace_many_by_media_source_content_path<Repo>(
    repo: &mut Repo,
    collection_uid: &CollectionUid,
    params: &Params,
    validated_track_iter: impl IntoIterator<Item = ValidatedInput>,
) -> Result<Summary>
where
    Repo: aoide_repo::collection::EntityRepo + TrackCollectionRepo,
{
    use anyhow::anyhow;
    use aoide_core::{
        media::content::resolver::ContentPathResolver as _,
        {tag::TagsMap, track::tag::FACET_ID_GROUPING},
    };

    use crate::collection::vfs::{ContentPathContext, RepoContext};

    let Params {
        mode: replace_mode,
        resolve_path_from_url,
        decode_gigtags,
        preserve_collected_at,
        update_last_synchronized_rev,
    } = params;
    let (collection_id, content_path_resolver) = if *resolve_path_from_url {
        let RepoContext {
            record_id,
            content_path: ContentPathContext { resolver, .. },
        } = RepoContext::resolve(repo, collection_uid, None)?;
        (record_id, resolver)
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
                .content
                .link
                .path
                .as_str()
                .parse()
                .map_err(|err| {
                    Error::Other(anyhow!(
                        "failed to parse URL from path '{path}': {err}",
                        path = track.media_source.content.link.path,
                    ))
                })?;
            track.media_source.content.link.path = content_path_resolver
                .resolve_path_from_url(&url)
                .map_err(|err| {
                    Error::Other(anyhow!(
                        "failed to resolve local file path from URL '{url}': {err}"
                    ))
                })?
                .ok_or_else(|| {
                    Error::Other(anyhow!(
                        "failed to resolve local file path from URL '{url}'"
                    ))
                })?;
        }
        if *decode_gigtags {
            let mut tags_map: TagsMap<'static> = track.tags.untie().into();
            if let Some(faceted_tags) = tags_map.take_faceted_tags(FACET_ID_GROUPING) {
                let decoded_gig_tags =
                    aoide_media_file::util::gigtag::import_from_faceted_tags(faceted_tags);
                tags_map.merge(decoded_gig_tags);
            }
            track.tags = tags_map.canonicalize_into();
        }
        let outcome = replace_collected_track_by_media_source_content_path(
            repo,
            collection_id,
            ReplaceParams {
                mode: *replace_mode,
                preserve_collected_at: *preserve_collected_at,
                update_last_synchronized_rev: *update_last_synchronized_rev,
            },
            ValidatedInput(track),
        )?;
        outcome.update_summary(&mut summary);
    }
    Ok(summary)
}
