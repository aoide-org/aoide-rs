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

use aoide_repo::{
    collection::RecordId as CollectionId,
    media::tracker::{Repo as MediaTrackerRepo, TrackedDirectory},
    prelude::{Pagination, PaginationOffset},
    track::{EntityRepo as TrackRepo, ReplaceMode},
};
use tracks::replace::{
    import_and_replace_by_media_source_uri_from_directory, Completion as ReplaceCompletion,
    Outcome as ReplaceOutcome, Summary as ReplaceSummary,
};

use std::{
    ops::AddAssign,
    sync::atomic::{AtomicBool, Ordering},
};
use url::Url;

///////////////////////////////////////////////////////////////////////

pub use aoide_media::io::import::{ImportTrackConfig, ImportTrackFlags};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Summary {
    pub tracks: TrackSummary,
    pub directories: DirectorySummary,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TrackSummary {
    pub created: usize,
    pub updated: usize,
    pub missing: usize,
    pub unchanged: usize,
    pub not_imported: usize,
    pub not_created: usize,
    pub not_updated: usize,
}

impl AddAssign<&ReplaceSummary> for TrackSummary {
    fn add_assign(&mut self, rhs: &ReplaceSummary) {
        let Self {
            created,
            updated,
            unchanged,
            missing: _,
            not_imported,
            not_created,
            not_updated,
        } = self;
        let ReplaceSummary {
            created: rhs_created,
            updated: rhs_updated,
            unchanged: rhs_unchanged,
            not_imported: rhs_not_imported,
            not_created: rhs_not_created,
            not_updated: rhs_not_updated,
        } = rhs;
        *created += rhs_created.len();
        *updated += rhs_updated.len();
        *unchanged += rhs_unchanged.len();
        *not_imported += rhs_not_imported.len();
        *not_created += rhs_not_created.len();
        *not_updated += rhs_not_updated.len();
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DirectorySummary {
    pub confirmed: usize,
    pub rejected: usize,
    pub skipped: usize,
}

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
    root_dir_url: Option<&Url>,
    import_mode: ImportMode,
    import_config: &ImportTrackConfig,
    import_flags: ImportTrackFlags,
    progress_fn: &mut impl FnMut(&Summary),
    abort_flag: &AtomicBool,
) -> Result<Outcome>
where
    Repo: MediaTrackerRepo + TrackRepo,
{
    let uri_prefix = root_dir_url.map(uri_path_prefix_from_url).transpose()?;
    let mut summary = Summary::default();
    let outcome = 'outcome: loop {
        progress_fn(&summary);
        let pending_entries = repo.media_tracker_load_directories_requiring_confirmation(
            collection_id,
            uri_prefix.as_deref(),
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
                uri,
                status: _status,
                digest,
            } = pending_entry;
            debug_assert!(_status.is_pending());
            let dir_url = match uri.clone().parse() {
                Ok(url) => url,
                Err(err) => {
                    log::warn!("Failed to convert URI {} to URL: {}", uri, err);
                    // Skip this directory and keep going
                    summary.directories.skipped += 1;
                    continue;
                }
            };
            let outcome = match import_and_replace_by_media_source_uri_from_directory(
                repo,
                collection_id,
                &dir_url,
                import_mode,
                import_config,
                import_flags,
                ReplaceMode::UpdateOrCreate,
                abort_flag,
            ) {
                Ok(outcome) => outcome,
                Err(err) => {
                    log::warn!("Failed to import pending directory {}: {}", uri, err);
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
                &uri,
                &digest,
                &media_source_ids,
            ) {
                Ok(true) => {
                    log::debug!("Confirmed pending directory {}", uri);
                    summary.directories.confirmed += 1;
                }
                Ok(false) => {
                    // Might be rejected if the digest has been updated meanwhile
                    log::info!("Confirmation of imported directory {} was rejected", uri);
                    summary.directories.rejected += 1;
                    // Try again
                    continue;
                }
                Err(err) => {
                    log::warn!("Failed to confirm pending directory {}: {}", uri, err);
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
