// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{sync::atomic::AtomicBool, time::Duration};

use aoide_core::{
    media::content::resolver::{vfs::RemappingVfsResolver, ContentPathResolver as _},
    util::clock::OffsetDateTimeMs,
};
use aoide_core_api::media::tracker::{
    scan_directories::{Outcome, Summary},
    Completion, FsTraversalDirectoriesProgress, FsTraversalEntriesProgress, FsTraversalParams,
    FsTraversalProgress,
};
use aoide_media_file::fs::{digest, visit};
use aoide_repo::{
    collection::EntityRepo as CollectionRepo,
    media::tracker::{DirUpdateOutcome, Repo as MediaTrackerRepo},
};
use url::Url;

use super::*;
use crate::collection::vfs::RepoContext;

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[allow(clippy::too_many_lines)] // TODO
#[allow(clippy::missing_panics_doc)] // Never panics
pub fn scan_directories<
    Repo: CollectionRepo + MediaTrackerRepo,
    ReportProgressFn: FnMut(ProgressEvent),
>(
    repo: &mut Repo,
    collection_uid: &CollectionUid,
    params: &FsTraversalParams,
    report_progress_fn: &mut ReportProgressFn,
    abort_flag: &AtomicBool,
) -> Result<Outcome> {
    let FsTraversalParams {
        root_url,
        max_depth,
    } = params;
    let collection_ctx = RepoContext::resolve(repo, collection_uid, root_url.as_ref())?;
    let Some(resolver) = &collection_ctx.content_path.resolver else {
        let path_kind = collection_ctx.content_path.kind;
        return Err(anyhow::anyhow!("unsupported path kind: {path_kind:?}").into());
    };
    let collection_id = collection_ctx.record_id;
    let root_file_path = resolver.build_file_path(resolver.root_path());
    let outdated_count = repo.media_tracker_mark_current_directories_outdated(
        OffsetDateTimeMs::now_utc(),
        collection_id,
        resolver.root_path(),
    )?;
    log::debug!(
        "Marked {} current cache entries as outdated",
        outdated_count
    );
    let mut summary = Summary::default();
    let completion = digest::hash_directories::<_, anyhow::Error, _, _, _>(
        &root_file_path,
        *max_depth,
        abort_flag,
        &mut blake3::Hasher::new,
        &mut |dir_path, digest| {
            log::debug!(
                "Finishing directory: {dir_path}",
                dir_path = dir_path.display()
            );
            debug_assert!(dir_path.is_relative());
            let full_path = root_file_path.join(dir_path);
            debug_assert!(full_path.is_absolute());
            let url = Url::from_directory_path(&full_path).expect("URL");
            debug_assert!(url
                .as_str()
                .starts_with(resolver.canonical_root_url().as_str()));
            let content_path = resolver.resolve_path_from_url(&url)?;
            log::debug!("Updating digest of content path: {content_path}");
            match repo
                .media_tracker_update_directory_digest(
                    OffsetDateTimeMs::now_utc(),
                    collection_id,
                    &content_path,
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
            log::trace!("{progress_event:?}");
            report_progress_fn(progress_event.clone().into());
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
                    OffsetDateTimeMs::now_utc(),
                    collection_id,
                    resolver.root_path(),
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
    let (root_url, root_path) = collection_ctx
        .content_path
        .resolver
        .map(RemappingVfsResolver::dismantle)
        .expect("collection with path kind VFS");
    Ok(Outcome {
        root_url,
        root_path,
        completion,
        summary,
    })
}
