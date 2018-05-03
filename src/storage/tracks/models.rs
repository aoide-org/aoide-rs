// Aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom>
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

use super::schema::{tracks_entity, tracks_media, tracks_media_collection};

use chrono::naive::NaiveDateTime;

use aoide_core::domain::entity::{EntityUid, EntityRevision, EntityHeader};
use aoide_core::domain::track::{MediaResource};

use storage::{StorageId, SerializationFormat};

pub type TracksEntityIdColumn = (
    tracks_entity::id,
);

pub const TRACKS_ENTITY_ID_COLUMN: TracksEntityIdColumn = (
    tracks_entity::id,
);

#[derive(Debug, Insertable)]
#[table_name = "tracks_entity"]
pub struct InsertableTracksEntity<'a> {
    pub uid: &'a str,
    pub rev_ordinal: i64,
    pub rev_timestamp: NaiveDateTime,
    pub ser_fmt: i16,
    pub ser_ver_major: i32,
    pub ser_ver_minor: i32,
    pub ser_blob: &'a [u8],
}

impl<'a> InsertableTracksEntity<'a> {
    pub fn bind(header: &'a EntityHeader, ser_fmt: SerializationFormat, ser_blob: &'a [u8]) -> Self {
        Self {
            uid: header.uid().as_str(),
            rev_ordinal: header.revision().ordinal() as i64,
            rev_timestamp: header.revision().timestamp().naive_utc(),
            ser_fmt: ser_fmt as i16,
            ser_ver_major: 0, // TODO
            ser_ver_minor: 0, // TODO
            ser_blob,
        }
    }
}

#[derive(Debug, AsChangeset)]
#[table_name = "tracks_entity"]
pub struct UpdatableTracksEntity<'a> {
    pub rev_ordinal: i64,
    pub rev_timestamp: NaiveDateTime,
    pub ser_fmt: i16,
    pub ser_ver_major: i32,
    pub ser_ver_minor: i32,
    pub ser_blob: &'a [u8],
}

impl<'a> UpdatableTracksEntity<'a> {
    pub fn bind(next_revision: &'a EntityRevision, ser_fmt: SerializationFormat, ser_blob: &'a [u8]) -> Self {
        Self {
            rev_ordinal: next_revision.ordinal() as i64,
            rev_timestamp: next_revision.timestamp().naive_utc(),
            ser_fmt: ser_fmt as i16,
            ser_ver_major: 0, // TODO
            ser_ver_minor: 0, // TODO
            ser_blob,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "tracks_media"]
pub struct InsertableTracksMedia<'a> {
    pub track_id: StorageId,
    pub uri: &'a str,
    pub content_type: &'a str,
    pub sync_rev_ordinal: Option<i64>,
    pub sync_rev_timestamp: Option<NaiveDateTime>,
    pub audio_duration: Option<i64>,
    pub audio_channels: Option<i16>,
    pub audio_samplerate: Option<i32>,
    pub audio_bitrate: Option<i32>,
    pub audio_enc_name: Option<&'a str>,
    pub audio_enc_settings: Option<&'a str>,
}

impl<'a> InsertableTracksMedia<'a> {
    pub fn bind(track_id: StorageId, resource: &'a MediaResource) -> Self {
        Self {
            track_id,
            uri: resource.uri.as_str(),
            content_type: resource.content_type.as_str(),
            sync_rev_ordinal: resource.synchronized_revision.map(|rev| rev.ordinal() as i64),
            sync_rev_timestamp: resource.synchronized_revision.map(|rev| rev.timestamp().naive_utc()),
            audio_duration: resource.audio_content.as_ref().map(|audio| audio.duration.millis as i64),
            audio_channels: resource.audio_content.as_ref().map(|audio| audio.channels.count as i16),
            audio_samplerate: resource.audio_content.as_ref().map(|audio| audio.samplerate.hz as i32),
            audio_bitrate: resource.audio_content.as_ref().map(|audio| audio.bitrate.bps as i32),
            audio_enc_name: resource.audio_content.as_ref().and_then(|audio| audio.encoder.as_ref()).map(|enc| enc.name.as_str()),
            audio_enc_settings: resource.audio_content.as_ref().and_then(|audio| audio.encoder.as_ref()).and_then(|enc| enc.settings.as_ref()).map(|settings| settings.as_str()),
        }
    }
}

/*
#[derive(Debug, Queryable)]
pub struct QueryableTracksMedia<'a> {
    pub id: StorageId,
    pub track_id: StorageId,
    pub uid: String,
    pub rev_ordinal: i64,
    pub rev_timestamp: NaiveDateTime,
    pub ser_fmt: i16,
    pub ser_ver_major: i32,
    pub ser_ver_minor: i32,
    pub ser_blob: &'a [u8],
}
*/

#[derive(Debug, Insertable)]
#[table_name = "tracks_media_collection"]
pub struct InsertableTracksMediaCollection<'a> {
    pub media_id: StorageId,
    pub collection_uid: &'a str,
}

impl<'a> InsertableTracksMediaCollection<'a> {
    pub fn bind(media_id: StorageId, collection_uid: &'a EntityUid) -> Self {
        Self {
            media_id,
            collection_uid: collection_uid.as_str(),
        }
    }
}

/*
#[derive(Debug, AsChangeset)]
#[table_name = "tracks_entity"]
pub struct UpdatableTrackEntity<'a> {
    pub rev_ordinal: i64,
    pub rev_timestamp: NaiveDateTime,
    pub name: &'a str,
    pub description: Option<&'a str>,
}

impl<'a> UpdatableTrackEntity<'a> {
    pub fn bind(next_revision: &EntityRevision, body: &'a TrackBody) -> Self {
        Self {
            rev_ordinal: next_revision.ordinal() as i64,
            rev_timestamp: next_revision.timestamp().naive_utc(),
            name: &body.name,
            description: body.description.as_ref().map(|s| s.as_str()),
        }
    }
}
*/
