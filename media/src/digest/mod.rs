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

use crate::Result;

use anyhow::anyhow;
use bytes::BufMut as _;
use digest::Digest;
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use walkdir::WalkDir;

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
) -> walkdir::Result<()> {
    digest_fs_metadata_for_detecting_changes(digest, &dir_entry.metadata()?);
    digest_os_str(digest, dir_entry.file_name());
    Ok(())
}

pub fn index_directories_recursively<D: Digest, F: Fn() -> D>(
    root_path: &Path,
    expected_number_of_directories: usize,
    new_digest: F,
) -> Result<HashMap<PathBuf, digest::Output<D>>> {
    if !root_path.is_dir() {
        return Err(anyhow!("Root path '{}' is not a directory", root_path.display()).into());
    }
    log::info!("Indexing all directories in '{}'", root_path.display());
    let started = Instant::now();
    let mut in_progress: HashMap<PathBuf, D> = HashMap::with_capacity(32); // capacity <= max depth (contents first = depth first search)
    let mut finished = HashMap::with_capacity(expected_number_of_directories);
    for dir_entry in WalkDir::new(root_path).contents_first(true).into_iter() {
        let dir_entry = match dir_entry {
            Ok(dir_entry) => dir_entry,
            Err(err) => {
                log::warn!("Skipping directory entry: {}", err);
                // Keep going
                continue;
            }
        };

        if dir_entry.path() == root_path {
            // Skip the root directory
            // The (default) parameter min_depth=0 is still required to
            // properly finish all in-progress child directories, i.e.
            // performing the traversal with min_depth=1 would NOT work!
            // The first visited child of the root directory remains
            // in-progress until the very end and is only finished
            // regularly with min_depth=0.
            continue;
        }

        // TODO: Allow to pass a (regex?) pattern for excluded file names
        // and skip those files here.

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
        if !parent_path.as_os_str().is_empty() {
            // TODO: Switch to HashMap::raw_entry_mut() for maximum performance?
            if let Some(digest) = in_progress.get_mut(parent_path) {
                // Parent directory is already in progress
                digest_walkdir_entry_for_detecting_changes(digest, &dir_entry)?;
            } else {
                // Insert a new digest for the parent directory
                debug_assert!(!finished.contains_key(parent_path));
                log::debug!("Found non-empty directory: {}", parent_path.display());
                let mut digest = new_digest();
                digest_walkdir_entry_for_detecting_changes(&mut digest, &dir_entry)?;
                in_progress.insert(parent_path.to_path_buf(), digest);
            }
        }

        // A directory is visited after all its childs have been visited (depth-first search)
        if dir_entry.file_type().is_dir() {
            if let Ok(path) = dir_entry.path().strip_prefix(root_path) {
                if let Some((path, digest)) = in_progress.remove_entry(path) {
                    debug_assert!(!finished.contains_key(&path));
                    let digest = digest.finalize();
                    log::debug!("Finished non-empty directory: {}", path.display());
                    finished.insert(path, digest);
                } else {
                    log::debug!("Skipping empty directory: {}", path.display());
                }
            } else {
                log::warn!(
                    "Skipping directory entry with out-of-tree path: {}",
                    dir_entry.path().display()
                );
            }
        }
    }
    debug_assert!(in_progress.is_empty());
    let elapsed = started.elapsed();
    log::info!(
        "Indexing {} directories in '{}' took {} s",
        finished.len(),
        root_path.display(),
        elapsed.as_millis() as f64 / 1000.0,
    );
    Ok(finished)
}
