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

use std::{
    fs::read_dir,
    sync::atomic::{AtomicBool, Ordering},
};

use url::Url;

use aoide_core::{
    entity::EntityUid,
    media::content::{
        resolver::{ContentPathResolver as _, VirtualFilePathResolver},
        ContentPath,
    },
    util::clock::DateTime,
};

use aoide_core_api::{media::SyncMode, track::replace::Summary};

use aoide_media::io::import::{ImportTrackConfig, Issues};

use aoide_repo::{
    collection::{EntityRepo as CollectionRepo, RecordId as CollectionId},
    media::source::RecordId as MediaSourceId,
    track::{CollectionRepo as TrackCollectionRepo, ReplaceMode, ReplaceParams},
};

use crate::{
    collection::vfs::RepoContext,
    media::{import_track_from_file_path, ImportTrackFromFileOutcome, SyncModeParams},
};

use super::{
    replace::{Completion, Outcome},
    *,
};

// TODO: Reduce number of arguments
#[allow(clippy::too_many_arguments)]
pub fn import_and_replace_from_file_path<Repo>(
    summary: &mut Summary,
    visited_media_source_ids: &mut Vec<MediaSourceId>,
    imported_media_sources_with_issues: &mut Vec<(MediaSourceId, ContentPath, Issues)>,
    repo: &Repo,
    collection_id: CollectionId,
    content_path_resolver: &VirtualFilePathResolver,
    content_path: ContentPath,
    params: &Params,
) -> Result<Vec<TrackInvalidity>>
where
    Repo: TrackCollectionRepo,
{
    let (media_source_id, external_rev, synchronized_rev, entity_body) = repo
        .load_track_entity_by_media_source_content_path(collection_id, &content_path)
        .optional()?
        .map(|(media_source_id, _, entity)| {
            (
                Some(media_source_id),
                entity.body.track.media_source.content_link.rev,
                entity.body.last_synchronized_rev.map(|rev| {
                    debug_assert!(rev <= entity.hdr.rev);
                    rev == entity.hdr.rev
                }),
                Some(entity.body),
            )
        })
        .unwrap_or((None, None, None, None));
    let Params {
        sync_mode,
        import_config,
        replace_mode,
    } = params;
    let replace_params = ReplaceParams {
        mode: *replace_mode,
        preserve_collected_at: true,
        update_last_synchronized_rev: true,
    };
    let mut invalidities = Default::default();
    match import_track_from_file_path(
        content_path_resolver,
        content_path.clone(),
        SyncModeParams::new(*sync_mode, external_rev, synchronized_rev),
        import_config,
        DateTime::now_local_or_utc(),
    ) {
        Ok(ImportTrackFromFileOutcome::Imported {
            track: imported_track,
            issues: import_issues,
        }) => {
            debug_assert_eq!(imported_track.media_source.content_link.path, content_path);
            let track = if let Some(mut collected_track) =
                entity_body.map(|entity_body| entity_body.track)
            {
                // Merge imported properties into existing properties, i.e.
                // keep existing properties if no replacement is available.
                collected_track.merge_newer_from_synchronized_media_source(imported_track);
                collected_track
            } else {
                imported_track
            };
            let (track, invalidities_from_input_validation) = validate_input(track)?;
            invalidities = invalidities_from_input_validation;
            if !invalidities.is_empty() {
                log::debug!("{:?} has invalidities: {:?}", track.0, invalidities);
            }
            if let Some(media_source_id) =
                super::replace::replace_collected_track_by_media_source_content_path(
                    summary,
                    repo,
                    collection_id,
                    replace_params,
                    track,
                )?
            {
                visited_media_source_ids.push(media_source_id);
                if !import_issues.is_empty() {
                    imported_media_sources_with_issues.push((
                        media_source_id,
                        content_path,
                        import_issues,
                    ));
                }
            }
        }
        Ok(ImportTrackFromFileOutcome::SkippedSynchronized { content_rev: _ }) => {
            debug_assert!(media_source_id.is_some());
            summary.unchanged.push(content_path);
            visited_media_source_ids.push(media_source_id.unwrap());
        }
        Ok(ImportTrackFromFileOutcome::SkippedUnsynchronized { content_rev: _ }) => {
            debug_assert!(media_source_id.is_some());
            debug_assert_eq!(Some(false), synchronized_rev);
            summary.not_imported.push(content_path);
            visited_media_source_ids.push(media_source_id.unwrap());
        }
        Ok(ImportTrackFromFileOutcome::SkippedDirectory) => {
            // Nothing to do
        }
        Err(err) => match err {
            Error::Media(MediaError::UnknownContentType)
            | Error::Media(MediaError::UnsupportedContentType(_)) => {
                log::info!(
                    "Skipped import of track from local file path {}: {}",
                    content_path_resolver
                        .build_file_path(&content_path)
                        .display(),
                    err
                );
                summary.skipped.push(content_path);
            }
            err => {
                log::warn!(
                    "Failed to import track from local file path {}: {}",
                    content_path_resolver
                        .build_file_path(&content_path)
                        .display(),
                    err
                );
                summary.failed.push(content_path);
            }
        },
    };
    Ok(invalidities)
}

const DEFAULT_MEDIA_SOURCE_COUNT: usize = 1024;

#[derive(Debug, Clone)]
pub struct Params {
    pub sync_mode: SyncMode,
    pub import_config: ImportTrackConfig,
    pub replace_mode: ReplaceMode,
}

pub fn import_and_replace_many_by_local_file_path<Repo>(
    repo: &Repo,
    collection_uid: &EntityUid,
    params: &Params,
    content_paths: impl IntoIterator<Item = ContentPath>,
    expected_content_path_count: Option<usize>,
    abort_flag: &AtomicBool,
) -> Result<Outcome>
where
    Repo: CollectionRepo + TrackCollectionRepo,
{
    let collection_ctx = RepoContext::resolve(repo, collection_uid, None)?;
    let vfs_ctx = if let Some(vfs_ctx) = &collection_ctx.content_path.vfs {
        vfs_ctx
    } else {
        return Err(anyhow::anyhow!(
            "Unsupported path kind: {:?}",
            collection_ctx.content_path.kind
        )
        .into());
    };
    let collection_id = collection_ctx.record_id;
    let mut summary = Summary::default();
    let mut visited_media_source_ids =
        Vec::with_capacity(expected_content_path_count.unwrap_or(DEFAULT_MEDIA_SOURCE_COUNT));
    let mut imported_media_sources_with_issues =
        Vec::with_capacity(expected_content_path_count.unwrap_or(DEFAULT_MEDIA_SOURCE_COUNT) / 4);
    for content_path in content_paths {
        if abort_flag.load(Ordering::Relaxed) {
            log::debug!("Aborting import of {}", content_path);
            return Ok(Outcome {
                completion: Completion::Aborted,
                summary,
                visited_media_source_ids,
                imported_media_sources_with_issues,
            });
        }
        let invalidities = import_and_replace_from_file_path(
            &mut summary,
            &mut visited_media_source_ids,
            &mut imported_media_sources_with_issues,
            repo,
            collection_id,
            &vfs_ctx.path_resolver,
            content_path,
            params,
        )?;
        if !invalidities.is_empty() {
            imported_media_sources_with_issues
                .last_mut()
                .unwrap()
                .2
                .add_message(format!("Track invalidities: {:?}", invalidities));
        }
    }
    Ok(Outcome {
        completion: Completion::Finished,
        summary,
        visited_media_source_ids,
        imported_media_sources_with_issues,
    })
}

const EXPECTED_NUMBER_OF_DIR_ENTRIES: usize = 1024;

pub fn import_and_replace_by_local_file_path_from_directory<Repo>(
    repo: &Repo,
    collection_uid: &EntityUid,
    params: &Params,
    source_dir_path: &str,
    abort_flag: &AtomicBool,
) -> Result<Outcome>
where
    Repo: CollectionRepo + TrackCollectionRepo,
{
    let collection_ctx = RepoContext::resolve(repo, collection_uid, None)?;
    let vfs_ctx = if let Some(vfs_ctx) = &collection_ctx.content_path.vfs {
        vfs_ctx
    } else {
        return Err(anyhow::anyhow!(
            "Unsupported path kind: {:?}",
            collection_ctx.content_path.kind
        )
        .into());
    };
    let collection_id = collection_ctx.record_id;
    import_and_replace_by_local_file_path_from_directory_with_content_path_resolver(
        repo,
        collection_id,
        &vfs_ctx.path_resolver,
        params,
        source_dir_path,
        abort_flag,
    )
}

pub fn import_and_replace_by_local_file_path_from_directory_with_content_path_resolver(
    repo: &impl TrackCollectionRepo,
    collection_id: CollectionId,
    content_path_resolver: &VirtualFilePathResolver,
    params: &Params,
    source_dir_path: &str,
    abort_flag: &AtomicBool,
) -> Result<Outcome> {
    let dir_path = content_path_resolver.build_file_path(source_dir_path);
    log::debug!("Importing files from directory: {}", dir_path.display());
    let dir_entries = read_dir(dir_path)?;
    let mut summary = Summary::default();
    let mut visited_media_source_ids = Vec::with_capacity(EXPECTED_NUMBER_OF_DIR_ENTRIES);
    let mut imported_media_sources_with_issues =
        Vec::with_capacity(EXPECTED_NUMBER_OF_DIR_ENTRIES / 4);
    for dir_entry in dir_entries {
        let dir_entry = match dir_entry {
            Ok(dir_entry) => dir_entry,
            Err(err) => {
                log::warn!("Failed to access directory entry: {}", err);
                // Skip entry and keep going
                continue;
            }
        };
        if abort_flag.load(Ordering::Relaxed) {
            log::debug!(
                "Aborting import before visiting {}",
                dir_entry.path().display()
            );
            return Ok(Outcome {
                completion: Completion::Aborted,
                summary,
                visited_media_source_ids,
                imported_media_sources_with_issues,
            });
        }
        let content_path = if let Some(content_path) = Url::from_file_path(dir_entry.path())
            .ok()
            .and_then(|url| content_path_resolver.resolve_path_from_url(&url).ok())
        {
            content_path.to_owned()
        } else {
            log::warn!(
                "Skipping invalid/unsupported directory entry: {}",
                dir_entry.path().display()
            );
            // Skip entry and keep going
            continue;
        };
        import_and_replace_from_file_path(
            &mut summary,
            &mut visited_media_source_ids,
            &mut imported_media_sources_with_issues,
            repo,
            collection_id,
            content_path_resolver,
            content_path,
            params,
        )?;
    }
    Ok(Outcome {
        completion: Completion::Finished,
        summary,
        visited_media_source_ids,
        imported_media_sources_with_issues,
    })
}