// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    collections::HashSet,
    fs,
    io::{self, BufReader, Read as _},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail};
use same_file::Handle as FileHandle;
use static_assertions::const_assert;
use url::Url;
use walkdir::WalkDir;

use aoide_core::{
    CollectionUid, TrackEntity,
    media::content::{ContentPathConfig, resolver::vfs::VfsResolver},
    util::url::BaseUrl,
};
use aoide_core_api::{Pagination, track::search::Filter};
use aoide_repo::{RecordCollector, ReservableRecordCollector};

use crate::{Error, Result};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum MatchFiles {
    /// Determine equality by comparing contents byte-wise.
    ///
    /// Slow but works reliably in any situation.
    #[default]
    Content,

    /// Determine equality by comparing metadata.
    ///
    /// Only works reliably if source and target files are stored on the same
    /// file system!
    Metadata,
}

#[derive(Debug)]
pub struct ExportTrackFileFailed {
    pub entity: TrackEntity,
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub error: io::Error,
}

#[derive(Debug, Default)]
pub struct ExportTrackFilesOutcome {
    pub exported: u64,
    pub skipped: u64,
    pub failed: Vec<ExportTrackFileFailed>,
    pub purged: Option<u64>,
}

impl ExportTrackFilesOutcome {
    #[must_use]
    #[allow(clippy::missing_panics_doc)] // Should never panic according to the const_assert.
    pub fn total_count(&self) -> u64 {
        let Self {
            exported,
            skipped,
            failed,
            purged,
        } = self;
        const_assert!(std::mem::size_of::<usize>() <= std::mem::size_of::<u64>());
        exported + skipped + u64::try_from(failed.len()).unwrap() + purged.unwrap_or(0)
    }
}

#[derive(Debug)]
struct TrackFileExporter {
    match_files: MatchFiles,
    source_path_resolver: VfsResolver,
    target_path_resolver: VfsResolver,
    canonical_target_paths: HashSet<PathBuf>,
    outcome: ExportTrackFilesOutcome,
}

impl TrackFileExporter {
    #[allow(clippy::manual_let_else)] // Verify that the error is the unit type.
    fn new(
        match_files: MatchFiles,
        source_path_resolver: VfsResolver,
        target_root_path: &Path,
    ) -> anyhow::Result<Self> {
        let target_root_url = match Url::from_file_path(target_root_path) {
            Ok(ok) => ok,
            Err(()) => {
                bail!("invalid target root path");
            }
        };
        let target_root_url = match BaseUrl::try_autocomplete_from(target_root_url) {
            Ok(ok) => ok,
            Err(err) => {
                bail!("invalid target root path: {err}");
            }
        };
        let target_path_resolver = VfsResolver::with_root_url(target_root_url);
        Ok(Self {
            match_files,
            source_path_resolver,
            target_path_resolver,
            canonical_target_paths: Default::default(),
            outcome: Default::default(),
        })
    }
}

fn is_file_content_eq(
    match_files: MatchFiles,
    source_path: &Path,
    target_path: &Path,
) -> io::Result<bool> {
    let source_handle = FileHandle::from_path(source_path)?;
    let target_handle = match FileHandle::from_path(target_path) {
        Ok(target_handle) => target_handle,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            // Target file does not exist.
            return Ok(false);
        }
        Err(err) => {
            // Target path is inaccessible.
            return Err(err);
        }
    };

    // Check for identity.
    if source_handle == target_handle {
        // Trivial special case: Same file.
        return Ok(true);
    }

    let source_file = source_handle.as_file();
    let target_file = target_handle.as_file();

    let source_metadata = source_file.metadata()?;
    let target_metadata = target_file.metadata()?;

    // Compare file size (fast).
    let source_size = source_metadata.len();
    let target_size = target_metadata.len();
    if source_size != target_size {
        // The file contents differ.
        return Ok(false);
    }

    match match_files {
        MatchFiles::Metadata => {
            let source_modified = source_metadata.modified()?;
            let target_modified = target_metadata.modified()?;
            if source_modified != target_modified {
                // The file contents might differ.
                return Ok(false);
            }
        }
        MatchFiles::Content => {
            // Compare file contents (slow).
            let source_file_reader = BufReader::new(source_file);
            let target_file_reader = BufReader::new(target_file);
            for (b1, b2) in source_file_reader.bytes().zip(target_file_reader.bytes()) {
                if b1? != b2? {
                    // The file contents differ.
                    return Ok(false);
                }
            }
        }
    }

    // Both file contents are equal.
    Ok(true)
}

impl RecordCollector for TrackFileExporter {
    type Header = aoide_repo::track::RecordHeader;
    type Record = TrackEntity;

    fn collect(&mut self, _header: Self::Header, entity: Self::Record) {
        let content_path = &entity.body.track.media_source.content.link.path;
        let Self {
            match_files,
            source_path_resolver,
            target_path_resolver,
            canonical_target_paths,
            outcome:
                ExportTrackFilesOutcome {
                    exported,
                    skipped,
                    failed,
                    purged,
                },
        } = self;
        debug_assert!(purged.is_none());
        let source_path = source_path_resolver.build_file_path(content_path);
        let target_path = target_path_resolver.build_file_path(content_path);
        match export_file_content(*match_files, &source_path, &target_path) {
            Ok(Some(_exported_file)) => {
                // Dropping the file handle closes the file.
                *exported += 1;
            }
            Ok(None) => {
                *skipped += 1;
            }
            Err(error) => {
                failed.push(ExportTrackFileFailed {
                    entity,
                    source_path,
                    target_path,
                    error,
                });
                return;
            }
        };
        match target_path.canonicalize() {
            Ok(canonical_target_path) => {
                debug_assert!(!canonical_target_paths.contains(&canonical_target_path));
                canonical_target_paths.insert(canonical_target_path);
            }
            Err(err) => {
                log::warn!(
                    "Failed to canonicalize target path \"{target_path}\": {err}",
                    target_path = target_path.display()
                );
            }
        }
    }
}

impl ReservableRecordCollector for TrackFileExporter {
    fn reserve(&mut self, _additional: usize) {
        // Nothing to do.
    }
}

fn export_file_content(
    match_files: MatchFiles,
    source_path: &Path,
    target_path: &Path,
) -> io::Result<Option<FileHandle>> {
    if is_file_content_eq(match_files, source_path, target_path)? {
        return Ok(None);
    }

    log::info!(
        "Copy file \"{source_path}\" to \"{target_path}\"",
        source_path = source_path.display(),
        target_path = target_path.display()
    );
    if let Some(parent_path) = target_path.parent() {
        fs::create_dir_all(parent_path)?;
    }
    fs::copy(source_path, target_path)?;

    // After successfully copying the file obtaining a handle to it should never fail.
    let target_file = FileHandle::from_path(target_path)?;

    // Adjust the target file's modified timestamp according to that of the source file.
    // Otherwise it might be copied again next time when solely using metadata for matching
    // file contents.
    if let Err(err) = fs::File::open(source_path)
        .and_then(|source_file| source_file.metadata())
        .and_then(|source_metadata| source_metadata.modified())
        .and_then(|source_modified| target_file.as_file().set_modified(source_modified))
    {
        log::warn!("Failed to set modified time stamp of target file: {err}");
    }

    #[cfg(feature = "expensive-debug-assertions")]
    debug_assert_eq!(
        is_file_content_eq(MatchFiles::Content, source_path, target_path).ok(),
        Some(true)
    );

    Ok(Some(target_file))
}

pub fn export_files<R>(
    repo: &mut R,
    collection_uid: &CollectionUid,
    filter: Option<&Filter>,
    batch_size: Option<u64>,
    target_root_path: &Path,
    match_files: MatchFiles,
    purge_other_files: bool,
) -> Result<ExportTrackFilesOutcome>
where
    R: aoide_repo::collection::EntityRepo + aoide_repo::track::CollectionRepo,
{
    let (collection_record_header, collection_entity_with_summary) = crate::collection::load_one(
        repo,
        collection_uid,
        aoide_core_api::collection::LoadScope::Entity,
    )?;
    let source_path_resolver = if let ContentPathConfig::VirtualFilePath(config) =
        &collection_entity_with_summary
            .entity
            .body
            .media_source_config
            .content_path
    {
        VfsResolver::with_root_url(config.root_url.clone())
    } else {
        return Err(Error::Other(anyhow!("unsupported config path config")));
    };
    let collection_id = collection_record_header.id;
    let offset = 0;
    let limit = batch_size.unwrap_or(u64::MAX);
    let mut exporter = TrackFileExporter::new(match_files, source_path_resolver, target_root_path)
        .map_err(Error::Other)?;
    loop {
        let pagination = Pagination {
            limit: Some(limit),
            offset: Some(offset),
        };
        let unordered = &[];
        crate::track::search::search(
            repo,
            collection_id,
            &pagination,
            filter,
            unordered,
            &mut exporter,
        )?;
        let total_count = exporter.outcome.total_count();
        debug_assert!(offset <= total_count);
        let batch_count = total_count - offset;
        debug_assert!(batch_count <= limit);
        if batch_count < limit {
            break;
        }
    }
    let TrackFileExporter {
        match_files: _,
        source_path_resolver: _,
        target_path_resolver: _,
        canonical_target_paths,
        outcome,
    } = exporter;
    // Only purge files if no errors occurred to prevent unintended data loss.
    let mut outcome = outcome;
    if purge_other_files && outcome.failed.is_empty() {
        let mut keep_file = |file_path: &Path| match file_path.canonicalize() {
            Ok(canonical_target_path) => canonical_target_paths.contains(&canonical_target_path),
            Err(err) => {
                log::warn!(
                    "Keeping file \"{file_path}\": {err}",
                    file_path = file_path.display()
                );
                true
            }
        };
        debug_assert!(outcome.purged.is_none());
        outcome.purged = Some(purge_files(target_root_path, &mut keep_file));
    }
    Ok(outcome)
}

fn purge_files(root_path: &Path, keep_file: &mut impl FnMut(&Path) -> bool) -> u64 {
    let mut purged = 0;
    // Resolve and follow symlinks
    let follow_links = true;
    let walkdir = WalkDir::new(root_path).follow_links(follow_links);
    for dir_entry in walkdir {
        let dir_entry = match &dir_entry {
            Ok(dir_entry) => dir_entry,
            Err(err) => {
                log::warn!("Failed to visit directory entry: {err}");
                continue;
            }
        };
        let file_path = dir_entry.path();
        let metadata = match dir_entry.metadata() {
            Ok(metadata) => metadata,
            Err(err) => {
                log::warn!(
                    "Failed to read metadata for directory entry \"{file_path}\": {err}",
                    file_path = file_path.display()
                );
                continue;
            }
        };
        if !metadata.is_file() {
            // Only purge regular files and nothing else.
            log::debug!(
                "Ignore directory entry for \"{file_path}\"",
                file_path = file_path.display()
            );
            continue;
        }
        if keep_file(file_path) {
            continue;
        }
        log::info!(
            "Remove file \"{file_path}\"",
            file_path = file_path.display()
        );
        if let Err(err) = fs::remove_file(file_path) {
            log::warn!(
                "Failed to remove file \"{file_path}\": {err}",
                file_path = file_path.display()
            );
        }
        purged += 1;
    }
    purged
}
