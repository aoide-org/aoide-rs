// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use aoide_core::{entity::EntityUid, media::SourcePath, util::clock::DateTime};

use aoide_core_api::{media::SyncMode, track::replace::Summary};

use aoide_media::{
    io::import::ImportTrackConfig,
    resolver::{SourcePathResolver, VirtualFilePathResolver},
};

use aoide_repo::{
    collection::{EntityRepo as CollectionRepo, RecordId as CollectionId},
    media::source::RecordId as MediaSourceId,
    track::{EntityRepo, ReplaceMode, ReplaceOutcome},
};

use crate::{
    collection::vfs::{RepoContext, SourcePathContext},
    media::{import_track_from_file_path, ImportTrackFromFileOutcome, SyncStatus},
};

use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Params {
    pub mode: ReplaceMode,

    /// Consider the `path` as an URL and resolve it according
    /// the collection's media source configuration.
    ///
    /// The default value is `false`.
    pub resolve_path_from_url: bool,

    /// Preserve the `collected_at` property of existing media
    /// sources and don't update it.
    ///
    /// The default value is `true`.
    pub preserve_collected_at: bool,
}

impl Params {
    #[must_use]
    pub fn new(mode: ReplaceMode) -> Self {
        Self {
            mode,
            resolve_path_from_url: false,
            preserve_collected_at: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Completion {
    Finished,
    Aborted,
}

#[derive(Debug, Clone)]
pub struct Outcome {
    pub completion: Completion,
    pub summary: Summary,
    pub media_source_ids: Vec<MediaSourceId>,
}

pub fn replace_collected_track_by_media_source_path<Repo>(
    summary: &mut Summary,
    repo: &Repo,
    collection_id: CollectionId,
    replace_mode: ReplaceMode,
    preserve_collected_at: bool,
    track: ValidatedInput,
) -> Result<Option<MediaSourceId>>
where
    Repo: EntityRepo,
{
    let ValidatedInput(track) = track;
    let media_source_path = track.media_source.path.clone();
    let outcome = repo
        .replace_collected_track_by_media_source_path(
            collection_id,
            preserve_collected_at,
            replace_mode,
            track,
        )
        .map_err(|err| {
            log::warn!(
                "Failed to replace track by URI '{}': {}",
                media_source_path,
                err
            );
            err
        })?;
    let media_source_id = match outcome {
        ReplaceOutcome::Created(media_source_id, _, entity) => {
            debug_assert_ne!(ReplaceMode::UpdateOnly, replace_mode);
            log::trace!(
                "Created {}: {:?}",
                entity.body.media_source.path,
                entity.hdr
            );
            summary.created.push(entity);
            media_source_id
        }
        ReplaceOutcome::Updated(media_source_id, _, entity) => {
            debug_assert_ne!(ReplaceMode::CreateOnly, replace_mode);
            log::trace!(
                "Updated {}: {:?}",
                entity.body.media_source.path,
                entity.hdr
            );
            summary.updated.push(entity);
            media_source_id
        }
        ReplaceOutcome::Unchanged(media_source_id, _, entity) => {
            log::trace!("Unchanged: {:?}", entity);
            summary.unchanged.push(entity.body.media_source.path);
            media_source_id
        }
        ReplaceOutcome::NotCreated(track) => {
            debug_assert_eq!(ReplaceMode::UpdateOnly, replace_mode);
            log::trace!("Not created: {:?}", track);
            summary.not_created.push(track);
            return Ok(None);
        }
        ReplaceOutcome::NotUpdated(media_source_id, _, track) => {
            debug_assert_eq!(ReplaceMode::CreateOnly, replace_mode);
            log::trace!("Not updated: {:?}", track);
            summary.not_updated.push(track);
            media_source_id
        }
    };
    Ok(Some(media_source_id))
}

pub fn replace_collected_tracks_by_media_source_path<Repo>(
    repo: &Repo,
    collection_uid: &EntityUid,
    params: &Params,
    tracks: impl IntoIterator<Item = ValidatedInput>,
) -> Result<Summary>
where
    Repo: CollectionRepo + EntityRepo,
{
    let Params {
        mode: replace_mode,
        resolve_path_from_url,
        preserve_collected_at,
    } = params;
    let (collection_id, source_path_resolver) = if *resolve_path_from_url {
        let RepoContext {
            record_id,
            source_path: SourcePathContext { kind: _, vfs },
        } = RepoContext::resolve(repo, collection_uid, None)?;
        (record_id, vfs.map(|vfs| vfs.path_resolver))
    } else {
        let collection_id = repo.resolve_collection_id(collection_uid)?;
        (collection_id, None)
    };
    let mut summary = Summary::default();
    for track in tracks {
        let ValidatedInput(mut track) = track;
        if let Some(source_path_resolver) = source_path_resolver.as_ref() {
            let url = track
                .media_source
                .path
                .parse()
                .map_err(|err| {
                    anyhow::anyhow!(
                        "Failed to parse URL from path '{}': {}",
                        track.media_source.path,
                        err
                    )
                })
                .map_err(Error::from)?;
            track.media_source.path = source_path_resolver
                .resolve_path_from_url(&url)
                .map_err(|err| {
                    anyhow::anyhow!(
                        "Failed to resolve local file path from URL '{}': {}",
                        url,
                        err
                    )
                })
                .map_err(Error::from)?;
        }
        replace_collected_track_by_media_source_path(
            &mut summary,
            repo,
            collection_id,
            *replace_mode,
            *preserve_collected_at,
            ValidatedInput(track),
        )?;
    }
    Ok(summary)
}

// TODO: Reduce number of arguments
#[allow(clippy::too_many_arguments)]
pub fn import_and_replace_from_file_path<Repo>(
    summary: &mut Summary,
    media_source_ids: &mut Vec<MediaSourceId>,
    repo: &Repo,
    collection_id: CollectionId,
    source_path_resolver: &VirtualFilePathResolver,
    sync_mode: SyncMode,
    import_config: &ImportTrackConfig,
    replace_mode: ReplaceMode,
    source_path: SourcePath,
) -> Result<()>
where
    Repo: EntityRepo,
{
    let (media_source_id, last_synchronized_at, collected_track) = repo
        .load_collected_track_entity_by_media_source_path(collection_id, &source_path)
        .optional()?
        .map(|(media_source_id, _, entity)| {
            (
                Some(media_source_id),
                entity.body.media_source.synchronized_at,
                Some(entity.body),
            )
        })
        .unwrap_or((None, None, None));
    match import_track_from_file_path(
        source_path_resolver,
        source_path.clone(),
        SyncStatus::new(sync_mode, last_synchronized_at),
        import_config,
        DateTime::now_local(),
    ) {
        Ok(ImportTrackFromFileOutcome::Imported(imported_track)) => {
            debug_assert_eq!(imported_track.media_source.path, source_path);
            let track = if let Some(mut collected_track) = collected_track {
                collected_track.merge_newer_from_synchronized_media_source(imported_track);
                collected_track
            } else {
                imported_track
            };
            let track = validate_input(track)?;
            if let Some(media_source_id) = replace_collected_track_by_media_source_path(
                summary,
                repo,
                collection_id,
                replace_mode,
                true,
                track,
            )? {
                media_source_ids.push(media_source_id);
            }
        }
        Ok(ImportTrackFromFileOutcome::SkippedSynchronized(_synchronized_at)) => {
            debug_assert!(media_source_id.is_some());
            debug_assert!(last_synchronized_at.is_some());
            debug_assert!(_synchronized_at <= last_synchronized_at.unwrap());
            summary.unchanged.push(source_path);
            media_source_ids.push(media_source_id.unwrap());
        }
        Ok(ImportTrackFromFileOutcome::SkippedDirectory) => {
            // Nothing to do
        }
        Err(err) => match err {
            Error::Media(MediaError::UnknownContentType)
            | Error::Media(MediaError::UnsupportedContentType(_)) => {
                log::info!(
                    "Skipped import of track from local file path {}: {}",
                    source_path_resolver.build_file_path(&source_path).display(),
                    err
                );
                summary.skipped.push(source_path);
            }
            err => {
                log::warn!(
                    "Failed to import track from local file path {}: {}",
                    source_path_resolver.build_file_path(&source_path).display(),
                    err
                );
                summary.failed.push(source_path);
            }
        },
    };
    Ok(())
}

pub fn replace_by_media_source_path(
    repo: &impl EntityRepo,
    collection_id: CollectionId,
    replace_mode: ReplaceMode,
    tracks: impl IntoIterator<Item = ValidatedInput>,
) -> Result<Summary> {
    let mut summary = Summary::default();
    for track in tracks {
        replace_collected_track_by_media_source_path(
            &mut summary,
            repo,
            collection_id,
            replace_mode,
            true,
            track,
        )?;
    }
    Ok(summary)
}

const DEFAULT_MEDIA_SOURCE_COUNT: usize = 1024;

// TODO: Reduce number of arguments
#[allow(clippy::too_many_arguments)]
pub fn import_and_replace_by_local_file_paths<Repo>(
    repo: &Repo,
    collection_uid: &EntityUid,
    sync_mode: SyncMode,
    import_config: &ImportTrackConfig,
    replace_mode: ReplaceMode,
    source_paths: impl IntoIterator<Item = SourcePath>,
    expected_source_path_count: Option<usize>,
    abort_flag: &AtomicBool,
) -> Result<Outcome>
where
    Repo: CollectionRepo + EntityRepo,
{
    let collection_ctx = RepoContext::resolve(repo, collection_uid, None)?;
    let vfs_ctx = if let Some(vfs_ctx) = &collection_ctx.source_path.vfs {
        vfs_ctx
    } else {
        return Err(anyhow::anyhow!(
            "Unsupported path kind: {:?}",
            collection_ctx.source_path.kind
        )
        .into());
    };
    let collection_id = collection_ctx.record_id;
    let mut summary = Summary::default();
    let mut media_source_ids =
        Vec::with_capacity(expected_source_path_count.unwrap_or(DEFAULT_MEDIA_SOURCE_COUNT));
    for source_path in source_paths {
        if abort_flag.load(Ordering::Relaxed) {
            log::debug!("Aborting import of {}", source_path);
            return Ok(Outcome {
                completion: Completion::Aborted,
                summary,
                media_source_ids,
            });
        }
        import_and_replace_from_file_path(
            &mut summary,
            &mut media_source_ids,
            repo,
            collection_id,
            &vfs_ctx.path_resolver,
            sync_mode,
            import_config,
            replace_mode,
            source_path,
        )?;
    }
    Ok(Outcome {
        completion: Completion::Finished,
        summary,
        media_source_ids,
    })
}

const EXPECTED_NUMBER_OF_DIR_ENTRIES: usize = 1024;

pub fn import_and_replace_by_local_file_path_from_directory<Repo>(
    repo: &Repo,
    collection_uid: &EntityUid,
    sync_mode: SyncMode,
    import_config: &ImportTrackConfig,
    replace_mode: ReplaceMode,
    source_dir_path: &str,
    abort_flag: &AtomicBool,
) -> Result<Outcome>
where
    Repo: CollectionRepo + EntityRepo,
{
    let collection_ctx = RepoContext::resolve(repo, collection_uid, None)?;
    let vfs_ctx = if let Some(vfs_ctx) = &collection_ctx.source_path.vfs {
        vfs_ctx
    } else {
        return Err(anyhow::anyhow!(
            "Unsupported path kind: {:?}",
            collection_ctx.source_path.kind
        )
        .into());
    };
    let collection_id = collection_ctx.record_id;
    import_and_replace_by_local_file_path_from_directory_with_source_path_resolver(
        repo,
        collection_id,
        &vfs_ctx.path_resolver,
        sync_mode,
        import_config,
        replace_mode,
        source_dir_path,
        abort_flag,
    )
}

// TODO: Reduce number of arguments
#[allow(clippy::too_many_arguments)]
pub fn import_and_replace_by_local_file_path_from_directory_with_source_path_resolver(
    repo: &impl EntityRepo,
    collection_id: CollectionId,
    source_path_resolver: &VirtualFilePathResolver,
    sync_mode: SyncMode,
    import_config: &ImportTrackConfig,
    replace_mode: ReplaceMode,
    source_dir_path: &str,
    abort_flag: &AtomicBool,
) -> Result<Outcome> {
    let dir_path = source_path_resolver.build_file_path(source_dir_path);
    log::debug!("Importing files from directory: {}", dir_path.display());
    let dir_entries = read_dir(dir_path)?;
    let mut summary = Summary::default();
    let mut media_source_ids = Vec::with_capacity(EXPECTED_NUMBER_OF_DIR_ENTRIES);
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
                media_source_ids,
            });
        }
        let source_path = if let Some(source_path) = Url::from_file_path(dir_entry.path())
            .ok()
            .and_then(|url| source_path_resolver.resolve_path_from_url(&url).ok())
        {
            source_path.to_owned()
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
            &mut media_source_ids,
            repo,
            collection_id,
            source_path_resolver,
            sync_mode,
            import_config,
            replace_mode,
            source_path,
        )?;
    }
    Ok(Outcome {
        completion: Completion::Finished,
        summary,
        media_source_ids,
    })
}
