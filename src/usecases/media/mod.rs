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

use super::*;

use aoide_core::{entity::EntityUid, track::Track, util::clock::DateTime};

use aoide_media::{
    fmt::{flac, mp3, mp4, ogg},
    fs::open_local_file_url_for_reading,
    io::import::*,
    util::guess_mime_from_url,
};

use aoide_repo::{collection::EntityRepo as _, media::source::Repo as _};

use std::io::BufReader;
use url::Url;

///////////////////////////////////////////////////////////////////////

pub mod dir_tracker;

pub fn import_track_from_url(
    url: &Url,
    config: &ImportTrackConfig,
    flags: ImportTrackFlags,
    collected_at: DateTime,
) -> Result<Track> {
    let file = open_local_file_url_for_reading(url)?;
    let file_metadata = file.metadata().map_err(MediaError::from)?;
    let mime = guess_mime_from_url(url)?;
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
    let track = input.try_from_url_into_new_track(url, &mime)?;
    match mime.as_ref() {
        "audio/flac" => Ok(flac::ImportTrack.import_track(config, flags, track, &mut reader)?),
        "audio/mpeg" => Ok(mp3::ImportTrack.import_track(config, flags, track, &mut reader)?),
        "audio/m4a" | "audio/mp4" => {
            Ok(mp4::ImportTrack.import_track(config, flags, track, &mut reader)?)
        }
        "audio/ogg" => Ok(ogg::ImportTrack.import_track(config, flags, track, &mut reader)?),
        _ => Err(Error::Media(MediaError::UnsupportedContentType(mime))),
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
