// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io::BufReader;

use aoide_core::{
    media::{
        artwork::EditEmbeddedArtworkImage,
        content::{ContentLink, ContentPath, ContentRevision, resolver::vfs::VfsResolver},
    },
    track::Track,
};
use aoide_core_api::media::SyncMode;
use aoide_media_file::{
    fs::open_file_for_reading,
    io::{
        export::{ExportTrackConfig, export_track_to_file_path},
        import::{ImportTrack, ImportTrackConfig, Issues, Reader, import_into_track},
    },
    util::guess_mime_from_file_path,
};

use crate::Result;

pub mod source;
pub mod tracker;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncModeParams {
    Once {
        synchronized_before: bool,
    },
    Modified {
        content_rev: Option<ContentRevision>,
        is_synchronized: bool,
    },
    Always,
}

impl SyncModeParams {
    #[must_use]
    pub fn new(
        sync_mode: SyncMode,
        content_rev: Option<ContentRevision>,
        synchronized_rev: Option<bool>,
    ) -> Self {
        debug_assert!(content_rev.is_some() || synchronized_rev != Some(true));
        match sync_mode {
            SyncMode::Once => Self::Once {
                synchronized_before: content_rev.is_some(),
            },
            SyncMode::Modified => Self::Modified {
                content_rev,
                is_synchronized: content_rev.is_none() || synchronized_rev.unwrap_or(true),
            },
            SyncMode::ModifiedResync => Self::Modified {
                content_rev,
                // Pretend that the revisions are synchronized
                is_synchronized: true,
            },
            SyncMode::Always => Self::Always,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[expect(clippy::large_enum_variant)]
pub enum ImportTrackFromFileOutcome {
    Imported {
        track: Track,
        issues: Issues,
    },
    SkippedSynchronized {
        content_rev: Option<ContentRevision>,
    },
    SkippedUnsynchronized {
        content_rev: ContentRevision,
    },
    SkippedDirectory,
}

pub fn import_track_from_file_path(
    import_track: ImportTrack,
    content_path_resolver: &VfsResolver,
    source_path: &ContentPath<'_>,
    sync_mode_params: &SyncModeParams,
    config: &ImportTrackConfig,
) -> Result<ImportTrackFromFileOutcome> {
    let file_path = content_path_resolver.build_file_path(source_path);
    let Some((canonical_path, file)) = open_file_for_reading(&file_path)? else {
        log::debug!(
            "{file_path} is a directory",
            file_path = file_path.display()
        );
        return Ok(ImportTrackFromFileOutcome::SkippedDirectory);
    };
    let new_content_rev = ContentRevision::try_from_file(&file)?;
    match sync_mode_params {
        SyncModeParams::Once {
            synchronized_before,
        } => {
            if *synchronized_before {
                log::debug!(
                    "Skipping reimport of file {path} that as already been imported once",
                    path = canonical_path.display(),
                );
                return Ok(ImportTrackFromFileOutcome::SkippedSynchronized {
                    content_rev: new_content_rev,
                });
            }
        }
        SyncModeParams::Modified {
            content_rev: old_content_rev,
            is_synchronized,
        } => {
            if let (old_content_rev, Some(new_content_rev)) = (old_content_rev, new_content_rev) {
                if let Some(old_content_rev) = old_content_rev {
                    if new_content_rev <= *old_content_rev {
                        log::debug!(
                            "Skipping reimport of synchronized file {path}",
                            path = canonical_path.display(),
                        );
                        return Ok(ImportTrackFromFileOutcome::SkippedSynchronized {
                            content_rev: Some(new_content_rev),
                        });
                    }
                }
                // If the existing information is synchronized or not only becomes
                // relevant if an import is desired. Checking this later at this
                // point can prevent some irrelevant warnings in the outer context.
                if !is_synchronized {
                    return Ok(ImportTrackFromFileOutcome::SkippedUnsynchronized {
                        content_rev: new_content_rev,
                    });
                }
                if old_content_rev.is_none() {
                    // Consider as modified even if no previous content revision is available.
                    // This happens upon the initial import or after resetting the content
                    // revision manually to selectively enforce a re-import.
                    log::info!(
                        "Importing file {path} with no prior content revision available",
                        path = canonical_path.display()
                    );
                }
            } else {
                log::debug!(
                    "Skipping reimport of file {path} for which no content revision could be \
                     determined",
                    path = canonical_path.display(),
                );
                return Ok(ImportTrackFromFileOutcome::SkippedSynchronized { content_rev: None });
            }
        }
        SyncModeParams::Always => {
            // Continue regardless of last_modified_at and synchronized revision
        }
    }
    log::debug!(
        "Importing file \"{canonical_path}\"",
        canonical_path = canonical_path.display()
    );
    let content_type = guess_mime_from_file_path(&canonical_path)?;
    let content_link = ContentLink {
        path: source_path.clone_owned(),
        rev: new_content_rev,
    };
    let mut track = import_track.with_content(content_link, content_type);
    let mut reader: Box<dyn Reader> = Box::new(BufReader::new(file));
    let issues = import_into_track(&mut reader, config, &mut track)?;
    if issues.is_empty() {
        log::debug!(
            "Finished import of file \"{canonical_path}\" without issues",
            canonical_path = canonical_path.display()
        );
    } else {
        log::warn!(
            "Finished import of file \"{canonical_path}\" with {num_issues} issue(s)",
            canonical_path = canonical_path.display(),
            num_issues = issues.len()
        );
    }
    Ok(ImportTrackFromFileOutcome::Imported { track, issues })
}

/// Export track metadata into file tags.
pub fn export_track_metadata_into_file(
    track: &mut Track,
    content_path_resolver: &VfsResolver,
    config: &ExportTrackConfig,
    edit_embedded_artwork_image: Option<EditEmbeddedArtworkImage>,
) -> Result<()> {
    let file_path = content_path_resolver.build_file_path(&track.media_source.content.link.path);
    export_track_to_file_path(&file_path, None, config, track, edit_embedded_artwork_image)
        .map_err(Into::into)
}
