// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    fs::read_link,
    ops::Not as _,
    path::{Path, PathBuf},
    result::Result as StdResult,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

use url::Url;
use walkdir::{DirEntry, WalkDir};

use crate::{Error, Result};

// TODO: Customize the hidden directories filter?
fn is_hidden_dir_entry(dir_entry: &DirEntry) -> bool {
    if dir_entry.file_type().is_dir() {
        return dir_entry
            .file_name()
            .to_str()
            .map_or(false, |dir_name| dir_name == ".DS_Store");
    }
    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AfterAncestorFinished {
    Continue,
    Abort,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Progress {
    pub entries: EntriesProgress,
    pub directories: DirectoriesProgress,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EntriesProgress {
    pub skipped: usize,
    pub finished: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DirectoriesProgress {
    pub finished: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    InProgress,
    Finished,
    Aborted,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Completion {
    Finished,
    Aborted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub completion: Completion,
    pub progress: Progress,
}

/// A state machine for tracking progress
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgressEvent {
    pub started_at: Instant,
    pub status: Status,
    pub progress: Progress,
}

impl ProgressEvent {
    #[must_use]
    pub fn start() -> Self {
        Self {
            started_at: Instant::now(),
            status: Status::InProgress,
            progress: Default::default(),
        }
    }

    #[must_use]
    pub fn elapsed_since_started(&self) -> Duration {
        self.started_at.elapsed()
    }

    pub fn try_finish(&mut self) -> bool {
        if !matches!(self.status, Status::InProgress) {
            return false;
        }
        self.status = Status::Finished;
        true
    }

    pub fn abort(&mut self) {
        debug_assert_eq!(self.status, Status::InProgress);
        self.status = Status::Aborted;
    }

    pub fn fail(&mut self) {
        debug_assert_eq!(self.status, Status::InProgress);
        self.status = Status::Failed;
    }

    #[must_use]
    pub fn finalize(self) -> Outcome {
        let Self {
            started_at: _,
            status,
            progress,
        } = self;
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

pub fn url_from_walkdir_entry(dir_entry: &walkdir::DirEntry) -> anyhow::Result<Url> {
    let url = if dir_entry.file_type().is_dir()
        || dir_entry.path_is_symlink() && read_link(dir_entry.path())?.is_dir()
    {
        Url::from_directory_path(dir_entry.path())
    } else {
        Url::from_file_path(dir_entry.path())
    }
    .expect("URL");
    Ok(url)
}

pub trait AncestorVisitor<C, T, E> {
    fn visit_dir_entry(
        &mut self,
        context: &mut C,
        dir_entry: &walkdir::DirEntry,
    ) -> std::result::Result<(), E>;
    fn finalize(self) -> T;
}

/// Visit directories and their entries in depth-first order
///
/// Returns the unfinished progress event that could be finished and
/// finalized by the caller for reporting, i.e. for sending a final
/// update after invoking [`ProgressEvent::try_finish()`] and for obtaining
/// execution statistics by invoking [`ProgressEvent::finalize()`].
pub fn visit_directories<
    C,
    T,
    E1: Into<Error>,
    E2: Into<Error>,
    V: AncestorVisitor<C, T, E1>,
    NewAncestorVisitorFn: FnMut(&walkdir::DirEntry) -> V,
    AncestorFinishedFn: FnMut(&Path, T) -> StdResult<AfterAncestorFinished, E2>,
    ReportProgressFn: FnMut(&ProgressEvent),
>(
    context: &mut C,
    root_path: &Path,
    max_depth: Option<usize>,
    abort_flag: &AtomicBool,
    new_ancestor_visitor_fn: &mut NewAncestorVisitorFn,
    ancestor_finished_fn: &mut AncestorFinishedFn,
    report_progress_fn: &mut ReportProgressFn,
) -> Result<ProgressEvent> {
    let mut progress_event = ProgressEvent::start();
    // Capacity <= max. expected depth
    let mut ancestor_visitors: Vec<(PathBuf, V)> = Vec::with_capacity(64);
    // Depth-first traversal to populate ancestors from their child entries
    let contents_first = false;
    // Resolve and follow symlinks
    let follow_links = true;
    // Start with root path
    let min_depth = 0;
    let mut walkdir = WalkDir::new(root_path)
        .contents_first(contents_first)
        .follow_links(follow_links)
        .min_depth(min_depth); // start with root directory (included)
    if let Some(max_depth) = max_depth {
        walkdir = walkdir.max_depth(max_depth);
    }
    for dir_entry in walkdir
        .into_iter()
        .filter_entry(|e| is_hidden_dir_entry(e).not())
    {
        if abort_flag.load(Ordering::Relaxed) {
            log::debug!("Aborting directory tree traversal");
            progress_event.abort();
            report_progress_fn(&progress_event);
            return Ok(progress_event);
        }
        report_progress_fn(&progress_event);
        let dir_entry = match dir_entry {
            Ok(dir_entry) => dir_entry,
            Err(err) => {
                if let Some(loop_ancestor) = err.loop_ancestor() {
                    log::info!(
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
                    log::warn!("Failed to visit directory: {}", path.display());
                }
                // Propagate I/O error
                let io_error = err.into_io_error();
                debug_assert!(io_error.is_some());
                progress_event.fail();
                report_progress_fn(&progress_event);
                return Err(Error::from(io_error.expect("I/O error")));
            }
        };

        if dir_entry.depth() == 0 {
            // Skip root directory that has no parent
            progress_event.progress.entries.skipped += 1;
            continue;
        }

        // Get the relative path
        let relative_path = match dir_entry.path().strip_prefix(root_path) {
            Ok(relative_path) => {
                debug_assert!(relative_path.is_relative());
                relative_path
            }
            Err(_) => {
                log::warn!(
                    "Skipping entry with out-of-tree path: {}",
                    dir_entry.path().display()
                );
                // Keep going
                progress_event.progress.entries.skipped += 1;
                continue;
            }
        };

        while let Some((ancestor_path, ancestor_visitor)) = ancestor_visitors.last_mut() {
            if relative_path.starts_with(&ancestor_path) {
                // Visit child entry
                log::debug!(
                    "Visiting child entry of {}: {}",
                    ancestor_path.display(),
                    relative_path.display()
                );
                ancestor_visitor
                    .visit_dir_entry(context, &dir_entry)
                    .map_err(|err| {
                        progress_event.fail();
                        report_progress_fn(&progress_event);
                        err.into()
                    })?;
                break;
            }
            // Stack unwinding of ancestor
            let (ancestor_path, ancestor_visitor) =
                ancestor_visitors.pop().expect("last ancestor visitor");
            let ancestor_data = ancestor_visitor.finalize();
            log::debug!("Finalized parent directory: {}", ancestor_path.display());
            match ancestor_finished_fn(&ancestor_path, ancestor_data).map_err(|err| {
                progress_event.fail();
                report_progress_fn(&progress_event);
                err.into()
            })? {
                AfterAncestorFinished::Continue => {
                    progress_event.progress.directories.finished += 1;
                }
                AfterAncestorFinished::Abort => {
                    progress_event.progress.directories.finished += 1;
                    log::debug!(
                        "Aborting directory tree traversal after finishing parent directory: {}",
                        ancestor_path.display()
                    );
                    progress_event.abort();
                    report_progress_fn(&progress_event);
                    return Ok(progress_event);
                }
            }
        }
        // Checking for `is_dir()` is sufficient when following symlinks
        debug_assert!(follow_links);
        if dir_entry.file_type().is_dir() {
            log::debug!("Adding parent directory: {}", relative_path.display());
            let ancestor_visitor = new_ancestor_visitor_fn(&dir_entry);
            ancestor_visitors.push((relative_path.to_path_buf(), ancestor_visitor));
        } else {
            log::debug!("Finished file entry: {}", relative_path.display());
            progress_event.progress.entries.finished += 1;
        }
    }
    // Stack unwinding of remaining ancestors
    while let Some((ancestor_path, ancestor_visitor)) = ancestor_visitors.pop() {
        let ancestor_data = ancestor_visitor.finalize();
        log::debug!("Finalized parent directory: {}", ancestor_path.display());
        match ancestor_finished_fn(&ancestor_path, ancestor_data)
            .map_err(Into::into)
            .map_err(|err| {
                progress_event.fail();
                report_progress_fn(&progress_event);
                err
            })? {
            AfterAncestorFinished::Continue => {
                progress_event.progress.directories.finished += 1;
            }
            AfterAncestorFinished::Abort => {
                progress_event.progress.directories.finished += 1;
                log::debug!(
                    "Aborting directory tree traversal after finishing parent directory: {}",
                    ancestor_path.display()
                );
                progress_event.abort();
                report_progress_fn(&progress_event);
                return Ok(progress_event);
            }
        }
    }
    Ok(progress_event)
}
