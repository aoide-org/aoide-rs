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

use std::{
    io,
    path::{Path, PathBuf},
    result::Result as StdResult,
    sync::atomic::{AtomicBool, Ordering},
};

use walkdir::{DirEntry, WalkDir};

use crate::{Error, Result};

// TODO: Customize the hidden directories filter?
fn is_hidden_dir_entry(dir_entry: &DirEntry) -> bool {
    if dir_entry.file_type().is_dir() {
        return dir_entry
            .file_name()
            .to_str()
            .map(|dir_name| dir_name == ".DS_Store")
            .unwrap_or(false);
    }
    false
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AfterAncestorFinished {
    Continue,
    Abort,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Progress {
    pub entries: EntriesProgress,
    pub directories: DirectoriesProgress,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct EntriesProgress {
    pub skipped: usize,
    pub finished: usize,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct DirectoriesProgress {
    pub finished: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Status {
    InProgress,
    Finished,
    Aborted,
    Failed,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Completion {
    Finished,
    Aborted,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Outcome {
    pub completion: Completion,
    pub progress: Progress,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProgressEvent {
    pub status: Status,
    pub progress: Progress,
}

impl ProgressEvent {
    pub fn finish(&mut self) {
        debug_assert_eq!(self.status, Status::InProgress);
        self.status = Status::Finished;
    }

    pub fn abort(&mut self) {
        debug_assert_eq!(self.status, Status::InProgress);
        self.status = Status::Aborted;
    }

    pub fn fail(&mut self) {
        debug_assert_eq!(self.status, Status::InProgress);
        self.status = Status::Failed;
    }

    pub fn finalize(self) -> Outcome {
        let Self { status, progress } = self;
        let completion = match status {
            Status::InProgress => {
                unreachable!("still in progress");
            }
            Status::Failed => {
                unreachable!("failed");
            }
            Status::Finished => Completion::Finished,
            Status::Aborted => Completion::Aborted,
        };
        Outcome {
            completion,
            progress,
        }
    }
}

pub trait AncestorVisitor<T> {
    fn visit_dir_entry(&mut self, dir_entry: &walkdir::DirEntry) -> io::Result<()>;
    fn finalize(self) -> T;
}

pub fn visit_directories<
    T,
    V: AncestorVisitor<T>,
    E: Into<Error>,
    NewAncestorVistor: FnMut(&walkdir::DirEntry) -> V,
    AncestorFinished: FnMut(&Path, T) -> StdResult<AfterAncestorFinished, E>,
    ReportProgress: FnMut(&ProgressEvent),
>(
    root_path: &Path,
    max_depth: Option<usize>,
    abort_flag: &AtomicBool,
    new_ancestor_visitor: &mut NewAncestorVistor,
    ancestor_finished: &mut AncestorFinished,
    report_progress: &mut ReportProgress,
) -> Result<ProgressEvent> {
    let mut progress_event = ProgressEvent {
        status: Status::InProgress,
        progress: Default::default(),
    };
    let mut ancestor_visitors: Vec<(PathBuf, V)> = Vec::with_capacity(64); // capacity <= max. expected depth
    let mut walkdir = WalkDir::new(root_path)
        .contents_first(false) // depth-first traversal to populate ancestors
        .follow_links(true) // digest metadata of actual files/directories, not symbolic links
        .min_depth(0); // start with root directory (included)
    if let Some(max_depth) = max_depth {
        walkdir = walkdir.max_depth(max_depth);
    }
    for dir_entry in walkdir
        .into_iter()
        .filter_entry(|e| !is_hidden_dir_entry(e))
    {
        if abort_flag.load(Ordering::Relaxed) {
            tracing::debug!("Aborting directory tree traversal");
            progress_event.abort();
            report_progress(&progress_event);
            return Ok(progress_event);
        }
        report_progress(&progress_event);
        let dir_entry = match dir_entry {
            Ok(dir_entry) => dir_entry,
            Err(err) => {
                if let Some(loop_ancestor) = err.loop_ancestor() {
                    tracing::info!(
                        "Cycle detected while visiting directory: {}",
                        loop_ancestor.display()
                    );
                    // Skip and continue
                    progress_event.progress.entries.skipped += 1;
                    continue;
                }
                debug_assert!(err.io_error().is_some());
                debug_assert!(err.path().is_some());
                if let Some(path) = err.path() {
                    // The actual path is probably not mentioned in the I/O error
                    // and should be logged here.
                    // TODO: Propagate the path with the I/O error instead of only
                    // logging it here
                    tracing::warn!("Failed to visit directory: {}", path.display());
                }
                // Propagate I/O error
                let io_error = err.into_io_error();
                debug_assert!(io_error.is_some());
                progress_event.fail();
                report_progress(&progress_event);
                return Err(Error::from(io_error.expect("I/O error")));
            }
        };

        if dir_entry.depth() == 0 {
            // Skip root directory that has no parent
            progress_event.progress.entries.skipped += 1;
            continue;
        }

        // Get the relative parent path
        let parent_path = if let Some(parent_path) = dir_entry.path().parent() {
            match parent_path.strip_prefix(root_path) {
                Ok(parent_path) => {
                    debug_assert!(parent_path.is_relative());
                    parent_path
                }
                Err(_) => {
                    tracing::warn!(
                        "Skipping entry with out-of-tree path: {}",
                        dir_entry.path().display()
                    );
                    // Keep going
                    progress_event.progress.entries.skipped += 1;
                    continue;
                }
            }
        } else {
            // Should never happen
            tracing::error!(
                "Skipping entry with no parent directory: {}",
                dir_entry.path().display()
            );
            // Keep going
            progress_event.progress.entries.skipped += 1;
            continue;
        };

        let mut push_ancestor = true;
        while let Some((ancestor_path, ancestor_visitor)) = ancestor_visitors.last_mut() {
            if parent_path.starts_with(&ancestor_path) {
                if parent_path == ancestor_path {
                    // Keep last ancestor on stack and stay in this line of ancestors
                    ancestor_visitor.visit_dir_entry(&dir_entry)?;
                    progress_event.progress.entries.finished += 1;
                    push_ancestor = false;
                }
                break;
            }
            let (ancestor_path, ancestor_visitor) =
                ancestor_visitors.pop().expect("last ancestor visitor");
            let ancestor_data = ancestor_visitor.finalize();
            tracing::trace!("Finished parent directory: {}", ancestor_path.display());
            match ancestor_finished(&ancestor_path, ancestor_data).map_err(Into::into)? {
                AfterAncestorFinished::Continue => {
                    progress_event.progress.directories.finished += 1;
                }
                AfterAncestorFinished::Abort => {
                    progress_event.progress.directories.finished += 1;
                    tracing::debug!(
                        "Aborting directory tree traversal after finishing '{}'",
                        ancestor_path.display()
                    );
                    progress_event.abort();
                    report_progress(&progress_event);
                    return Ok(progress_event);
                }
            }
        }
        if push_ancestor {
            tracing::trace!("Found parent directory: {}", parent_path.display());
            progress_event.progress.entries.finished += 1;
            // Continue with a new ancestor
            ancestor_visitors.push((parent_path.to_path_buf(), new_ancestor_visitor(&dir_entry)));
        }
    }
    // Unwind the stack of remaining ancestors
    while let Some((ancestor_path, ancestor_visitor)) = ancestor_visitors.pop() {
        let ancestor_data = ancestor_visitor.finalize();
        tracing::trace!("Finished parent directory: {}", ancestor_path.display());
        match ancestor_finished(&ancestor_path, ancestor_data).map_err(Into::into)? {
            AfterAncestorFinished::Continue => {
                progress_event.progress.directories.finished += 1;
            }
            AfterAncestorFinished::Abort => {
                progress_event.progress.directories.finished += 1;
                progress_event.abort();
                report_progress(&progress_event);
                return Ok(progress_event);
            }
        }
    }
    progress_event.finish();
    Ok(progress_event)
}
