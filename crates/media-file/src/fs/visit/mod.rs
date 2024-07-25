// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    fs::read_link,
    ops::Not as _,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

use url::Url;
use walkdir::{DirEntry, WalkDir};

use aoide_core::util::fs::DirPath;

use crate::{Error, Result};

// TODO: Customize the hidden directories filter?
const HIDDEN_DIR_NAMES: [&str; 2] = [".DS_Store", ".git"];

fn is_hidden_dir_entry(dir_entry: &DirEntry) -> bool {
    if dir_entry.file_type().is_dir() {
        return dir_entry
            .file_name()
            .to_str()
            .map_or(false, |dir_name| HIDDEN_DIR_NAMES.contains(&dir_name));
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

#[allow(clippy::missing_panics_doc)] // Never panics
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

    #[must_use]
    fn finish(self) -> T;
}

pub trait DirectoryVisitor {
    type AncestorVisitor;
    type AncestorFinished;
    type AfterAncestorFinishedError;

    #[must_use]
    fn new_ancestor_visitor(&mut self, dir_entry: &walkdir::DirEntry) -> Self::AncestorVisitor;

    fn after_ancestor_finished(
        &mut self,
        path: &Path,
        ancestor_finished: Self::AncestorFinished,
    ) -> std::result::Result<AfterAncestorFinished, Self::AfterAncestorFinishedError>;
}

/// Visit directories and their entries in depth-first order
///
/// Returns the unfinished progress event that could be finished and
/// finalized by the caller for reporting, i.e. for sending a final
/// update after invoking [`ProgressEvent::try_finish()`] and for obtaining
/// execution statistics by invoking [`ProgressEvent::finalize()`].
#[allow(clippy::too_many_lines)] // TODO
#[allow(clippy::missing_panics_doc)] // Never panics
pub fn visit_directories<
    C,
    T,
    V: AncestorVisitor<C, T, E1>,
    E1: Into<Error>,
    E2: Into<Error>,
    ReportProgressFn: FnMut(&ProgressEvent),
>(
    context: &mut C,
    root_path: &DirPath<'_>,
    excluded_paths: &[DirPath<'_>],
    max_depth: Option<usize>,
    abort_flag: &AtomicBool,
    directory_visitor: &mut impl DirectoryVisitor<
        AncestorVisitor = V,
        AncestorFinished = T,
        AfterAncestorFinishedError = E2,
    >,
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

    for dir_entry in walkdir.into_iter().filter_entry(|e| {
        is_hidden_dir_entry(e).not() && !excluded_paths.iter().any(|p| e.path().starts_with(p))
    }) {
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
                        "Cycle detected while visiting directory: {path}",
                        path = loop_ancestor.display()
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
                    log::warn!("Failed to visit directory: {path}", path = path.display());
                }
                // Propagate I/O error
                let io_error = err.into_io_error();
                debug_assert!(io_error.is_some());
                progress_event.fail();
                report_progress_fn(&progress_event);
                return Err(Error::from(io_error.expect("I/O error")));
            }
        };

        // Get the relative path
        let relative_path = if dir_entry.depth() == 0 {
            // Root directory has an empty relative path and no ancestors
            Path::new("")
        } else {
            let Ok(relative_path) = dir_entry.path().strip_prefix(root_path) else {
                log::warn!(
                    "Skipping entry with out-of-tree path: {path}",
                    path = dir_entry.path().display()
                );
                // Keep going
                progress_event.progress.entries.skipped += 1;
                continue;
            };
            debug_assert!(relative_path.is_relative());

            while let Some((ancestor_path, ancestor_visitor)) = ancestor_visitors.last_mut() {
                #[allow(clippy::needless_borrows_for_generic_args)] // false positive
                if relative_path.starts_with(&ancestor_path) {
                    // Visit child entry
                    log::debug!(
                        "Visiting child entry of {ancestor_path}: {relative_path}",
                        ancestor_path = ancestor_path.display(),
                        relative_path = relative_path.display()
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
                let ancestor_finished = ancestor_visitor.finish();
                log::debug!(
                    "Finalized parent directory: {ancestor_path}",
                    ancestor_path = ancestor_path.display()
                );
                #[allow(clippy::blocks_in_conditions)] // TODO
                match directory_visitor
                    .after_ancestor_finished(&ancestor_path, ancestor_finished)
                    .map_err(|err| {
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
                            "Aborting directory tree traversal after finishing parent directory: \
                             {path}",
                            path = ancestor_path.display()
                        );
                        progress_event.abort();
                        report_progress_fn(&progress_event);
                        return Ok(progress_event);
                    }
                }
            }
            relative_path
        };

        // Checking for `is_dir()` is sufficient when following symlinks
        debug_assert!(follow_links);
        if dir_entry.file_type().is_dir() {
            log::debug!(
                "Adding parent directory: {path}",
                path = relative_path.display()
            );
            let ancestor_visitor = directory_visitor.new_ancestor_visitor(&dir_entry);
            ancestor_visitors.push((relative_path.to_path_buf(), ancestor_visitor));
        } else {
            log::debug!(
                "Finished file entry: {path}",
                path = relative_path.display()
            );
            progress_event.progress.entries.finished += 1;
        }
    }

    // Stack unwinding of remaining ancestors
    while let Some((ancestor_path, ancestor_visitor)) = ancestor_visitors.pop() {
        let ancestor_data = ancestor_visitor.finish();
        log::debug!(
            "Finalized parent directory: {path}",
            path = ancestor_path.display()
        );
        #[allow(clippy::blocks_in_conditions)] // TODO
        match directory_visitor
            .after_ancestor_finished(&ancestor_path, ancestor_data)
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
                    "Aborting directory tree traversal after finishing parent directory: {path}",
                    path = ancestor_path.display()
                );
                progress_event.abort();
                report_progress_fn(&progress_event);
                return Ok(progress_event);
            }
        }
    }
    Ok(progress_event)
}
