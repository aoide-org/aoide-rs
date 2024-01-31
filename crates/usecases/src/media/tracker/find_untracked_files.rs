// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{marker::PhantomData, path::Path, sync::atomic::AtomicBool, time::Duration};

use aoide_core::media::content::resolver::{vfs::RemappingVfsResolver, ContentPathResolver as _};
use aoide_core_api::media::tracker::{
    find_untracked_files::Outcome, Completion, FsTraversalDirectoriesProgress,
    FsTraversalEntriesProgress, FsTraversalParams, FsTraversalProgress,
};
use aoide_media_file::fs::visit::{self, url_from_walkdir_entry};
use aoide_repo::{
    collection::{EntityRepo as CollectionRepo, RecordId as CollectionId},
    media::tracker::Repo as MediaTrackerRepo,
};

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

struct AncestorVisitor<'r, Repo> {
    collection_id: CollectionId,
    content_path_resolver: &'r VfsResolver,
    content_paths: Vec<ContentPath<'static>>,
    _repo_marker: PhantomData<Repo>,
}

impl<'r, Repo> AncestorVisitor<'r, Repo> {
    #[must_use]
    const fn new(collection_id: CollectionId, content_path_resolver: &'r VfsResolver) -> Self {
        Self {
            collection_id,
            content_path_resolver,
            content_paths: Vec::new(),
            _repo_marker: PhantomData,
        }
    }
}

impl<'r, Repo> visit::AncestorVisitor<Repo, Vec<ContentPath<'static>>, anyhow::Error>
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
        match repo.media_tracker_resolve_source_id_synchronized_at_by_content_path(
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

    fn finish(self) -> Vec<ContentPath<'static>> {
        self.content_paths
    }
}

struct DirectoryVisitor<'a, Repo> {
    collection_id: CollectionId,
    resolver: &'a RemappingVfsResolver,
    content_paths: Vec<ContentPath<'static>>,
    repo: PhantomData<Repo>,
}

impl<'a, Repo> DirectoryVisitor<'a, Repo> {
    fn new(collection_id: CollectionId, resolver: &'a RemappingVfsResolver) -> Self {
        Self {
            collection_id,
            resolver,
            content_paths: Default::default(),
            repo: PhantomData,
        }
    }

    fn finish(self) -> Vec<ContentPath<'static>> {
        self.content_paths
    }
}

impl<'a, Repo> aoide_media_file::fs::visit::DirectoryVisitor for DirectoryVisitor<'a, Repo> {
    type AncestorVisitor = AncestorVisitor<'a, Repo>;
    type AncestorFinished = Vec<ContentPath<'static>>;
    type AfterAncestorFinishedError = anyhow::Error;

    fn new_ancestor_visitor(&mut self, _dir_entry: &walkdir::DirEntry) -> Self::AncestorVisitor {
        AncestorVisitor::new(self.collection_id, self.resolver.canonical_resolver())
    }

    fn after_ancestor_finished(
        &mut self,
        _path: &Path,
        mut content_paths: Vec<ContentPath<'static>>,
    ) -> anyhow::Result<visit::AfterAncestorFinished> {
        self.content_paths.append(&mut content_paths);
        Ok(visit::AfterAncestorFinished::Continue)
    }
}

#[allow(clippy::missing_panics_doc)] // Never panics
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
    let Some(resolver) = &collection_ctx.content_path.resolver else {
        let path_kind = collection_ctx.content_path.kind;
        return Err(anyhow::anyhow!("unsupported path kind: {path_kind:?}").into());
    };
    let mut directory_visitor = DirectoryVisitor::new(collection_ctx.record_id, resolver);
    let root_file_path = resolver.build_file_path(resolver.root_path());
    let completion = visit::visit_directories(
        repo,
        &root_file_path,
        *max_depth,
        abort_flag,
        &mut directory_visitor,
        &mut |progress_event| {
            log::trace!("{progress_event:?}");
            report_progress_fn(progress_event.clone().into());
        },
    )
    .map(|mut progress_event| {
        progress_event.try_finish();
        report_progress_fn(progress_event.clone().into());
        let elapsed = progress_event.elapsed_since_started();
        let outcome = progress_event.finalize();
        log::info!(
            "Finding {num_untracked_dir_entries} untracked directory entries in \
             '{root_file_path}' took {elapsed_secs} s",
            num_untracked_dir_entries = directory_visitor.content_paths.len(),
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
    let content_paths = directory_visitor.finish();
    let (root_url, root_path) = collection_ctx
        .content_path
        .resolver
        .map(RemappingVfsResolver::dismantle)
        .expect("collection with path kind VFS");
    Ok(Outcome {
        root_url,
        root_path,
        completion,
        content_paths,
    })
}
