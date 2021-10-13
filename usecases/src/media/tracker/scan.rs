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

use std::sync::atomic::AtomicBool;

use url::Url;

use aoide_core::util::{clock::DateTime, url::BaseUrl};

use aoide_core_ext::media::tracker::{
    scan::{Outcome, Summary},
    Completion, ScanningDirectoriesProgress, ScanningEntriesProgress, ScanningProgress,
};

use aoide_media::{fs::digest, resolver::SourcePathResolver};

use aoide_repo::{
    collection::RecordId as CollectionId,
    media::tracker::{DirUpdateOutcome, Repo as MediaTrackerRepo},
};

use super::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProgressEvent {
    pub status: digest::Status,
    pub progress: ScanningProgress,
}

impl From<digest::ProgressEvent> for ProgressEvent {
    fn from(from: digest::ProgressEvent) -> Self {
        let digest::ProgressEvent {
            status,
            progress:
                digest::Progress {
                    directories:
                        digest::DirectoriesProgress {
                            finished: directories_finished,
                        },
                    entries:
                        digest::EntriesProgress {
                            skipped: entries_skipped,
                            finished: entries_finished,
                        },
                },
        } = from;
        Self {
            status,
            progress: ScanningProgress {
                directories: ScanningDirectoriesProgress {
                    finished: directories_finished,
                },
                entries: ScanningEntriesProgress {
                    skipped: entries_skipped,
                    finished: entries_finished,
                },
            },
        }
    }
}

pub fn scan_directories_recursively<Repo>(
    repo: &Repo,
    collection_id: CollectionId,
    root_url: Option<BaseUrl>,
    source_path_resolver: &VirtualFilePathResolver,
    max_depth: Option<usize>,
    progress_fn: &mut impl FnMut(ProgressEvent),
    abort_flag: &AtomicBool,
) -> Result<Outcome>
where
    Repo: MediaTrackerRepo,
{
    let root_path_prefix = root_url
        .as_ref()
        .map(|url| resolve_path_prefix_from_base_url(source_path_resolver, url))
        .transpose()?
        .unwrap_or_default();
    let root_url = source_path_resolver
        .resolve_url_from_path(&root_path_prefix)
        .map_err(anyhow::Error::from)?;
    let root_path = source_path_resolver.build_file_path(&root_path_prefix);
    let outdated_count = repo.media_tracker_mark_current_directories_outdated(
        DateTime::now_utc(),
        collection_id,
        &root_path_prefix,
    )?;
    tracing::debug!(
        "Marked {} current cache entries as outdated",
        outdated_count
    );
    let mut summary = Summary::default();
    let completion = digest::hash_directories::<_, anyhow::Error, _, _, _>(
        &root_path,
        max_depth,
        abort_flag,
        blake3::Hasher::new,
        |path, digest| {
            debug_assert!(path.is_relative());
            let full_path = root_path.join(&path);
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
                    tracing::debug!("Found added directory: {}", full_path.display());
                    summary.added += 1;
                }
                DirUpdateOutcome::Updated => {
                    tracing::debug!("Found modified directory: {}", full_path.display());
                    summary.modified += 1;
                }
                DirUpdateOutcome::Skipped => {
                    tracing::debug!("Skipped directory: {}", full_path.display());
                    summary.skipped += 1;
                }
            }
            Ok(digest::AfterDirFinished::Continue)
        },
        |progress_event| {
            tracing::trace!("{:?}", progress_event);
            progress_fn(progress_event.to_owned().into());
        },
    )
    .map_err(anyhow::Error::from)
    .map_err(RepoError::from)
    .and_then(|outcome| {
        let digest::Outcome {
            completion,
            progress: _,
        } = outcome;
        match completion {
            digest::Completion::Finished => {
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
            digest::Completion::Aborted => {
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
