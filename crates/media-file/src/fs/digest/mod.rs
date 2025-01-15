// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    fs, io, marker::PhantomData, path::Path, result::Result as StdResult, sync::atomic::AtomicBool,
};

use digest::Digest;

use aoide_core::util::fs::DirPath;

use super::visit::{
    visit_directories, AfterAncestorFinished, AncestorVisitor, DirectoryVisitor, Outcome,
    ProgressEvent,
};
use crate::{util::digest::*, Error, Result};

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
    digest.update([flags]);
    // File size
    digest_u64(digest, fs_metadata.len());
    // Time stamps
    if let Ok(created) = fs_metadata.created() {
        digest_system_time(digest, created);
    }
    if let Ok(modified) = fs_metadata.modified() {
        digest_system_time(digest, modified);
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

#[derive(Debug)]
pub struct AncestorDigest<D> {
    digest: D,
}

impl<D: Digest> AncestorVisitor<(), digest::Output<D>, Error> for AncestorDigest<D> {
    fn visit_dir_entry(&mut self, (): &mut (), dir_entry: &walkdir::DirEntry) -> Result<()> {
        digest_walkdir_entry_for_detecting_changes(&mut self.digest, dir_entry).map_err(Into::into)
    }

    fn finish(self) -> digest::Output<D> {
        self.digest.finalize()
    }
}

#[derive(Debug)]
pub struct HashDirectoryVisitor<D, E, F1, F2> {
    new_digest_fn: F1,
    digest_finished_fn: F2,
    digest: PhantomData<D>,
    error: PhantomData<E>,
}

impl<D, E, F1, F2> HashDirectoryVisitor<D, E, F1, F2> {
    #[must_use]
    pub const fn new(new_digest_fn: F1, digest_finished_fn: F2) -> Self {
        Self {
            new_digest_fn,
            digest_finished_fn,
            digest: PhantomData,
            error: PhantomData,
        }
    }
}

impl<D, E, F1, F2> DirectoryVisitor for HashDirectoryVisitor<D, E, F1, F2>
where
    D: Digest,
    E: Into<Error>,
    F1: FnMut() -> D,
    F2: FnMut(&Path, digest::Output<D>) -> std::result::Result<AfterAncestorFinished, E>,
{
    type AncestorVisitor = AncestorDigest<D>;
    type AncestorFinished = digest::Output<D>;
    type AfterAncestorFinishedError = Error;

    fn new_ancestor_visitor(&mut self, _dir_entry: &walkdir::DirEntry) -> Self::AncestorVisitor {
        let digest = (self.new_digest_fn)();
        AncestorDigest { digest }
    }

    fn after_ancestor_finished(
        &mut self,
        path: &Path,
        ancestor_finished: Self::AncestorFinished,
    ) -> std::result::Result<AfterAncestorFinished, Self::AfterAncestorFinishedError> {
        (self.digest_finished_fn)(path, ancestor_finished).map_err(Into::into)
    }
}

pub fn hash_directories<
    D: Digest,
    E: Into<Error>,
    NewDigestFn: FnMut() -> D,
    DigestFinishedFn: FnMut(&Path, digest::Output<D>) -> StdResult<AfterAncestorFinished, E>,
    ReportProgressFn: FnMut(&ProgressEvent),
>(
    root_path: &DirPath<'_>,
    excluded_paths: &[DirPath<'_>],
    max_depth: Option<usize>,
    abort_flag: &AtomicBool,
    directory_visitor: &mut HashDirectoryVisitor<D, E, NewDigestFn, DigestFinishedFn>,
    report_progress_fn: &mut ReportProgressFn,
) -> Result<Outcome> {
    log::info!(
        "Digesting all directories in '{root_path}'",
        root_path = root_path.display()
    );
    visit_directories(
        &mut (),
        root_path,
        excluded_paths,
        max_depth,
        abort_flag,
        directory_visitor,
        report_progress_fn,
    )
    .map(|mut progress_event| {
        progress_event.try_finish();
        report_progress_fn(&progress_event);
        let elapsed = progress_event.elapsed_since_started();
        let outcome = progress_event.finalize();
        log::info!(
            "Digesting {finished} directories in '{root_path}' took {elapsed} s",
            finished = outcome.progress.directories.finished,
            root_path = root_path.display(),
            elapsed = elapsed.as_secs_f64(),
        );
        outcome
    })
}
