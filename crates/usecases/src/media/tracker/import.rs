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
    io,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

use aoide_core::{entity::EntityUid, util::url::BaseUrl};

use aoide_core_api::media::tracker::{
    import::{Outcome, Params, Summary},
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
    collection::resolve_collection_id_for_virtual_file_path,
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

// TODO: Reduce number of arguments
#[allow(clippy::too_many_arguments)]
pub fn import<
    Repo: CollectionRepo + MediaTrackerRepo + TrackRepo,
    ReportProgress: FnMut(ProgressEvent),
>(
    repo: &Repo,
    collection_uid: &EntityUid,
    params: &Params,
    config: &ImportTrackConfig,
    report_progress: &mut ReportProgress,
    abort_flag: &AtomicBool,
) -> Result<Outcome> {
    let (collection_id, source_path_resolver) =
        resolve_collection_id_for_virtual_file_path(repo, collection_uid, None)?;
    let Params {
        root_url,
        sync_mode,
    } = params;
    let sync_mode = sync_mode.unwrap_or(SyncMode::Modified);
    let root_path_prefix = root_url
        .as_ref()
        .map(|url| resolve_path_prefix_from_base_url(&source_path_resolver, url))
        .transpose()?
        .unwrap_or_default();
    let root_url = source_path_resolver
        .resolve_url_from_path(&root_path_prefix)
        .map_err(anyhow::Error::from)?;
    let root_url = BaseUrl::new(root_url);
    let started_at = Instant::now();
    let mut summary = Summary::default();
    let outcome = 'outcome: loop {
        report_progress(ProgressEvent {
            elapsed: started_at.elapsed(),
            summary: summary.clone(),
        });
        let pending_entries = repo.media_tracker_load_directories_requiring_confirmation(
            collection_id,
            &root_path_prefix,
            &Pagination {
                offset: Some(summary.directories.skipped as PaginationOffset),
                limit: Some(1),
            },
        )?;
        if pending_entries.is_empty() {
            log::debug!("Finished import of pending directories: {:?}", summary);
            let outcome = Outcome {
                root_url,
                completion: Completion::Finished,
                summary,
            };
            break 'outcome outcome;
        }
        for pending_entry in pending_entries {
            if abort_flag.load(Ordering::Relaxed) {
                log::debug!("Aborting import of pending directories: {:?}", summary);
                let outcome = Outcome {
                    root_url,
                    completion: Completion::Aborted,
                    summary,
                };
                break 'outcome outcome;
            }
            let TrackedDirectory {
                path: dir_path,
                status: _status,
                digest,
            } = pending_entry;
            debug_assert!(_status.is_pending());
            let outcome =
                match import_and_replace_by_local_file_path_from_directory_with_source_path_resolver(
                    repo,
                    collection_id,
                    &source_path_resolver,
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
                                summary.directories.untracked +=
                                    repo.media_tracker_untrack(collection_id, &dir_path, None)?;
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
                media_source_ids,
            } = outcome;
            summary.tracks += &tracks_summary;
            match completion {
                ReplaceCompletion::Finished => {}
                ReplaceCompletion::Aborted => {
                    log::debug!("Aborting import of pending directories: {:?}", summary);
                    let outcome = Outcome {
                        root_url,
                        completion: Completion::Aborted,
                        summary,
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
                    &media_source_ids,
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
                        // Try again
                        continue;
                    }
                    Err(err) => {
                        log::warn!(
                            "Failed to confirm pending directory '{}': {}",
                            dir_path,
                            err
                        );
                        // Skip this directory and keep going
                        summary.directories.skipped += 1;
                        continue;
                    }
                }
            } else {
                log::warn!(
                    "Postponing confirmation of pending directory '{}' after {} import failure(s)",
                    dir_path,
                    tracks_summary.failed.len()
                );
                // Skip this directory (only partially imported) and keep going
                summary.directories.skipped += 1;
                continue;
            }
        }
    };
    report_progress(ProgressEvent {
        elapsed: started_at.elapsed(),
        summary: outcome.summary.clone(),
    });
    Ok(outcome)
}
