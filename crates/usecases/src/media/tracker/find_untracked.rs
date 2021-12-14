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

use aoide_core::util::url::BaseUrl;

use aoide_core_ext::media::tracker::{
    find_untracked::Outcome, Completion, FsTraversalDirectoriesProgress,
    FsTraversalEntriesProgress, FsTraversalParams, FsTraversalProgress,
};

use aoide_media::{
    fs::visit::{self, url_from_walkdir_entry},
    resolver::SourcePathResolver,
};

use aoide_repo::{collection::RecordId as CollectionId, media::tracker::Repo as MediaTrackerRepo};

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
    source_path_resolver: &'r VirtualFilePathResolver,
    collection_id: CollectionId,
    source_paths: Vec<SourcePath>,
}

impl<'r, Repo> AncestorVisitor<'r, Repo> {
    pub fn new(
        repo: &'r Repo,
        source_path_resolver: &'r VirtualFilePathResolver,
        collection_id: CollectionId,
    ) -> Self {
        Self {
            repo,
            source_path_resolver,
            collection_id,
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

pub fn visit_directories<Repo>(
    repo: &Repo,
    source_path_resolver: &VirtualFilePathResolver,
    collection_id: CollectionId,
    params: &FsTraversalParams,
    report_progress: &mut impl FnMut(ProgressEvent),
    abort_flag: &AtomicBool,
) -> Result<Outcome>
where
    Repo: MediaTrackerRepo,
{
    let FsTraversalParams {
        root_url,
        max_depth,
    } = params;
    let root_path_prefix = root_url
        .as_ref()
        .map(|url| resolve_path_prefix_from_base_url(source_path_resolver, url))
        .transpose()?
        .unwrap_or_default();
    let root_url = source_path_resolver
        .resolve_url_from_path(&root_path_prefix)
        .map_err(anyhow::Error::from)?;
    let root_url = BaseUrl::new(root_url);
    let root_path = source_path_resolver.build_file_path(&root_path_prefix);
    let mut source_paths = Vec::new();
    let completion = visit::visit_directories(
        &root_path,
        *max_depth,
        abort_flag,
        &mut |_| AncestorVisitor::new(repo, source_path_resolver, collection_id),
        &mut |_path, untracked_source_paths| {
            ancestor_finished(&mut source_paths, untracked_source_paths)
        },
        &mut |progress_event| {
            tracing::trace!("{:?}", progress_event);
            report_progress(progress_event.to_owned().into());
        },
    )
    .map_err(anyhow::Error::from)
    .map(|mut progress_event| {
        progress_event.finish();
        report_progress(progress_event.clone().into());
        let elapsed = progress_event.elapsed_since_started();
        let outcome = progress_event.finalize();
        tracing::info!(
            "Finding {} untracked directory entries in '{}' took {} s",
            source_paths.len(),
            root_path.display(),
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
    Ok(Outcome {
        root_url,
        completion,
        source_paths,
    })
}
