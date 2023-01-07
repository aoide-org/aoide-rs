// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{marker::PhantomData, sync::atomic::AtomicBool, time::Duration};

use aoide_core::media::content::resolver::ContentPathResolver as _;

use aoide_core_api::media::tracker::{
    find_untracked_files::Outcome, Completion, FsTraversalDirectoriesProgress,
    FsTraversalEntriesProgress, FsTraversalParams, FsTraversalProgress,
};

use aoide_media::fs::visit::{self, url_from_walkdir_entry};

use aoide_repo::{
    collection::{EntityRepo as CollectionRepo, RecordId as CollectionId},
    media::tracker::Repo as MediaTrackerRepo,
};

use crate::collection::vfs::RepoContext;

use super::*;

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

struct AncestorVisitor<'r, Repo> {
    collection_id: CollectionId,
    content_path_resolver: &'r VirtualFilePathResolver,
    content_paths: Vec<ContentPath>,
    _repo_marker: PhantomData<Repo>,
}

impl<'r, Repo> AncestorVisitor<'r, Repo> {
    #[must_use]
    fn new(
        collection_id: CollectionId,
        content_path_resolver: &'r VirtualFilePathResolver,
    ) -> Self {
        Self {
            collection_id,
            content_path_resolver,
            content_paths: Vec::new(),
            _repo_marker: PhantomData,
        }
    }
}

impl<'r, Repo> visit::AncestorVisitor<Repo, Vec<ContentPath>, anyhow::Error>
    for AncestorVisitor<'r, Repo>
where
    Repo: MediaTrackerRepo,
{
    fn visit_dir_entry(
        &mut self,
        repo: &mut Repo,
        dir_entry: &walkdir::DirEntry,
    ) -> anyhow::Result<()> {
        let url = url_from_walkdir_entry(dir_entry)?;
        let content_path = self.content_path_resolver.resolve_path_from_url(&url)?;
        if !content_path.is_terminal() {
            // Skip non-terminal paths, i.e. directories
            return Ok(());
        }
        match repo.media_tracker_resolve_source_id_synchronized_at_by_path(
            self.collection_id,
            &content_path,
        ) {
            Ok(_) => Ok(()),
            Err(RepoError::NotFound) => {
                self.content_paths.push(content_path);
                Ok(())
            }
            Err(err) => Err(err.into()),
        }
    }
    fn finalize(self) -> Vec<ContentPath> {
        self.content_paths
    }
}

#[allow(clippy::unnecessary_wraps)]
fn ancestor_finished(
    all_content_paths: &mut Vec<ContentPath>,
    mut content_paths: Vec<ContentPath>,
) -> anyhow::Result<visit::AfterAncestorFinished> {
    all_content_paths.append(&mut content_paths);
    Ok(visit::AfterAncestorFinished::Continue)
}

pub fn visit_directories<
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
    let vfs_ctx = if let Some(vfs_ctx) = &collection_ctx.content_path.vfs {
        vfs_ctx
    } else {
        let path_kind = collection_ctx.content_path.kind;
        return Err(anyhow::anyhow!("Unsupported path kind: {path_kind:?}").into());
    };
    let collection_id = collection_ctx.record_id;
    let root_file_path = vfs_ctx.build_root_file_path();
    let mut content_paths = Vec::new();
    let completion = visit::visit_directories(
        repo,
        &root_file_path,
        *max_depth,
        abort_flag,
        &mut move |_| AncestorVisitor::new(collection_id, &vfs_ctx.path_resolver),
        &mut |_path, untracked_content_paths| {
            ancestor_finished(&mut content_paths, untracked_content_paths)
        },
        &mut |progress_event| {
            log::trace!("{progress_event:?}");
            report_progress_fn(progress_event.clone().into());
        },
    )
    .map_err(anyhow::Error::from)
    .map(|mut progress_event| {
        progress_event.try_finish();
        report_progress_fn(progress_event.clone().into());
        let elapsed = progress_event.elapsed_since_started();
        let outcome = progress_event.finalize();
        log::info!(
            "Finding {num_untracked_dir_entries} untracked directory entries in '{root_file_path}' took {elapsed_secs} s",
            num_untracked_dir_entries = content_paths.len(),
            root_file_path = root_file_path.display(),
            elapsed_secs = elapsed.as_secs_f64(),
        );
        outcome
    })
    .map(|outcome| {
        let visit::Outcome {
            completion,
            progress: _,
        } = outcome;
        match completion {
            visit::Completion::Finished => Completion::Finished,
            visit::Completion::Aborted => Completion::Aborted,
        }
    })?;
    let (root_url, root_path) = collection_ctx
        .content_path
        .vfs
        .map(|vfs_context| (vfs_context.root_url, vfs_context.root_path))
        .expect("collection with path kind VFS");
    Ok(Outcome {
        root_url,
        root_path,
        completion,
        content_paths,
    })
}
