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

use super::schema::{tracks_entity, aux_tracks_resource, aux_tracks_tag, aux_tracks_comment, aux_tracks_rating};

use chrono::naive::NaiveDateTime;

use storage::StorageId;
use storage::serde::SerializationFormat;

use aoide_core::domain::entity::{EntityRevision, EntityHeader};
use aoide_core::domain::track::TrackResource;
use aoide_core::domain::metadata::{ConfidenceValue, Tag, Comment, Rating};

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
#[table_name = "aux_tracks_resource"]
pub struct InsertableTracksResource<'a> {
    pub track_id: StorageId,
    pub collection_uid: &'a str,
    pub collection_since: NaiveDateTime,
    pub source_uri: &'a str,
    pub source_sync_when: Option<NaiveDateTime>,
    pub source_sync_rev_ordinal: Option<i64>,
    pub source_sync_rev_timestamp: Option<NaiveDateTime>,
    pub content_type: &'a str,
    pub audio_duration: Option<i64>,
    pub audio_channels: Option<i16>,
    pub audio_samplerate: Option<i32>,
    pub audio_bitrate: Option<i32>,
    pub audio_enc_name: Option<&'a str>,
    pub audio_enc_settings: Option<&'a str>,
    pub color_code: Option<i32>,
}

impl<'a> InsertableTracksResource<'a> {
    pub fn bind(track_id: StorageId, track_resource: &'a TrackResource) -> Self {
        Self {
            track_id,
            collection_uid: track_resource.collection.uid.as_str(),
            collection_since: track_resource.collection.since.naive_utc(),
            source_uri: track_resource.source.uri.as_str(),
            source_sync_when: track_resource.source.synchronization.map(|sync| sync.when.naive_utc()),
            source_sync_rev_ordinal: track_resource.source.synchronization.map(|sync| sync.revision.ordinal() as i64),
            source_sync_rev_timestamp: track_resource.source.synchronization.map(|sync| sync.revision.timestamp().naive_utc()),
            content_type: track_resource.source.content_type.as_str(),
            audio_duration: track_resource.source.audio_content.as_ref().map(|audio| audio.duration.millis as i64),
            audio_channels: track_resource.source.audio_content.as_ref().map(|audio| audio.channels.count as i16),
            audio_samplerate: track_resource.source.audio_content.as_ref().map(|audio| audio.samplerate.hz as i32),
            audio_bitrate: track_resource.source.audio_content.as_ref().map(|audio| audio.bitrate.bps as i32),
            audio_enc_name: track_resource.source.audio_content.as_ref().and_then(|audio| audio.encoder.as_ref()).map(|enc| enc.name.as_str()),
            audio_enc_settings: track_resource.source.audio_content.as_ref().and_then(|audio| audio.encoder.as_ref()).and_then(|enc| enc.settings.as_ref()).map(|settings| settings.as_str()),
            color_code: track_resource.color.map(|color| color.code as i32),
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_tracks_tag"]
pub struct InsertableTracksTag<'a> {
    pub track_id: StorageId,
    pub facet: Option<&'a str>,
    pub term: &'a str,
    pub confidence: ConfidenceValue,
}

impl<'a> InsertableTracksTag<'a> {
    pub fn bind(track_id: StorageId, tag: &'a Tag) -> Self {
        Self {
            track_id,
            facet: tag.facet.as_ref().map(|facet| facet.as_str()),
            term: tag.term.as_str(),
            confidence: *tag.confidence,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_tracks_comment"]
pub struct InsertableTracksComment<'a> {
    pub track_id: StorageId,
    pub owner: Option<&'a str>,
    pub comment: &'a str,
}

impl<'a> InsertableTracksComment<'a> {
    pub fn bind(track_id: StorageId, comment: &'a Comment) -> Self {
        Self {
            track_id,
            owner: comment.owner.as_ref().map(|owner| owner.as_str()),
            comment: comment.comment.as_str(),
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_tracks_rating"]
pub struct InsertableTracksRating<'a> {
    pub track_id: StorageId,
    pub owner: Option<&'a str>,
    pub rating: ConfidenceValue,
}

impl<'a> InsertableTracksRating<'a> {
    pub fn bind(track_id: StorageId, rating: &'a Rating) -> Self {
        Self {
            track_id,
            owner: rating.owner.as_ref().map(|owner| owner.as_str()),
            rating: *rating.rating,
        }
    }
}
