// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use std::{fs, io, path::Path, result::Result as StdResult, sync::atomic::AtomicBool};

use digest::Digest;

use crate::{
    util::digest::*,
    {Error, Result},
};

use super::visit::{
    visit_directories, AfterAncestorFinished, AncestorVisitor, Outcome, ProgressEvent,
};

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

struct AncestorDigest<D> {
    digest: D,
}

impl<D: Digest> AncestorVisitor<digest::Output<D>, Error> for AncestorDigest<D> {
    fn visit_dir_entry(&mut self, dir_entry: &walkdir::DirEntry) -> Result<()> {
        digest_walkdir_entry_for_detecting_changes(&mut self.digest, dir_entry).map_err(Into::into)
    }
    fn finalize(self) -> digest::Output<D> {
        self.digest.finalize()
    }
}

pub fn hash_directories<
    D: Digest,
    E: Into<Error>,
    NewDigestFn: FnMut() -> D,
    DigestFinishedFn: FnMut(&Path, digest::Output<D>) -> StdResult<AfterAncestorFinished, E>,
    ReportProgressFn: FnMut(&ProgressEvent),
>(
    root_path: &Path,
    max_depth: Option<usize>,
    abort_flag: &AtomicBool,
    new_digest_fn: &mut NewDigestFn,
    digest_finished_fn: &mut DigestFinishedFn,
    report_progress_fn: &mut ReportProgressFn,
) -> Result<Outcome> {
    log::info!("Digesting all directories in '{}'", root_path.display());
    let mut new_ancestor_visitor = |_: &_| AncestorDigest {
        digest: new_digest_fn(),
    };
    visit_directories(
        root_path,
        max_depth,
        abort_flag,
        &mut new_ancestor_visitor,
        digest_finished_fn,
        report_progress_fn,
    )
    .map(|mut progress_event| {
        progress_event.finish();
        report_progress_fn(&progress_event);
        let elapsed = progress_event.elapsed_since_started();
        let outcome = progress_event.finalize();
        log::info!(
            "Digesting {} directories in '{}' took {} s",
            outcome.progress.directories.finished,
            root_path.display(),
            elapsed.as_millis() as f64 / 1000.0,
        );
        outcome
    })
}
