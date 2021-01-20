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

use aoide_core::{entity::EntityUid, track::Track, util::clock::DateTime};

use aoide_media::{
    digest as media_digest, guess_mime_from_url, mp4, open_local_file_url_for_reading, ImportTrack,
    ImportTrackConfig, ImportTrackOptions, NewTrackInput, Reader,
};

use aoide_repo::{collection::EntityRepo as _, media::source::Repo as _};

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
    let mut path_with_digests = Vec::with_capacity(expected_number_of_directories);
    media_digest::digest_directories_recursively(root_path, blake3::Hasher::new, |path, digest| {
        path_with_digests.push(PathWithDigest {
            path,
            digest: PathDigest::from(digest),
        });
    })
    .map_err(Error::Media)?;
    Ok(path_with_digests)
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

pub fn relocate_collected_sources(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    old_uri_prefix: &str,
    new_uri_prefix: &str,
) -> Result<usize> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, DieselRepoError, _>(|| {
        let collection_id = db.resolve_collection_id(collection_uid)?;
        let updated_at = DateTime::now_utc();
        Ok(db.relocate_media_sources_by_uri_prefix(
            updated_at,
            collection_id,
            old_uri_prefix,
            new_uri_prefix,
        )?)
    })
    .map_err(Into::into)
}
