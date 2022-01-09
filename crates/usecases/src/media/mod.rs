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

use std::io::BufReader;

use aoide_core::{media::SourcePath, track::Track, util::clock::DateTime};

use aoide_core_api::media::SyncMode;

use aoide_media::{
    fs::{file_last_modified_at, open_file_for_reading},
    io::{
        export::{export_track_to_path, ExportTrackConfig},
        import::*,
    },
    resolver::VirtualFilePathResolver,
    util::guess_mime_from_path,
};

use super::*;

pub mod source;
pub mod tracker;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SyncStatus {
    Once {
        synchronized_before: bool,
    },
    Modified {
        last_synchronized_at: Option<DateTime>,
    },
    Always,
}

impl SyncStatus {
    #[must_use]
    pub const fn new(sync_mode: SyncMode, last_synchronized_at: Option<DateTime>) -> Self {
        match sync_mode {
            SyncMode::Once => Self::Once {
                synchronized_before: last_synchronized_at.is_some(),
            },
            SyncMode::Modified => Self::Modified {
                last_synchronized_at,
            },
            SyncMode::Always => Self::Always,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum ImportTrackFromFileOutcome {
    Imported { track: Track, issues: Issues },
    SkippedSynchronized(DateTime),
    SkippedDirectory,
}

pub fn import_track_from_file_path(
    source_path_resolver: &VirtualFilePathResolver,
    source_path: SourcePath,
    sync_status: SyncStatus,
    config: &ImportTrackConfig,
    collected_at: DateTime,
) -> Result<ImportTrackFromFileOutcome> {
    let file_path = source_path_resolver.build_file_path(&source_path);
    let (canonical_path, file) =
        if let Some((canonical_path, file)) = open_file_for_reading(&file_path)? {
            (canonical_path, file)
        } else {
            log::debug!("{} is a directory", file_path.display());
            return Ok(ImportTrackFromFileOutcome::SkippedDirectory);
        };
    let last_modified_at = file_last_modified_at(&file).unwrap_or_else(|_| {
        log::error!(
            "Using current time instead of inaccessible last modification time for file {:?}",
            file
        );
        DateTime::now_utc()
    });
    match sync_status {
        SyncStatus::Once {
            synchronized_before,
        } => {
            if synchronized_before {
                log::debug!(
                    "Skipping reimport of file {} last modified at {}",
                    canonical_path.display(),
                    last_modified_at,
                );
                return Ok(ImportTrackFromFileOutcome::SkippedSynchronized(
                    last_modified_at,
                ));
            }
        }
        SyncStatus::Modified {
            last_synchronized_at,
        } => {
            if let Some(last_synchronized_at) = last_synchronized_at {
                if last_modified_at <= last_synchronized_at {
                    log::debug!(
                        "Skipping reimport of synchronized file {} modified at {} <= {}",
                        canonical_path.display(),
                        last_modified_at,
                        last_synchronized_at
                    );
                    return Ok(ImportTrackFromFileOutcome::SkippedSynchronized(
                        last_modified_at,
                    ));
                }
            } else {
                log::debug!(
                    "Last synchronization of file {} modified at {} is unknown",
                    canonical_path.display(),
                    last_modified_at
                );
            }
        }
        SyncStatus::Always => {
            // Continue regardless of last_modified_at
        }
    }
    let input = NewTrackInput {
        collected_at,
        synchronized_at: last_modified_at,
    };
    let mut reader: Box<dyn Reader> = Box::new(BufReader::new(file));
    let mime = guess_mime_from_path(&canonical_path)?;
    let mut track = input.into_new_track(source_path, mime);
    let issues = import_into_track(&mut reader, config, &mut track)?;
    Ok(ImportTrackFromFileOutcome::Imported { track, issues })
}

/// Export track metadata into file tags.
///
/// The parameter `update_source_synchronized_at` controls if the synchronization
/// time stamp of the media source is updated immediately or deferred until the
/// next re-import. Deferring the update enforces a re-import ensures that
/// the file tags remain the single source of truth for all track metadata
/// by completing this round trip.
pub fn export_track_metadata_into_file(
    source_path_resolver: &VirtualFilePathResolver,
    config: &ExportTrackConfig,
    track: &mut Track,
    update_source_synchronized_at: bool,
) -> Result<()> {
    let file_path = source_path_resolver.build_file_path(&track.media_source.path);
    let mut source_synchronized_at = DateTime::now_utc();
    if !export_track_to_path(&file_path, config, track)? {
        // Unmodified and not exported
        return Ok(());
    }
    if !update_source_synchronized_at {
        // Defer update of synchronization time stamp until next re-import
        return Ok(());
    }
    // Update the synchronization time stamp immediately
    match open_file_for_reading(&file_path) {
        Ok(Some((_canonical_path, file))) => match file_last_modified_at(&file) {
            Ok(last_modified_at) => {
                if source_synchronized_at <= last_modified_at {
                    source_synchronized_at = last_modified_at;
                } else {
                    log::warn!(
                        "Last modification time of file {:?} has not been updated while exporting track metadata",
                        file
                    );
                }
            }
            Err(err) => {
                log::error!(
                    "Failed to obtain last modification time for file {:?} after exporting track metadata: {}",
                    file,
                    err,
                );
            }
        },
        Ok(None) => {
            log::error!(
                "Invalid file path {:?} after exporting track metadata",
                file_path.display(),
            );
        }
        Err(err) => {
            log::error!(
                "Failed to open file path {} for reading after exporting track metadata: {}",
                file_path.display(),
                err
            );
        }
    }
    track.media_source.synchronized_at = Some(source_synchronized_at);
    Ok(())
}
