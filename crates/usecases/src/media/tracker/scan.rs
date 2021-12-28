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

use std::{sync::atomic::AtomicBool, time::Duration};

use url::Url;

use aoide_core::{
    entity::EntityUid,
    util::{clock::DateTime, url::BaseUrl},
};

use aoide_core_api::media::tracker::{
    scan::{Outcome, Summary},
    Completion, FsTraversalDirectoriesProgress, FsTraversalEntriesProgress, FsTraversalParams,
    FsTraversalProgress,
};

use aoide_media::{
    fs::{digest, visit},
    resolver::SourcePathResolver,
};

use aoide_repo::{
    collection::EntityRepo as CollectionRepo,
    media::tracker::{DirUpdateOutcome, Repo as MediaTrackerRepo},
};

use crate::collection::resolve_collection_id_for_virtual_file_path;

use super::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProgressEvent {
    pub elapsed: Duration,
    pub status: visit::Status,
    pub progress: FsTraversalProgress,
}

impl From<visit::ProgressEvent> for ProgressEvent {
    fn from(from: visit::ProgressEvent) -> Self {
        let visit::ProgressEvent {
            started_at,
            status,
            progress:
                visit::Progress {
                    directories:
                        visit::DirectoriesProgress {
                            finished: directories_finished,
                        },
                    entries:
                        visit::EntriesProgress {
                            skipped: entries_skipped,
                            finished: entries_finished,
                        },
                },
        } = from;
        Self {
            elapsed: started_at.elapsed(),
            status,
            progress: FsTraversalProgress {
                directories: FsTraversalDirectoriesProgress {
                    finished: directories_finished,
                },
                entries: FsTraversalEntriesProgress {
                    skipped: entries_skipped,
                    finished: entries_finished,
                },
            },
        }
    }
}

pub fn visit_directories<
    Repo: CollectionRepo + MediaTrackerRepo,
    ReportProgressFn: FnMut(ProgressEvent),
>(
    repo: &Repo,
    collection_uid: &EntityUid,
    params: &FsTraversalParams,
    report_progress_fn: &mut ReportProgressFn,
    abort_flag: &AtomicBool,
) -> Result<Outcome> {
    let (collection_id, source_path_resolver) =
        resolve_collection_id_for_virtual_file_path(repo, collection_uid, None)?;
    let FsTraversalParams {
        root_url,
        max_depth,
    } = params;
    let root_path_prefix = root_url
        .as_ref()
        .map(|url| resolve_path_prefix_from_base_url(&source_path_resolver, url))
        .transpose()?
        .unwrap_or_default();
    let root_url = source_path_resolver
        .resolve_url_from_path(&root_path_prefix)
        .map_err(anyhow::Error::from)?;
    let root_url = BaseUrl::new(root_url);
    let root_path = source_path_resolver.build_file_path(&root_path_prefix);
    let outdated_count = repo.media_tracker_mark_current_directories_outdated(
        DateTime::now_utc(),
        collection_id,
        &root_path_prefix,
    )?;
    log::debug!(
        "Marked {} current cache entries as outdated",
        outdated_count
    );
    let mut summary = Summary::default();
    let completion = digest::hash_directories::<_, anyhow::Error, _, _, _>(
        &root_path,
        *max_depth,
        abort_flag,
        &mut blake3::Hasher::new,
        &mut |dir_path, digest| {
            debug_assert!(dir_path.is_relative());
            let full_path = root_path.join(&dir_path);
            debug_assert!(full_path.is_absolute());
            let url = Url::from_directory_path(&full_path).expect("URL");
            debug_assert!(url.as_str().starts_with(root_url.as_str()));
            let path = source_path_resolver.resolve_path_from_url(&url)?;
            match repo
                .media_tracker_update_directory_digest(
                    DateTime::now_utc(),
                    collection_id,
                    &path,
                    &digest.into(),
                )
                .map_err(anyhow::Error::from)?
            {
                DirUpdateOutcome::Current => {
                    summary.current += 1;
                }
                DirUpdateOutcome::Inserted => {
                    log::debug!("Found added directory: {}", full_path.display());
                    summary.added += 1;
                }
                DirUpdateOutcome::Updated => {
                    log::debug!("Found modified directory: {}", full_path.display());
                    summary.modified += 1;
                }
                DirUpdateOutcome::Skipped => {
                    log::debug!("Skipped directory: {}", full_path.display());
                    summary.skipped += 1;
                }
            }
            Ok(visit::AfterAncestorFinished::Continue)
        },
        &mut |progress_event| {
            log::trace!("{:?}", progress_event);
            report_progress_fn(progress_event.to_owned().into());
        },
    )
    .map_err(anyhow::Error::from)
    .map_err(RepoError::from)
    .and_then(|outcome| {
        let visit::Outcome {
            completion,
            progress: _,
        } = outcome;
        match completion {
            visit::Completion::Finished => {
                // Mark all remaining entries that are unreachable and
                // have not been visited as orphaned.
                summary.orphaned = repo.media_tracker_mark_outdated_directories_orphaned(
                    DateTime::now_utc(),
                    collection_id,
                    &root_path_prefix,
                )?;
                debug_assert!(summary.orphaned <= outdated_count);
                Ok(Completion::Finished)
            }
            visit::Completion::Aborted => {
                // All partial results up to now can safely be committed.
                Ok(Completion::Aborted)
            }
        }
    })?;
    Ok(Outcome {
        root_url,
        completion,
        summary,
    })
}
