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
    io,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

use aoide_core::entity::EntityUid;

use aoide_core_api::media::tracker::{
    import_files::{ImportedSourceWithIssues, Outcome, Params, Summary},
    Completion,
};

use aoide_media::io::import::ImportTrackConfig;

use aoide_repo::{
    collection::EntityRepo as CollectionRepo,
    media::tracker::{Repo as MediaTrackerRepo, TrackedDirectory},
    prelude::{Pagination, PaginationOffset},
    track::{EntityRepo as TrackRepo, ReplaceMode},
};

use crate::{
    collection::vfs::RepoContext,
    track::replace::{
        import_and_replace_by_local_file_path_from_directory_with_source_path_resolver,
        Completion as ReplaceCompletion, Outcome as ReplaceOutcome,
    },
};

use super::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProgressEvent {
    pub elapsed: Duration,
    pub summary: Summary,
}

pub fn import_files<
    Repo: CollectionRepo + MediaTrackerRepo + TrackRepo,
    ReportProgressFn: FnMut(ProgressEvent),
>(
    repo: &Repo,
    collection_uid: &EntityUid,
    params: &Params,
    config: &ImportTrackConfig,
    report_progress_fn: &mut ReportProgressFn,
    abort_flag: &AtomicBool,
) -> Result<Outcome> {
    let Params {
        root_url,
        sync_mode,
    } = params;
    let sync_mode = sync_mode.unwrap_or(SyncMode::Modified);
    let collection_ctx = RepoContext::resolve(repo, collection_uid, root_url.as_ref())?;
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
            log::debug!("Finished import of pending directories: {:?}", summary);
            let (root_url, root_path) = collection_ctx
                .source_path
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
                log::debug!("Aborting import of pending directories: {:?}", summary);
                let (root_url, root_path) = collection_ctx
                    .source_path
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
                status: _status,
                digest,
            } = pending_directory;
            debug_assert!(_status.is_pending());
            let outcome =
                match import_and_replace_by_local_file_path_from_directory_with_source_path_resolver(
                    repo,
                    collection_id,
                    &vfs_ctx.path_resolver,
                    sync_mode,
                    config,
                    ReplaceMode::UpdateOrCreate,
                    &dir_path,
                    abort_flag,
                ) {
                    Ok(outcome) => outcome,
                    Err(err) => {
                        let err = if let Error::Io(io_err) = err {
                            if io_err.kind() == io::ErrorKind::NotFound {
                                log::info!("Untracking missing directory '{}'", dir_path);
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
                        log::warn!("Failed to import pending directory '{}': {}", dir_path, err);
                        // Skip this directory and keep going
                        summary.directories.skipped += 1;
                        continue;
                    }
                };
            let ReplaceOutcome {
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
                    log::debug!("Aborting import of pending directories: {:?}", summary);
                    let (root_url, root_path) = collection_ctx
                        .source_path
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
                        log::debug!("Confirmed pending directory '{}'", dir_path);
                        summary.directories.confirmed += 1;
                    }
                    Ok(false) => {
                        // Might be rejected if the digest has been updated meanwhile
                        log::info!(
                            "Confirmation of imported directory '{}' was rejected",
                            dir_path
                        );
                        // Keep going and retry to import this directory later
                        continue;
                    }
                    Err(err) => {
                        log::warn!(
                            "Failed to confirm pending directory '{}': {}",
                            dir_path,
                            err
                        );
                        // Skip this directory, but remember the sources imported from
                        // this directory (see below)
                        summary.directories.skipped += 1;
                    }
                }
            } else {
                log::warn!(
                    "Postponing confirmation of pending directory '{}' after {} import failure(s)",
                    dir_path,
                    tracks_summary.failed.len()
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
                log::warn!(
                    "Failed replace imported sources in directory '{}': {}",
                    dir_path,
                    err
                );
            }
        }
    };
    report_progress_fn(ProgressEvent {
        elapsed: started_at.elapsed(),
        summary: outcome.summary.clone(),
    });
    Ok(outcome)
}
