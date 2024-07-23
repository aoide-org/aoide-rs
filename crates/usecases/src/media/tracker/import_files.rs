// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    io,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

use aoide_core::media::content::resolver::vfs::RemappingVfsResolver;
use aoide_core_api::{
    media::tracker::{
        import_files::{ImportedSourceWithIssues, Outcome, Params, Summary},
        Completion,
    },
    track::replace::Summary as TracksSummary,
};
use aoide_media_file::io::import::ImportTrackConfig;
use aoide_repo::{
    collection::{EntityRepo as CollectionRepo, RecordId as CollectionId},
    media::tracker::{Repo as MediaTrackerRepo, TrackedDirectory},
    prelude::{Pagination, PaginationOffset},
    track::{CollectionRepo as TrackCollectionRepo, ReplaceMode},
};

use super::*;
use crate::{
    collection::vfs::RepoContext,
    track::import_and_replace::{
        self, import_and_replace_by_local_file_path_from_directory_with_content_path_resolver,
        Outcome as ImportAndReplaceOutcome,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgressEvent {
    pub elapsed: Duration,
    pub summary: Summary,
}

#[allow(clippy::too_many_lines)] // TODO
#[allow(clippy::missing_panics_doc)] // Never panics
pub fn import_files<Repo, InterceptImportedTrackFn, ReportProgressFn>(
    repo: &mut Repo,
    collection_uid: &CollectionUid,
    params: &Params,
    import_config: ImportTrackConfig,
    intercept_imported_track_fn: &InterceptImportedTrackFn,
    report_progress_fn: &mut ReportProgressFn,
    abort_flag: &AtomicBool,
) -> Result<Outcome>
where
    Repo: CollectionRepo + MediaTrackerRepo + TrackCollectionRepo,
    InterceptImportedTrackFn: Fn(Track) -> Track + Send,
    ReportProgressFn: FnMut(ProgressEvent),
{
    let Params {
        root_url,
        sync_mode,
    } = params;
    let collection_ctx = RepoContext::resolve(repo, collection_uid, root_url.as_ref())?;
    let Some(resolver) = &collection_ctx.content_path.resolver else {
        let path_kind = collection_ctx.content_path.kind;
        return Err(anyhow::anyhow!("unsupported path kind: {path_kind:?}").into());
    };
    let import_and_replace_params = import_and_replace::Params {
        sync_mode: *sync_mode,
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
            resolver.root_path(),
            &Pagination {
                offset: Some(summary.directories.skipped as PaginationOffset),
                limit: Some(1),
            },
        )?;

        if pending_directories.is_empty() {
            log::debug!("Finished import of pending directories: {summary:?}");
            let (root_url, root_path) = collection_ctx
                .content_path
                .resolver
                .map(RemappingVfsResolver::dismantle)
                .expect("collection with path kind VFS");
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
                    .resolver
                    .map(RemappingVfsResolver::dismantle)
                    .expect("collection with path kind VFS");
                let outcome = Outcome {
                    root_url,
                    root_path,
                    completion: Completion::Aborted,
                    summary,
                    imported_sources_with_issues,
                };
                break 'outcome outcome;
            }

            let import_pending_directory_res = import_pending_directory(
                repo,
                collection_id,
                resolver,
                &import_and_replace_params,
                intercept_imported_track_fn,
                abort_flag,
                &pending_directory,
            );
            let TrackedDirectory {
                content_path: content_dir_path,
                ..
            } = pending_directory;
            match import_pending_directory_res {
                Ok(outcome) => match outcome {
                    ImportPendingDirectoryOutcome::Untracked(untracked) => {
                        summary.directories.untracked += untracked;
                        continue;
                    }
                    ImportPendingDirectoryOutcome::Finished {
                        completion,
                        tracks_summary,
                        imported_sources_with_issues: mut more_imported_sources_with_issues,
                    } => {
                        summary.tracks += &tracks_summary;
                        imported_sources_with_issues.append(&mut more_imported_sources_with_issues);
                        match completion {
                            ImportPendingDirectoryCompletion::Aborted => {
                                log::debug!("Aborting import of pending directories: {summary:?}");
                                let (root_url, root_path) = collection_ctx
                                    .content_path
                                    .resolver
                                    .map(RemappingVfsResolver::dismantle)
                                    .expect("collection with path kind VFS");
                                let outcome = Outcome {
                                    root_url,
                                    root_path,
                                    completion: Completion::Aborted,
                                    summary,
                                    imported_sources_with_issues,
                                };
                                break 'outcome outcome;
                            }
                            ImportPendingDirectoryCompletion::Rejected => {
                                // Might be rejected if the digest has been updated meanwhile
                                log::info!(
                                    "Confirmation of imported directory '{content_dir_path}' was \
                                     rejected",
                                );
                                // Keep going and retry to import this directory later
                                continue;
                            }
                            ImportPendingDirectoryCompletion::NotConfirmed => {
                                // Warnings why the confirmation didn't complete have already been logged.
                                summary.directories.skipped += 1;
                            }
                            ImportPendingDirectoryCompletion::Confirmed => {
                                log::debug!("Confirmed pending directory '{content_dir_path}'");
                                summary.directories.confirmed += 1;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::warn!("Failed to import pending directory '{content_dir_path}': {err}");
                    // Skip this directory and keep going
                    summary.directories.skipped += 1;
                    continue;
                }
            }
        }
    };

    // Report final progress.
    report_progress_fn(ProgressEvent {
        elapsed: started_at.elapsed(),
        summary: outcome.summary.clone(),
    });

    Ok(outcome)
}

enum ImportPendingDirectoryOutcome {
    Untracked(usize),
    Finished {
        completion: ImportPendingDirectoryCompletion,
        tracks_summary: TracksSummary,
        imported_sources_with_issues: Vec<ImportedSourceWithIssues>,
    },
}

enum ImportPendingDirectoryCompletion {
    Aborted,
    Rejected,
    Confirmed,
    /// Maybe imported some media sources from files, but couldn't confirm the directory.
    NotConfirmed,
}

fn import_pending_directory<Repo, InterceptImportedTrackFn>(
    repo: &mut Repo,
    collection_id: CollectionId,
    resolver: &RemappingVfsResolver,
    import_and_replace_params: &import_and_replace::Params,
    intercept_imported_track_fn: &InterceptImportedTrackFn,
    abort_flag: &AtomicBool,
    pending_directory: &TrackedDirectory,
) -> Result<ImportPendingDirectoryOutcome>
where
    Repo: CollectionRepo + MediaTrackerRepo + TrackCollectionRepo,
    InterceptImportedTrackFn: Fn(Track) -> Track + Send,
{
    let TrackedDirectory {
        content_path,
        status,
        digest,
    } = pending_directory;
    debug_assert!(status.is_pending());
    let outcome =
        match import_and_replace_by_local_file_path_from_directory_with_content_path_resolver(
            repo,
            collection_id,
            resolver.canonical_resolver(),
            content_path,
            import_and_replace_params,
            intercept_imported_track_fn,
            abort_flag,
        ) {
            Ok(outcome) => outcome,
            Err(err) => {
                let err = if let Error::Io(io_err) = err {
                    if io_err.kind() == io::ErrorKind::NotFound {
                        log::info!("Untracking missing directory '{content_path}'");
                        let untracked = repo.media_tracker_untrack_directories(
                            collection_id,
                            content_path,
                            None,
                        )?;
                        return Ok(ImportPendingDirectoryOutcome::Untracked(untracked));
                    }
                    // Restore error
                    Error::Io(io_err)
                } else {
                    // Pass-through error
                    err
                };
                return Err(err);
            }
        };
    let ImportAndReplaceOutcome {
        completion,
        summary: tracks_summary,
        visited_media_source_ids,
        imported_media_sources_with_issues,
    } = outcome;
    let imported_sources_with_issues = imported_media_sources_with_issues
        .into_iter()
        .map(|(_, path, issues)| ImportedSourceWithIssues {
            path,
            messages: issues.into_messages(),
        })
        .collect();
    match completion {
        Completion::Finished => {}
        Completion::Aborted => {
            return Ok(ImportPendingDirectoryOutcome::Finished {
                completion: ImportPendingDirectoryCompletion::Aborted,
                tracks_summary,
                imported_sources_with_issues,
            });
        }
    }
    let updated_at: OffsetDateTimeMs = OffsetDateTimeMs::now_utc();
    let completion;
    if tracks_summary.failed.is_empty() {
        match repo.media_tracker_confirm_directory(&updated_at, collection_id, content_path, digest)
        {
            Ok(true) => {
                completion = ImportPendingDirectoryCompletion::Confirmed;
            }
            Ok(false) => {
                // Might be rejected if the digest has been updated meanwhile
                return Ok(ImportPendingDirectoryOutcome::Finished {
                    completion: ImportPendingDirectoryCompletion::Rejected,
                    tracks_summary,
                    imported_sources_with_issues,
                });
            }
            Err(err) => {
                log::warn!("Failed to confirm pending directory '{content_path}': {err}");
                // Skip this directory, but remember the sources imported from
                // this directory (see below).
                completion = ImportPendingDirectoryCompletion::NotConfirmed;
            }
        }
    } else {
        log::warn!(
            "Postponing confirmation of pending directory '{content_path}' after {num_failures} \
             import failure(s)",
            num_failures = tracks_summary.failed.len(),
        );
        // Skip this directory, but remember the sources imported from
        // this directory (see below).
        completion = ImportPendingDirectoryCompletion::NotConfirmed;
    }
    if let Err(err) = repo.media_tracker_replace_directory_sources(
        collection_id,
        content_path,
        &visited_media_source_ids,
    ) {
        log::warn!("Failed to replace imported sources in directory '{content_path}': {err}");
    }
    Ok(ImportPendingDirectoryOutcome::Finished {
        completion,
        tracks_summary,
        imported_sources_with_issues,
    })
}
