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

use aoide_core::entity::EntityUid;

use aoide_core_api::media::tracker::{
    find_untracked_files::Outcome, Completion, FsTraversalDirectoriesProgress,
    FsTraversalEntriesProgress, FsTraversalParams, FsTraversalProgress,
};

use aoide_media::{
    fs::visit::{self, url_from_walkdir_entry},
    resolver::SourcePathResolver,
};

use aoide_repo::{
    collection::{EntityRepo as CollectionRepo, RecordId as CollectionId},
    media::tracker::Repo as MediaTrackerRepo,
};

use crate::collection::vfs::RepoContext;

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

struct AncestorVisitor<'r, Repo> {
    repo: &'r Repo,
    collection_id: CollectionId,
    source_path_resolver: &'r VirtualFilePathResolver,
    source_paths: Vec<SourcePath>,
}

impl<'r, Repo> AncestorVisitor<'r, Repo> {
    #[must_use]
    pub fn new(
        repo: &'r Repo,
        collection_id: CollectionId,
        source_path_resolver: &'r VirtualFilePathResolver,
    ) -> Self {
        Self {
            repo,
            collection_id,
            source_path_resolver,
            source_paths: Vec::new(),
        }
    }
}

impl<'r, Repo> visit::AncestorVisitor<Vec<SourcePath>, anyhow::Error> for AncestorVisitor<'r, Repo>
where
    Repo: MediaTrackerRepo,
{
    fn visit_dir_entry(&mut self, dir_entry: &walkdir::DirEntry) -> anyhow::Result<()> {
        let url = url_from_walkdir_entry(dir_entry)?;
        let source_path = self.source_path_resolver.resolve_path_from_url(&url)?;
        if !source_path.is_terminal() {
            // Skip non-terminal paths, i.e. directories
            return Ok(());
        }
        match self
            .repo
            .media_tracker_resolve_source_id_synchronized_at_by_path(
                self.collection_id,
                &source_path,
            ) {
            Ok(_) => Ok(()),
            Err(RepoError::NotFound) => {
                self.source_paths.push(source_path);
                Ok(())
            }
            Err(err) => Err(err.into()),
        }
    }
    fn finalize(self) -> Vec<SourcePath> {
        self.source_paths
    }
}

fn ancestor_finished(
    all_source_paths: &mut Vec<SourcePath>,
    mut source_paths: Vec<SourcePath>,
) -> anyhow::Result<visit::AfterAncestorFinished> {
    all_source_paths.append(&mut source_paths);
    Ok(visit::AfterAncestorFinished::Continue)
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
    let FsTraversalParams {
        root_url,
        max_depth,
    } = params;
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
    let root_file_path = vfs_ctx.build_root_file_path();
    let mut source_paths = Vec::new();
    let completion = visit::visit_directories(
        &root_file_path,
        *max_depth,
        abort_flag,
        &mut |_| AncestorVisitor::new(repo, collection_id, &vfs_ctx.path_resolver),
        &mut |_path, untracked_source_paths| {
            ancestor_finished(&mut source_paths, untracked_source_paths)
        },
        &mut |progress_event| {
            log::trace!("{:?}", progress_event);
            report_progress_fn(progress_event.to_owned().into());
        },
    )
    .map_err(anyhow::Error::from)
    .map(|mut progress_event| {
        progress_event.finish();
        report_progress_fn(progress_event.clone().into());
        let elapsed = progress_event.elapsed_since_started();
        let outcome = progress_event.finalize();
        log::info!(
            "Finding {} untracked directory entries in '{}' took {} s",
            source_paths.len(),
            root_file_path.display(),
            elapsed.as_millis() as f64 / 1000.0,
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
        .source_path
        .vfs
        .map(|vfs_context| (vfs_context.root_url, vfs_context.root_path))
        .unwrap();
    Ok(Outcome {
        root_url,
        root_path,
        completion,
        source_paths,
    })
}
