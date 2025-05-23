// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{path::Path, sync::atomic::AtomicBool, time::Duration};

use anyhow::anyhow;
use url::Url;

use aoide_core::{
    CollectionUid,
    media::content::resolver::{ContentPathResolver as _, vfs::RemappingVfsResolver},
    util::clock::UtcDateTimeMs,
};
use aoide_core_api::media::tracker::{
    Completion, FsTraversalDirectoriesProgress, FsTraversalEntriesProgress, FsTraversalParams,
    FsTraversalProgress,
    scan_directories::{Outcome, Summary},
};
use aoide_media_file::fs::{
    digest::{HashDirectoryVisitor, hash_directories},
    visit,
};
use aoide_repo::{
    RepoError,
    collection::EntityRepo as CollectionRepo,
    media::tracker::{DirUpdateOutcome, Repo as MediaTrackerRepo},
};

use crate::{Error, Result, collection::vfs::RepoContext};

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

#[expect(clippy::too_many_lines)] // TODO
#[expect(clippy::missing_panics_doc)] // Never panics
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
        excluded_paths,
        max_depth,
    } = params;
    let collection_ctx = RepoContext::resolve(repo, collection_uid, root_url.as_ref())?;
    let Some(resolver) = &collection_ctx.content_path.resolver else {
        let path_kind = collection_ctx.content_path.kind;
        return Err(Error::Other(anyhow!(
            "unsupported path kind: {path_kind:?}"
        )));
    };
    let collection_id = collection_ctx.record_id;
    let root_file_path = resolver.build_file_path(resolver.root_path());
    let excluded_paths = excluded_paths
        .iter()
        .map(|dir_path| resolver.build_file_path(dir_path).into())
        .collect::<Vec<_>>();
    let outdated_count = repo.media_tracker_mark_current_directories_outdated(
        UtcDateTimeMs::now(),
        collection_id,
        resolver.root_path(),
    )?;
    log::debug!("Marked {outdated_count} current cache entries as outdated");
    let mut summary = Summary::default();
    let digest_finished_fn = |dir_path: &Path, digest: digest::Output<blake3::Hasher>| {
        log::debug!(
            "Finishing directory: {dir_path}",
            dir_path = dir_path.display()
        );
        debug_assert!(dir_path.is_relative());
        let full_path = root_file_path.join(dir_path);
        debug_assert!(full_path.is_absolute());
        let url = Url::from_directory_path(&full_path).expect("URL");
        debug_assert!(
            url.as_str()
                .starts_with(resolver.canonical_root_url().as_str())
        );
        let content_path = resolver
            .resolve_path_from_url(&url)?
            .ok_or_else(|| anyhow!("unresolved URL: {url}"))?;
        log::debug!("Updating digest of content path: {content_path}");
        let updated_at = UtcDateTimeMs::now();
        match repo
            .media_tracker_update_directory_digest(
                updated_at,
                collection_id,
                &content_path,
                &digest.into(),
            )
            .map_err(Into::into)
            .map_err(Error::Other)?
        {
            DirUpdateOutcome::Current => {
                summary.current += 1;
            }
            DirUpdateOutcome::Inserted => {
                log::debug!("Found added directory: {path}", path = full_path.display());
                summary.added += 1;
            }
            DirUpdateOutcome::Updated => {
                log::debug!(
                    "Found modified directory: {path}",
                    path = full_path.display()
                );
                summary.modified += 1;
            }
            DirUpdateOutcome::Skipped => {
                log::debug!("Skipped directory: {path}", path = full_path.display());
                summary.skipped += 1;
            }
        }
        Ok(visit::AfterAncestorFinished::Continue)
    };
    let mut directory_visitor = HashDirectoryVisitor::new(blake3::Hasher::new, digest_finished_fn);
    let mut report_progress_fn = |progress_event: &visit::ProgressEvent| {
        log::trace!("{progress_event:?}");
        report_progress_fn(progress_event.clone().into());
    };
    let completion = hash_directories::<_, anyhow::Error, _, _, _>(
        &root_file_path.as_path().into(),
        &excluded_paths,
        *max_depth,
        abort_flag,
        &mut directory_visitor,
        &mut report_progress_fn,
    )
    .map_err(Into::into)
    .map_err(RepoError::Other)
    .and_then(|outcome| {
        let visit::Outcome {
            completion,
            progress: _,
        } = outcome;
        match completion {
            visit::Completion::Finished => {
                // Mark all remaining entries that are unreachable and
                // have not been visited as orphaned.
                let updated_at = UtcDateTimeMs::now();
                summary.orphaned = repo.media_tracker_mark_outdated_directories_orphaned(
                    updated_at,
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
