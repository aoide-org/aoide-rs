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

use aoide_core_ext::media::ImportMode;

use aoide_media::{
    fmt::{flac, mp3, mp4, ogg},
    fs::open_local_file_for_reading,
    io::import::*,
    resolver::VirtualFilePathResolver,
    util::guess_mime_from_path,
};

use super::*;

pub mod source;
pub mod tracker;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SynchronizedImportMode {
    Once {
        synchronized_before: bool,
    },
    Modified {
        last_synchronized_at: Option<DateTime>,
    },
    Always,
}

impl SynchronizedImportMode {
    pub const fn new(import_mode: ImportMode, last_synchronized_at: Option<DateTime>) -> Self {
        match import_mode {
            ImportMode::Once => Self::Once {
                synchronized_before: last_synchronized_at.is_some(),
            },
            ImportMode::Modified => Self::Modified {
                last_synchronized_at,
            },
            ImportMode::Always => Self::Always,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportTrackFromFileOutcome {
    Imported(Track),
    SkippedSynchronized(DateTime),
    SkippedDirectory,
}

pub fn import_track_from_local_file_path(
    source_path_resolver: &VirtualFilePathResolver,
    source_path: SourcePath,
    mode: SynchronizedImportMode,
    config: &ImportTrackConfig,
    flags: ImportTrackFlags,
    collected_at: DateTime,
) -> Result<ImportTrackFromFileOutcome> {
    let file_path = source_path_resolver.build_file_path(&source_path);
    let (canonical_path, file) =
        if let Some((canonical_path, file)) = open_local_file_for_reading(&file_path)? {
            (canonical_path, file)
        } else {
            tracing::debug!("{} is a directory", file_path.display());
            return Ok(ImportTrackFromFileOutcome::SkippedDirectory);
        };
    let file_metadata = file.metadata().map_err(MediaError::from)?;
    let last_modified_at = file_metadata
        .modified()
        .map(DateTime::from)
        .map(|last_modified_at| {
            if last_modified_at.timestamp_millis() > 0 {
                // Only consider time stamps strictly after the epoch origin
                // meaningful and valid
                last_modified_at
            } else {
                tracing::warn!(
                    "Using current time instead of invalid last modification time {}",
                    last_modified_at
                );
                DateTime::now_utc()
            }
        })
        .unwrap_or_else(|_| {
            tracing::error!("Using current time instead of inaccessible last modification time");
            DateTime::now_utc()
        });
    match mode {
        SynchronizedImportMode::Once {
            synchronized_before,
        } => {
            if synchronized_before {
                tracing::debug!(
                    "Skipping reimport of file {} last modified at {}",
                    canonical_path.display(),
                    last_modified_at,
                );
                return Ok(ImportTrackFromFileOutcome::SkippedSynchronized(
                    last_modified_at,
                ));
            }
        }
        SynchronizedImportMode::Modified {
            last_synchronized_at,
        } => {
            if let Some(last_synchronized_at) = last_synchronized_at {
                if last_modified_at <= last_synchronized_at {
                    tracing::debug!(
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
                tracing::debug!(
                    "Last synchronization of file {} modified at {} is unknown",
                    canonical_path.display(),
                    last_modified_at
                );
            }
        }
        SynchronizedImportMode::Always => {
            // Continue regardless of last_modified_at
        }
    }
    let input = NewTrackInput {
        collected_at,
        synchronized_at: last_modified_at,
    };
    let mut reader: Box<dyn Reader> = Box::new(BufReader::new(file));
    let mime = guess_mime_from_path(&canonical_path)?;
    let new_track = input.into_new_track(source_path, &mime);
    let track = match mime.as_ref() {
        "audio/flac" => flac::ImportTrack.import_track(config, flags, new_track, &mut reader),
        "audio/mpeg" => mp3::ImportTrack.import_track(config, flags, new_track, &mut reader),
        "audio/m4a" | "video/mp4" => {
            mp4::ImportTrack.import_track(config, flags, new_track, &mut reader)
        }
        "audio/ogg" => ogg::ImportTrack.import_track(config, flags, new_track, &mut reader),
        _ => Err(MediaError::UnsupportedContentType(mime)),
    }?;
    Ok(ImportTrackFromFileOutcome::Imported(track))
}
