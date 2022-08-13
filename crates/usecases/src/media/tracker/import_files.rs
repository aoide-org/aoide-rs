// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    io,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

use aoide_core_api::media::tracker::{
    import_files::{ImportedSourceWithIssues, Outcome, Params, Summary},
    Completion,
};

use aoide_media::io::import::ImportTrackConfig;

use aoide_repo::{
    collection::EntityRepo as CollectionRepo,
    media::tracker::{Repo as MediaTrackerRepo, TrackedDirectory},
    prelude::{Pagination, PaginationOffset},
    track::{CollectionRepo as TrackCollectionRepo, ReplaceMode},
};

use crate::{
    collection::vfs::RepoContext,
    track::{
        import_and_replace::{
            self, import_and_replace_by_local_file_path_from_directory_with_content_path_resolver,
            Outcome as ImportAndReplaceOutcome,
        },
        replace::Completion as ReplaceCompletion,
    },
};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgressEvent {
    pub elapsed: Duration,
    pub summary: Summary,
}

pub fn import_files<
    Repo: CollectionRepo + MediaTrackerRepo + TrackCollectionRepo,
    ReportProgressFn: FnMut(ProgressEvent),
>(
    repo: &mut Repo,
    collection_uid: &CollectionUid,
    params: &Params,
    import_config: ImportTrackConfig,
    report_progress_fn: &mut ReportProgressFn,
    abort_flag: &AtomicBool,
) -> Result<Outcome> {
    let Params {
        root_url,
        sync_mode,
    } = params;
    let collection_ctx = RepoContext::resolve(repo, collection_uid, root_url.as_ref())?;
    let vfs_ctx = if let Some(vfs_ctx) = &collection_ctx.content_path.vfs {
        vfs_ctx
    } else {
        let path_kind = collection_ctx.content_path.kind;
        return Err(anyhow::anyhow!("Unsupported path kind: {path_kind:?}").into());
    };
    let import_and_replace_params = import_and_replace::Params {
        sync_mode: sync_mode.unwrap_or(SyncMode::Modified),
        import_config,
        replace_mode: ReplaceMode::UpdateOrCreate,
    };
    let collection_id = collection_ctx.record_id;
    let started_at = Instant::now();
    let mut summary = Summary::default();
    let mut imported_sources_with_issues = Vec::new();
    let outcome = 'outcome: loop {
        report_progress_fn(ProgressEvent {
            elapsed: started_at.elapsed(),
            summary: summary.clone(),
        });
        let pending_directories = repo.media_tracker_load_directories_requiring_confirmation(
            collection_id,
            &vfs_ctx.root_path,
            &Pagination {
                offset: Some(summary.directories.skipped as PaginationOffset),
                limit: Some(1),
            },
        )?;
        if pending_directories.is_empty() {
            log::debug!("Finished import of pending directories: {summary:?}");
            let (root_url, root_path) = collection_ctx
                .content_path
                .vfs
                .map(|vfs| (vfs.root_url, vfs.root_path))
                .unwrap();
            let outcome = Outcome {
                root_url,
                root_path,
                completion: Completion::Finished,
                summary,
                imported_sources_with_issues,
            };
            break 'outcome outcome;
        }
        for pending_directory in pending_directories {
            if abort_flag.load(Ordering::Relaxed) {
                log::debug!("Aborting import of pending directories: {summary:?}");
                let (root_url, root_path) = collection_ctx
                    .content_path
                    .vfs
                    .map(|vfs| (vfs.root_url, vfs.root_path))
                    .unwrap();
                let outcome = Outcome {
                    root_url,
                    root_path,
                    completion: Completion::Aborted,
                    summary,
                    imported_sources_with_issues,
                };
                break 'outcome outcome;
            }
            let TrackedDirectory {
                path: dir_path,
                status,
                digest,
            } = pending_directory;
            debug_assert!(status.is_pending());
            let outcome =
                match import_and_replace_by_local_file_path_from_directory_with_content_path_resolver(
                    repo,
                    collection_id,
                    &vfs_ctx.path_resolver,
                    &import_and_replace_params,
                    &dir_path,
                    abort_flag,
                ) {
                    Ok(outcome) => outcome,
                    Err(err) => {
                        let err = if let Error::Io(io_err) = err {
                            if io_err.kind() == io::ErrorKind::NotFound {
                                log::info!("Untracking missing directory '{dir_path}'");
                                summary.directories.untracked += repo
                                    .media_tracker_untrack_directories(
                                        collection_id,
                                        &dir_path,
                                        None,
                                    )?;
                                continue;
                            }
                            // Restore error
                            Error::Io(io_err)
                        } else {
                            // Pass-through error
                            err
                        };
                        log::warn!("Failed to import pending directory '{dir_path}': {err}");
                        // Skip this directory and keep going
                        summary.directories.skipped += 1;
                        continue;
                    }
                };
            let ImportAndReplaceOutcome {
                completion,
                summary: tracks_summary,
                visited_media_source_ids,
                imported_media_sources_with_issues,
            } = outcome;
            summary.tracks += &tracks_summary;
            imported_sources_with_issues.reserve(imported_media_sources_with_issues.len());
            for (_, path, issues) in imported_media_sources_with_issues {
                imported_sources_with_issues.push(ImportedSourceWithIssues {
                    path,
                    messages: issues.into_messages(),
                });
            }
            match completion {
                ReplaceCompletion::Finished => {}
                ReplaceCompletion::Aborted => {
                    log::debug!("Aborting import of pending directories: {summary:?}");
                    let (root_url, root_path) = collection_ctx
                        .content_path
                        .vfs
                        .map(|vfs| (vfs.root_url, vfs.root_path))
                        .unwrap();
                    let outcome = Outcome {
                        root_url,
                        root_path,
                        completion: Completion::Aborted,
                        summary,
                        imported_sources_with_issues,
                    };
                    break 'outcome outcome;
                }
            }
            let updated_at = DateTime::now_utc();
            if tracks_summary.failed.is_empty() {
                match repo.media_tracker_confirm_directory(
                    updated_at,
                    collection_id,
                    &dir_path,
                    &digest,
                ) {
                    Ok(true) => {
                        log::debug!("Confirmed pending directory '{dir_path}'");
                        summary.directories.confirmed += 1;
                    }
                    Ok(false) => {
                        // Might be rejected if the digest has been updated meanwhile
                        log::info!("Confirmation of imported directory '{dir_path}' was rejected",);
                        // Keep going and retry to import this directory later
                        continue;
                    }
                    Err(err) => {
                        log::warn!("Failed to confirm pending directory '{dir_path}': {err}");
                        // Skip this directory, but remember the sources imported from
                        // this directory (see below)
                        summary.directories.skipped += 1;
                    }
                }
            } else {
                log::warn!(
                    "Postponing confirmation of pending directory '{dir_path}' after {num_failures} import failure(s)",
                    num_failures = tracks_summary.failed.len(),
                );
                // Skip this directory, but remember the sources imported from
                // this directory (see below)
                summary.directories.skipped += 1;
            }
            if let Err(err) = repo.media_tracker_replace_directory_sources(
                collection_id,
                &dir_path,
                &visited_media_source_ids,
            ) {
                log::warn!("Failed replace imported sources in directory '{dir_path}': {err}");
            }
        }
    };
    report_progress_fn(ProgressEvent {
        elapsed: started_at.elapsed(),
        summary: outcome.summary.clone(),
    });
    Ok(outcome)
}
