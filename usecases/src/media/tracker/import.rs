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

use super::*;

use aoide_core::usecases::media::tracker::import::Summary;

use aoide_repo::{
    collection::RecordId as CollectionId,
    media::tracker::{Repo as MediaTrackerRepo, TrackedDirectory},
    prelude::{Pagination, PaginationOffset},
    track::{EntityRepo as TrackRepo, ReplaceMode},
};

use tracks::replace::{
    import_and_replace_by_local_file_path_from_directory, Completion as ReplaceCompletion,
    Outcome as ReplaceOutcome,
};

use std::sync::atomic::{AtomicBool, Ordering};
use url::Url;

///////////////////////////////////////////////////////////////////////

pub use aoide_media::io::import::{ImportTrackConfig, ImportTrackFlags};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Outcome {
    pub completion: Completion,
    pub summary: Summary,
}

impl Outcome {
    const fn new(completion: Completion, summary: Summary) -> Self {
        Self {
            completion,
            summary,
        }
    }
}

// TODO: Reduce number of arguments
#[allow(clippy::too_many_arguments)]
pub fn import<Repo>(
    repo: &Repo,
    collection_id: CollectionId,
    import_mode: ImportMode,
    import_config: &ImportTrackConfig,
    import_flags: ImportTrackFlags,
    source_path_resolver: &VirtualFilePathResolver,
    root_dir_url: Option<&Url>,
    progress_fn: &mut impl FnMut(&Summary),
    abort_flag: &AtomicBool,
) -> Result<Outcome>
where
    Repo: MediaTrackerRepo + TrackRepo,
{
    let path_prefix = root_dir_url
        .map(|url| resolve_path_prefix_from_url(source_path_resolver, url))
        .transpose()?;
    let mut summary = Summary::default();
    let outcome = 'outcome: loop {
        progress_fn(&summary);
        let pending_entries = repo.media_tracker_load_directories_requiring_confirmation(
            collection_id,
            path_prefix
                .as_deref()
                .map(String::as_str)
                .unwrap_or_default(),
            &Pagination {
                offset: Some(summary.directories.skipped as PaginationOffset),
                limit: 1,
            },
        )?;
        if pending_entries.is_empty() {
            log::debug!("Finished import of pending directories: {:?}", summary);
            let outcome = Outcome::new(Completion::Finished, summary);
            break 'outcome outcome;
        }
        for pending_entry in pending_entries {
            if abort_flag.load(Ordering::Relaxed) {
                log::debug!("Aborting import of pending directories: {:?}", summary);
                let outcome = Outcome::new(Completion::Aborted, summary);
                break 'outcome outcome;
            }
            let TrackedDirectory {
                path: dir_path,
                status: _status,
                digest,
            } = pending_entry;
            debug_assert!(_status.is_pending());
            let outcome = match import_and_replace_by_local_file_path_from_directory(
                repo,
                collection_id,
                import_mode,
                import_config,
                import_flags,
                ReplaceMode::UpdateOrCreate,
                source_path_resolver,
                &dir_path,
                abort_flag,
            ) {
                Ok(outcome) => outcome,
                Err(err) => {
                    log::warn!("Failed to import pending directory {}: {}", dir_path, err);
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
                    let outcome = Outcome::new(Completion::Aborted, summary);
                    break 'outcome outcome;
                }
            }
            match repo.media_tracker_confirm_directory(
                DateTime::now_utc(),
                collection_id,
                &dir_path,
                &digest,
                &media_source_ids,
            ) {
                Ok(true) => {
                    log::debug!("Confirmed pending directory {}", dir_path);
                    summary.directories.confirmed += 1;
                }
                Ok(false) => {
                    // Might be rejected if the digest has been updated meanwhile
                    log::info!(
                        "Confirmation of imported directory {} was rejected",
                        dir_path
                    );
                    summary.directories.rejected += 1;
                    // Try again
                    continue;
                }
                Err(err) => {
                    log::warn!("Failed to confirm pending directory {}: {}", dir_path, err);
                    // Skip this directory and keep going
                    summary.directories.skipped += 1;
                    continue;
                }
            }
        }
    };
    progress_fn(&outcome.summary);
    Ok(outcome)
}
