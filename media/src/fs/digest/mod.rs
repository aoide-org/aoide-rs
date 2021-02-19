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

///////////////////////////////////////////////////////////////////////

use crate::{
    util::digest::*,
    {Error, Result},
};

use digest::Digest;
use std::{
    fs, io,
    path::{Path, PathBuf},
    result::Result as StdResult,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
use walkdir::{DirEntry, WalkDir};

/// Fingerprint file metadata for detecting changes on the file.
/// system.
///
/// Only considers properties that are supposed to change when
/// adding, modifying, or deleting files in a file system tree.
///
///  - File type
///  - File size
///  - Creation/modification time stamps
///
/// The following properties are deliberately excluded to avoid
/// false positives when detecting changes:
///  - Access time stamp
///  - Permissions
///
/// The file name is considered by the outer context.
pub fn digest_fs_metadata_for_detecting_changes<D: Digest>(
    digest: &mut D,
    fs_metadata: &fs::Metadata,
) {
    // File type
    let mut flags = 0u8;
    let file_type = fs_metadata.file_type();
    if file_type.is_file() {
        flags |= 0b0001;
    }
    if file_type.is_dir() {
        flags |= 0b0010;
    }
    if file_type.is_symlink() {
        flags |= 0b0100;
    }
    digest.update(&[flags]);
    // File size
    digest_u64(digest, fs_metadata.len());
    // Time stamps
    if let Ok(created) = fs_metadata.created() {
        digest_system_time(digest, created)
    }
    if let Ok(modified) = fs_metadata.modified() {
        digest_system_time(digest, modified)
    }
}

pub fn digest_dir_entry_for_detecting_changes<D: Digest>(
    digest: &mut D,
    dir_entry: &fs::DirEntry,
) -> io::Result<()> {
    digest_fs_metadata_for_detecting_changes(digest, &dir_entry.metadata()?);
    digest_os_str(digest, &dir_entry.file_name());
    Ok(())
}

pub fn digest_walkdir_entry_for_detecting_changes<D: Digest>(
    digest: &mut D,
    dir_entry: &walkdir::DirEntry,
) -> io::Result<()> {
    digest_fs_metadata_for_detecting_changes(digest, &dir_entry.metadata()?);
    digest_os_str(digest, dir_entry.file_name());
    Ok(())
}

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
pub enum AfterDirFinished {
    Continue,
    Abort,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Progress {
    pub entries_skipped: usize,
    pub entries_finished: usize,
    pub directories_finished: usize,
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

pub fn hash_directories<
    D: Digest,
    E: Into<Error>,
    NewDigest: FnMut() -> D,
    DirFinished: FnMut(&PathBuf, digest::Output<D>) -> StdResult<AfterDirFinished, E>,
    ReportProgress: FnMut(&ProgressEvent),
>(
    root_path: &Path,
    max_depth: Option<usize>,
    abort_flag: &AtomicBool,
    mut new_digest: NewDigest,
    mut dir_finished: DirFinished,
    mut report_progress: ReportProgress,
) -> Result<Outcome> {
    log::info!("Digesting all directories in '{}'", root_path.display());

    let started = Instant::now();
    let mut progress_event = ProgressEvent {
        status: Status::InProgress,
        progress: Default::default(),
    };
    let mut walker = || {
        let mut ancestors: Vec<(PathBuf, D)> = Vec::with_capacity(64); // capacity <= max. expected depth
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
                log::debug!("Aborting directory tree traversal");
                progress_event.abort();
                report_progress(&progress_event);
                return Ok(());
            }
            report_progress(&progress_event);
            let dir_entry = match dir_entry {
                Ok(dir_entry) => dir_entry,
                Err(err) => {
                    if let Some(loop_ancestor) = err.loop_ancestor() {
                        log::info!(
                            "Cycle detected while visiting directory: {}",
                            loop_ancestor.display()
                        );
                        // Skip and continue
                        progress_event.progress.entries_skipped += 1;
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
                    report_progress(&progress_event);
                    return Err(Error::from(io_error.expect("I/O error")));
                }
            };

            if dir_entry.depth() == 0 {
                // Skip root directory that has no parent
                progress_event.progress.entries_skipped += 1;
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
                        log::warn!(
                            "Skipping entry with out-of-tree path: {}",
                            dir_entry.path().display()
                        );
                        // Keep going
                        progress_event.progress.entries_skipped += 1;
                        continue;
                    }
                }
            } else {
                // Should never happen
                log::error!(
                    "Skipping entry with no parent directory: {}",
                    dir_entry.path().display()
                );
                // Keep going
                progress_event.progress.entries_skipped += 1;
                continue;
            };

            let mut push_ancestor = true;
            while let Some((ancestor_path, ancestor_digest)) = ancestors.last_mut() {
                if parent_path.starts_with(&ancestor_path) {
                    if parent_path == ancestor_path {
                        // Keep last ancestor on stack and stay in this line of ancestors
                        digest_walkdir_entry_for_detecting_changes(ancestor_digest, &dir_entry)?;
                        progress_event.progress.entries_finished += 1;
                        push_ancestor = false;
                    }
                    break;
                }
                let (ancestor_path, ancestor_digest) = ancestors.pop().expect("last ancestor");
                let ancestor_digest = ancestor_digest.finalize();
                log::trace!("Finished parent directory: {}", ancestor_path.display());
                match dir_finished(&ancestor_path, ancestor_digest).map_err(Into::into)? {
                    AfterDirFinished::Continue => {
                        progress_event.progress.directories_finished += 1;
                    }
                    AfterDirFinished::Abort => {
                        progress_event.progress.directories_finished += 1;
                        log::debug!(
                            "Aborting directory tree traversal after finishing '{}'",
                            ancestor_path.display()
                        );
                        progress_event.abort();
                        report_progress(&progress_event);
                        return Ok(());
                    }
                }
            }
            if push_ancestor {
                log::trace!("Found parent directory: {}", parent_path.display());
                let mut digest = new_digest();
                digest_walkdir_entry_for_detecting_changes(&mut digest, &dir_entry)?;
                progress_event.progress.entries_finished += 1;
                ancestors.push((parent_path.to_path_buf(), digest));
            }
        }
        // Unwind the stack of remaining ancestors
        while let Some((ancestor_path, ancestor_digest)) = ancestors.pop() {
            let ancestor_digest = ancestor_digest.finalize();
            log::trace!("Finished parent directory: {}", ancestor_path.display());
            match dir_finished(&ancestor_path, ancestor_digest).map_err(Into::into)? {
                AfterDirFinished::Continue => {
                    progress_event.progress.directories_finished += 1;
                }
                AfterDirFinished::Abort => {
                    progress_event.progress.directories_finished += 1;
                    progress_event.abort();
                    report_progress(&progress_event);
                    return Ok(());
                }
            }
        }
        progress_event.finish();
        Ok(())
    };
    match walker() {
        Ok(()) => {
            let elapsed = started.elapsed();
            log::info!(
                "Digesting {} directories in '{}' took {} s",
                progress_event.progress.directories_finished,
                root_path.display(),
                elapsed.as_millis() as f64 / 1000.0,
            );
            report_progress(&progress_event);
            Ok(progress_event.finalize())
        }
        Err(err) => {
            progress_event.fail();
            report_progress(&progress_event);
            Err(err)
        }
    }
}
