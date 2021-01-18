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

use std::{
    io::BufReader,
    path::{Path, PathBuf},
};

use super::*;

use aoide_core::{track::Track, util::clock::DateTime};

use aoide_media::{
    digest as media_digest, guess_mime_from_url, mp4, open_local_file_url_for_reading, ImportTrack,
    ImportTrackConfig, ImportTrackOptions, NewTrackInput, Reader,
};

use url::Url;

///////////////////////////////////////////////////////////////////////

pub type PathDigest = [u8; 32];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathWithDigest {
    pub path: PathBuf,
    pub digest: PathDigest,
}

pub fn index_directories_recursively(
    root_path: &Path,
    expected_number_of_directories: usize,
) -> Result<Vec<PathWithDigest>> {
    media_digest::index_directories_recursively(
        root_path,
        expected_number_of_directories.max(1024),
        blake3::Hasher::new,
    )
    .map(|v| {
        v.into_iter()
            .map(|(path, digest)| PathWithDigest {
                path,
                digest: PathDigest::from(digest),
            })
            .collect()
    })
    .map_err(Error::Media)
}

pub fn import_track_from_url(
    url: &Url,
    config: &ImportTrackConfig,
    options: ImportTrackOptions,
) -> Result<Track> {
    let file = open_local_file_url_for_reading(url)?;
    let file_metadata = file.metadata().map_err(MediaError::from)?;
    let mime = guess_mime_from_url(url)?;
    let collected_at = DateTime::now_local();
    let synchronized_at = file_metadata
        .modified()
        .map(DateTime::from)
        .unwrap_or_else(|_| {
            log::debug!("Using current time instead of inaccessible last modification time");
            DateTime::now_utc()
        });
    let input = NewTrackInput {
        collected_at,
        synchronized_at,
    };
    let mut reader: Box<dyn Reader> = Box::new(BufReader::new(file));
    let file_size = file_metadata.len();
    let track = input.try_from_url_into_new_track(url, &mime)?;
    if mime == "audio/m4a" {
        Ok(mp4::ImportTrack.import_track(config, options, track, &mut reader, file_size)?)
    } else {
        Err(Error::Media(MediaError::UnsupportedContentType(mime)))
    }
}
