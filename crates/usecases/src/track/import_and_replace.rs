// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    fs::read_dir,
    sync::atomic::{AtomicBool, Ordering},
};

use aoide_core::{
    media::content::{
        resolver::{vfs::VfsResolver, ContentPathResolver as _},
        ContentPath, ContentRevision,
    },
    util::clock::OffsetDateTimeMs,
};
use aoide_core_api::{
    media::{tracker::Completion, SyncMode},
    track::replace::Summary,
};
use aoide_media_file::io::import::{ImportTrack, ImportTrackConfig, Issues};
use aoide_repo::{
    collection::{EntityRepo as CollectionRepo, RecordId as CollectionId},
    media::source::RecordId as MediaSourceId,
    track::{CollectionRepo as TrackCollectionRepo, ReplaceMode, ReplaceParams},
};
use url::Url;

use super::*;
use crate::{
    collection::vfs::RepoContext,
    media::{import_track_from_file_path, ImportTrackFromFileOutcome, SyncModeParams},
};

#[derive(Debug, Clone)]
pub struct Outcome {
    pub completion: Completion,
    pub summary: Summary,
    pub visited_media_source_ids: Vec<MediaSourceId>,
    pub imported_media_sources_with_issues: Vec<(MediaSourceId, ContentPath<'static>, Issues)>,
}

#[derive(Debug)]
struct ImportReplacementFromFilePathContext {
    collection_id: CollectionId,
    content_path: ContentPath<'static>,
    media_source_id: Option<MediaSourceId>,
    external_rev: Option<ContentRevision>,
    synchronized_rev: Option<bool>,
    import_track: ImportTrack,
}

impl ImportReplacementFromFilePathContext {
    fn load_from_repo<Repo>(
        repo: &mut Repo,
        collection_id: CollectionId,
        content_path: ContentPath<'static>,
    ) -> RepoResult<Self>
    where
        Repo: TrackCollectionRepo,
    {
        let (media_source_id, external_rev, synchronized_rev, entity_body) = repo
            .load_track_entity_by_media_source_content_path(collection_id, &content_path)
            .optional()?
            .map_or((None, None, None, None), |(media_source_id, _, entity)| {
                (
                    Some(media_source_id),
                    entity.body.track.media_source.content.link.rev,
                    entity.body.last_synchronized_rev.map(|rev| {
                        debug_assert!(rev <= entity.hdr.rev);
                        rev == entity.hdr.rev
                    }),
                    Some(entity.raw.body),
                )
            });
        let import_track = entity_body.map_or_else(
            || ImportTrack::NewTrack {
                collected_at: OffsetDateTimeMs::now_local_or_utc(),
            },
            |entity_body| ImportTrack::UpdateTrack(entity_body.track),
        );
        Ok(Self {
            collection_id,
            content_path,
            media_source_id,
            external_rev,
            synchronized_rev,
            import_track,
        })
    }
}

#[derive(Debug)]
struct ImportReplacementFromFilePathOutcome {
    collection_id: CollectionId,
    content_path: ContentPath<'static>,
    import_issues: Issues,
    media_source_id: Option<MediaSourceId>,
    replacement: Replacement,
}

#[derive(Debug)]
enum Replacement {
    SkippedDirectory,
    SkippedFile(MediaFileError),
    Unchanged,
    NotImported,
    ImportFailed(Error),
    Replace {
        params: ReplaceParams,
        validated_input: ValidatedInput,
    },
}

fn import_replacement_from_file_path<InterceptImportedTrackFn>(
    context: ImportReplacementFromFilePathContext,
    content_path_resolver: &VfsResolver,
    params: &Params,
    intercept_imported_track_fn: &mut InterceptImportedTrackFn,
) -> Result<ImportReplacementFromFilePathOutcome>
where
    InterceptImportedTrackFn: FnMut(Track) -> Track,
{
    let ImportReplacementFromFilePathContext {
        collection_id,
        content_path,
        external_rev,
        import_track,
        media_source_id,
        synchronized_rev,
    } = context;
    let Params {
        sync_mode,
        import_config,
        replace_mode,
    } = params;
    let mut import_issues = Default::default();
    let replacement = match import_track_from_file_path(
        import_track,
        content_path_resolver,
        &content_path,
        &SyncModeParams::new(*sync_mode, external_rev, synchronized_rev),
        import_config,
    ) {
        Ok(ImportTrackFromFileOutcome::Imported { track, issues }) => {
            debug_assert_eq!(track.media_source.content.link.path, content_path);
            import_issues = issues;
            let track = intercept_imported_track_fn(track);
            let (validated_input, invalidities) = validate_input(track)?;
            // Merge low-level import issues with high-level invalidities.
            for invalidity in invalidities {
                import_issues.add_message(format!("{invalidity:?}"));
            }
            let params = ReplaceParams {
                mode: *replace_mode,
                preserve_collected_at: true,
                update_last_synchronized_rev: true,
            };
            Replacement::Replace {
                params,
                validated_input,
            }
        }
        Ok(ImportTrackFromFileOutcome::SkippedSynchronized { content_rev: _ }) => {
            debug_assert!(media_source_id.is_some());
            Replacement::Unchanged
        }
        Ok(ImportTrackFromFileOutcome::SkippedUnsynchronized { content_rev: _ }) => {
            debug_assert!(media_source_id.is_some());
            debug_assert_eq!(Some(false), synchronized_rev);
            Replacement::NotImported
        }
        Ok(ImportTrackFromFileOutcome::SkippedDirectory) => Replacement::SkippedDirectory,
        Err(err) => match err {
            Error::MediaFile(
                err @ (MediaFileError::UnknownContentType(_)
                | MediaFileError::UnsupportedContentType(_)),
            ) => Replacement::SkippedFile(err),
            err => Replacement::ImportFailed(err),
        },
    };
    Ok(ImportReplacementFromFilePathOutcome {
        collection_id,
        content_path,
        import_issues,
        media_source_id,
        replacement,
    })
}

#[derive(Debug)]
struct ImportAndReplaceFromFilePathOutcome {
    content_path: ContentPath<'static>,
    import_issues: Issues,
    media_source_id: Option<MediaSourceId>,
}

fn replace_after_imported_from_file_path<Repo>(
    outcome: ImportReplacementFromFilePathOutcome,
    summary: &mut Summary,
    repo: &mut Repo,
    content_path_resolver: &VfsResolver,
) -> Result<ImportAndReplaceFromFilePathOutcome>
where
    Repo: TrackCollectionRepo,
{
    let ImportReplacementFromFilePathOutcome {
        collection_id,
        content_path,
        import_issues,
        mut media_source_id,
        replacement,
    } = outcome;
    match replacement {
        Replacement::Replace {
            validated_input,
            params,
        } => {
            debug_assert_eq!(
                validated_input.0.media_source.content.link.path,
                content_path
            );
            let replaced_media_source_id =
                super::replace::replace_collected_track_by_media_source_content_path(
                    repo,
                    collection_id,
                    params,
                    validated_input,
                )?
                .update_summary(summary);
            debug_assert!(
                media_source_id.is_none()
                    || replaced_media_source_id.is_none()
                    || media_source_id == replaced_media_source_id
            );
            media_source_id = replaced_media_source_id;
        }
        Replacement::Unchanged => {
            debug_assert!(media_source_id.is_some());
            summary.unchanged.push(content_path.clone());
        }
        Replacement::NotImported => {
            debug_assert!(media_source_id.is_some());
            summary.not_imported.push(content_path.clone());
        }
        Replacement::SkippedFile(err) => {
            log::info!(
                "Skipped import of track from local file path {}: {err}",
                content_path_resolver
                    .build_file_path(&content_path)
                    .display()
            );
            summary.skipped.push(content_path.clone());
        }
        Replacement::ImportFailed(err) => {
            log::warn!(
                "Failed to import track from local file path {}: {err}",
                content_path_resolver
                    .build_file_path(&content_path)
                    .display()
            );
            summary.failed.push(content_path.clone());
        }
        Replacement::SkippedDirectory => {
            // Nothing to do
        }
    };
    Ok(ImportAndReplaceFromFilePathOutcome {
        content_path,
        import_issues,
        media_source_id,
    })
}

const DEFAULT_MEDIA_SOURCE_COUNT: usize = 1024;

#[derive(Debug, Clone)]
pub struct Params {
    pub sync_mode: SyncMode,
    pub import_config: ImportTrackConfig,
    pub replace_mode: ReplaceMode,
}

pub fn import_and_replace_many_by_local_file_path<Repo, InterceptImportedTrackFn>(
    repo: &mut Repo,
    collection_uid: &CollectionUid,
    params: &Params,
    content_paths: impl IntoIterator<Item = ContentPath<'static>>,
    expected_content_path_count: Option<usize>,
    intercept_imported_track_fn: &mut InterceptImportedTrackFn,
    abort_flag: &AtomicBool,
) -> Result<Outcome>
where
    Repo: CollectionRepo + TrackCollectionRepo,
    InterceptImportedTrackFn: FnMut(Track) -> Track,
{
    let collection_ctx = RepoContext::resolve(repo, collection_uid, None)?;
    let Some(resolver) = &collection_ctx.content_path.resolver else {
        let path_kind = collection_ctx.content_path.kind;
        return Err(anyhow::anyhow!("unsupported path kind: {path_kind:?}").into());
    };
    let collection_id = collection_ctx.record_id;
    let mut summary = Summary::default();
    let mut visited_media_source_ids =
        Vec::with_capacity(expected_content_path_count.unwrap_or(DEFAULT_MEDIA_SOURCE_COUNT));
    let mut imported_media_sources_with_issues =
        Vec::with_capacity(expected_content_path_count.unwrap_or(DEFAULT_MEDIA_SOURCE_COUNT) / 4);
    for content_path in content_paths {
        if abort_flag.load(Ordering::Relaxed) {
            log::debug!("Aborting import of {content_path}");
            return Ok(Outcome {
                completion: Completion::Aborted,
                summary,
                visited_media_source_ids,
                imported_media_sources_with_issues,
            });
        }

        let context = ImportReplacementFromFilePathContext::load_from_repo(
            repo,
            collection_id,
            content_path,
        )?;
        let content_path_resolver = resolver.canonical_resolver();

        // TODO: Import multiple tracks from different files in parallel.
        let outcome = import_replacement_from_file_path(
            context,
            content_path_resolver,
            params,
            intercept_imported_track_fn,
        )?;

        let ImportAndReplaceFromFilePathOutcome {
            content_path,
            import_issues,
            media_source_id,
        } = replace_after_imported_from_file_path(
            outcome,
            &mut summary,
            repo,
            content_path_resolver,
        )?;
        if let Some(media_source_id) = media_source_id {
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
    Ok(Outcome {
        completion: Completion::Finished,
        summary,
        visited_media_source_ids,
        imported_media_sources_with_issues,
    })
}

const EXPECTED_NUMBER_OF_DIR_ENTRIES: usize = 1024;

pub fn import_and_replace_by_local_file_path_from_directory<Repo, InterceptImportedTrackFn>(
    repo: &mut Repo,
    collection_uid: &CollectionUid,
    source_dir_path: &ContentPath<'_>,
    params: &Params,
    intercept_imported_track_fn: &mut InterceptImportedTrackFn,
    abort_flag: &AtomicBool,
) -> Result<Outcome>
where
    Repo: CollectionRepo + TrackCollectionRepo,
    InterceptImportedTrackFn: FnMut(Track) -> Track + Send + Sync,
{
    let collection_ctx = RepoContext::resolve(repo, collection_uid, None)?;
    let Some(resolver) = &collection_ctx.content_path.resolver else {
        let path_kind = collection_ctx.content_path.kind;
        return Err(anyhow::anyhow!("unsupported path kind: {path_kind:?}").into());
    };
    let collection_id = collection_ctx.record_id;
    import_and_replace_by_local_file_path_from_directory_with_content_path_resolver(
        repo,
        collection_id,
        resolver.canonical_resolver(),
        source_dir_path,
        params,
        intercept_imported_track_fn,
        abort_flag,
    )
}

pub fn import_and_replace_by_local_file_path_from_directory_with_content_path_resolver<
    InterceptImportedTrackFn,
>(
    repo: &mut impl TrackCollectionRepo,
    collection_id: CollectionId,
    content_path_resolver: &VfsResolver,
    source_dir_path: &ContentPath<'_>,
    params: &Params,
    intercept_imported_track_fn: &mut InterceptImportedTrackFn,
    abort_flag: &AtomicBool,
) -> Result<Outcome>
where
    InterceptImportedTrackFn: FnMut(Track) -> Track + Send + Sync,
{
    let dir_path = content_path_resolver.build_file_path(source_dir_path);
    log::debug!("Importing files from directory: {}", dir_path.display());
    let dir_entries = read_dir(dir_path)?;
    let mut summary = Summary::default();
    let mut visited_media_source_ids = Vec::with_capacity(EXPECTED_NUMBER_OF_DIR_ENTRIES);
    let mut imported_media_sources_with_issues =
        Vec::with_capacity(EXPECTED_NUMBER_OF_DIR_ENTRIES / 4);
    // TODO: Import multiple directory entries in parallel.
    for dir_entry in dir_entries {
        let dir_entry = match dir_entry {
            Ok(dir_entry) => dir_entry,
            Err(err) => {
                log::warn!("Failed to access directory entry: {err}");
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
            content_path.clone()
        } else {
            log::warn!(
                "Skipping invalid/unsupported directory entry: {}",
                dir_entry.path().display()
            );
            // Skip entry and keep going
            continue;
        };
        let context = ImportReplacementFromFilePathContext::load_from_repo(
            repo,
            collection_id,
            content_path,
        )?;
        let outcome = import_replacement_from_file_path(
            context,
            content_path_resolver,
            params,
            intercept_imported_track_fn,
        )?;
        let ImportAndReplaceFromFilePathOutcome {
            content_path,
            import_issues,
            media_source_id,
        } = replace_after_imported_from_file_path(
            outcome,
            &mut summary,
            repo,
            content_path_resolver,
        )?;
        if let Some(media_source_id) = media_source_id {
            visited_media_source_ids.push(media_source_id);
            imported_media_sources_with_issues.push((media_source_id, content_path, import_issues));
        }
    }
    Ok(Outcome {
        completion: Completion::Finished,
        summary,
        visited_media_source_ids,
        imported_media_sources_with_issues,
    })
}
