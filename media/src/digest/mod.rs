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

use crate::{Error, Result};

use bytes::BufMut as _;
use digest::Digest;
use std::{
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    result::Result as StdResult,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use walkdir::{DirEntry, WalkDir};

pub fn digest_u64<D: Digest>(digest: &mut D, val: u64) {
    let mut bytes = [0u8; 8];
    let mut buf = &mut bytes[..];
    buf.put_u64(val);
    digest.update(bytes);
}

pub fn digest_u128<D: Digest>(digest: &mut D, val: u128) {
    let mut bytes = [0u8; 16];
    let mut buf = &mut bytes[..];
    buf.put_u128(val);
    digest.update(bytes);
}

pub fn digest_duration<D: Digest>(digest: &mut D, duration: Duration) {
    digest_u128(digest, duration.as_nanos());
}

pub fn digest_system_time<D: Digest>(digest: &mut D, system_time: SystemTime) {
    digest_duration(
        digest,
        system_time
            .duration_since(UNIX_EPOCH)
            .expect("valid system time not before 1970-01-01 00:00:00 UTC"),
    );
}

pub fn digest_os_str<D: Digest>(digest: &mut D, os_str: &OsStr) {
    if let Some(utf8_str) = os_str.to_str() {
        digest.update(utf8_str.as_bytes());
    } else {
        digest.update(os_str.to_string_lossy().as_bytes());
    }
}

pub fn digest_path<D: Digest>(digest: &mut D, path: &Path) {
    digest_os_str(digest, path.as_os_str());
}

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
pub enum NextDirScanStep {
    Continue,
    Abort,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DirScanOutcome {
    Finished(usize),
    Aborted,
}

pub fn digest_directories_recursively<
    D: Digest,
    N: FnMut() -> D,
    E: Into<Error>,
    R: FnMut(PathBuf, digest::Output<D>) -> StdResult<NextDirScanStep, E>,
>(
    root_path: &Path,
    max_depth: Option<usize>,
    mut new_digest: N,
    mut receive_dir_path_digest: R,
) -> Result<DirScanOutcome> {
    log::info!("Scanning all directories in '{}'", root_path.display());

    let started = Instant::now();
    let mut ancestors: Vec<(PathBuf, D)> = Vec::with_capacity(64); // capacity <= max. expected depth
    let mut total_count = 0;
    let mut walkdir = WalkDir::new(root_path)
        .contents_first(false) // depth-first traversal to populate ancestors
        .follow_links(true)
        .min_depth(1);
    if let Some(max_depth) = max_depth {
        walkdir = walkdir.max_depth(1 + max_depth);
    }
    for dir_entry in walkdir // exclude root folder
        .into_iter()
        .filter_entry(|e| !is_hidden_dir_entry(e))
    {
        let dir_entry = match dir_entry {
            Ok(dir_entry) => dir_entry,
            Err(err) => {
                if let Some(loop_ancestor) = err.loop_ancestor() {
                    log::info!(
                        "Cycle detected while visiting directory: {}",
                        loop_ancestor.display()
                    );
                    // Skip and continue
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
                return Err(io_error.expect("I/O error").into());
            }
        };

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
            continue;
        };

        // Exclude the root directory from indexing
        if parent_path.as_os_str().is_empty() {
            continue;
        }

        let mut push_ancestor = true;
        while let Some((ancestor_path, ancestor_digest)) = ancestors.last_mut() {
            if parent_path.starts_with(&ancestor_path) {
                if parent_path == ancestor_path {
                    // Keep last ancestor on stack and stay in this line of ancestors
                    digest_walkdir_entry_for_detecting_changes(ancestor_digest, &dir_entry)?;
                    push_ancestor = false;
                }
                break;
            }
            let (ancestor_path, ancestor_digest) = ancestors.pop().expect("last ancestor");
            let ancestor_digest = ancestor_digest.finalize();
            log::trace!("Finished non-empty directory: {}", ancestor_path.display());
            match receive_dir_path_digest(ancestor_path, ancestor_digest).map_err(Into::into)? {
                NextDirScanStep::Continue => {
                    total_count += 1;
                }
                NextDirScanStep::Abort => {
                    return Ok(DirScanOutcome::Aborted);
                }
            }
        }
        if push_ancestor {
            log::trace!("Found non-empty directory: {}", parent_path.display());
            let mut digest = new_digest();
            digest_walkdir_entry_for_detecting_changes(&mut digest, &dir_entry)?;
            ancestors.push((parent_path.to_path_buf(), digest));
        }
    }
    // Unwind the stack of remaining ancestors
    while let Some((ancestor_path, ancestor_digest)) = ancestors.pop() {
        let ancestor_digest = ancestor_digest.finalize();
        log::trace!("Finished non-empty directory: {}", ancestor_path.display());
        match receive_dir_path_digest(ancestor_path, ancestor_digest).map_err(Into::into)? {
            NextDirScanStep::Continue => {
                total_count += 1;
            }
            NextDirScanStep::Abort => {
                return Ok(DirScanOutcome::Aborted);
            }
        }
    }
    let elapsed = started.elapsed();
    log::info!(
        "Indexing {} directories in '{}' took {} s",
        total_count,
        root_path.display(),
        elapsed.as_millis() as f64 / 1000.0,
    );
    Ok(DirScanOutcome::Finished(total_count))
}
