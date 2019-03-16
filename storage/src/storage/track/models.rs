// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use crate::api::{
    entity::StorageId,
    serde::{SerializationFormat, SerializedEntity},
};

use chrono::{naive::NaiveDateTime, Datelike};

use percent_encoding::percent_decode;

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Insertable)]
#[table_name = "tbl_track"]
pub struct InsertableTracksEntity<'a> {
    pub uid: &'a [u8],
    pub rev_no: i64,
    pub rev_ts: TickType,
    pub ser_fmt: i16,
    pub ser_vmaj: i32,
    pub ser_vmin: i32,
    pub ser_blob: &'a [u8],
}

impl<'a> InsertableTracksEntity<'a> {
    pub fn bind(
        header: &'a EntityHeader,
        ser_fmt: SerializationFormat,
        ser_blob: &'a [u8],
    ) -> Self {
        Self {
            uid: header.uid().as_ref(),
            rev_no: header.revision().ordinal() as i64,
            rev_ts: (header.revision().instant().0).0,
            ser_fmt: ser_fmt as i16,
            ser_vmaj: 0, // TODO
            ser_vmin: 0, // TODO
            ser_blob,
        }
    }
}

#[derive(Debug, AsChangeset)]
#[table_name = "tbl_track"]
pub struct UpdatableTracksEntity<'a> {
    pub rev_no: i64,
    pub rev_ts: TickType,
    pub ser_fmt: i16,
    pub ser_vmaj: i32,
    pub ser_vmin: i32,
    pub ser_blob: &'a [u8],
}

impl<'a> UpdatableTracksEntity<'a> {
    pub fn bind(
        next_revision: &'a EntityRevision,
        ser_fmt: SerializationFormat,
        ser_blob: &'a [u8],
    ) -> Self {
        Self {
            rev_no: next_revision.ordinal() as i64,
            rev_ts: (next_revision.instant().0).0,
            ser_fmt: ser_fmt as i16,
            ser_vmaj: 0, // TODO
            ser_vmin: 0, // TODO
            ser_blob,
        }
    }
}

#[derive(Debug, Queryable, Identifiable)]
#[table_name = "tbl_track"]
pub struct QueryableSerializedEntity {
    pub id: StorageId,
    pub uid: Vec<u8>,
    pub rev_no: i64,
    pub rev_ts: TickType,
    pub ser_fmt: i16,
    pub ser_vmaj: i32,
    pub ser_vmin: i32,
    pub ser_blob: Vec<u8>,
}

impl From<QueryableSerializedEntity> for SerializedEntity {
    fn from(from: QueryableSerializedEntity) -> Self {
        let uid = EntityUid::from_slice(&from.uid);
        let revision = EntityRevision::new(from.rev_no as u64, TickInstant(Ticks(from.rev_ts)));
        let header = EntityHeader::new(uid, revision);
        let format = SerializationFormat::from(from.ser_fmt).unwrap();
        debug_assert!(from.ser_vmaj >= 0);
        debug_assert!(from.ser_vmin >= 0);
        let version = EntityVersion::new(from.ser_vmaj as u32, from.ser_vmin as u32);
        SerializedEntity {
            header,
            format,
            version,
            blob: from.ser_blob,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_track_collection"]
pub struct InsertableTracksCollection<'a> {
    pub track_id: StorageId,
    pub collection_uid: &'a [u8],
    pub since: NaiveDateTime,
    pub color_code: Option<i32>,
    pub play_count: Option<i32>,
}

impl<'a> InsertableTracksCollection<'a> {
    pub fn bind(track_id: StorageId, track_collection: &'a TrackCollection) -> Self {
        Self {
            track_id,
            collection_uid: track_collection.uid.as_ref(),
            since: track_collection.since.naive_utc(),
            color_code: track_collection.color.map(|color| color.code() as i32),
            play_count: track_collection.play_count.map(|count| count as i32),
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_track_source"]
pub struct InsertableTracksSource<'a> {
    pub track_id: StorageId,
    pub uri: &'a str,
    pub uri_decoded: String,
    pub content_type: &'a str,
    pub audio_channel_count: Option<i16>,
    pub audio_duration: Option<f64>,
    pub audio_samplerate: Option<i32>,
    pub audio_bitrate: Option<i32>,
    pub audio_loudness: Option<f64>,
    pub audio_enc_name: Option<&'a str>,
    pub audio_enc_settings: Option<&'a str>,
}

impl<'a> InsertableTracksSource<'a> {
    pub fn bind(track_id: StorageId, track_source: &'a TrackSource) -> Self {
        Self {
            track_id,
            uri: track_source.uri.as_str(),
            uri_decoded: percent_decode(track_source.uri.as_bytes())
                .decode_utf8_lossy()
                .into(),
            content_type: track_source.content_type.as_str(),
            audio_channel_count: track_source
                .audio_content
                .as_ref()
                .map(|audio| audio.channel_count.0 as i16),
            audio_duration: track_source
                .audio_content
                .as_ref()
                .map(|audio| audio.duration.0),
            audio_samplerate: track_source
                .audio_content
                .as_ref()
                .map(|audio| audio.sample_rate.0 as i32),
            audio_bitrate: track_source
                .audio_content
                .as_ref()
                .map(|audio| audio.bit_rate.0 as i32),
            audio_loudness: track_source
                .audio_content
                .as_ref()
                .and_then(|audio| audio.loudness)
                .map(|loudness| loudness.0),
            audio_enc_name: track_source
                .audio_content
                .as_ref()
                .and_then(|audio| audio.encoder.as_ref())
                .map(|enc| enc.name.as_str()),
            audio_enc_settings: track_source
                .audio_content
                .as_ref()
                .and_then(|audio| audio.encoder.as_ref())
                .and_then(|enc| enc.settings.as_ref())
                .map(|settings| settings.as_str()),
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_track_brief"]
pub struct InsertableTracksBrief<'a> {
    pub track_id: StorageId,
    pub track_title: Option<&'a str>,
    pub track_artist: Option<&'a str>,
    pub track_composer: Option<&'a str>,
    pub album_title: Option<&'a str>,
    pub album_artist: Option<&'a str>,
    pub release_year: Option<i32>,
    pub track_index: Option<i32>,
    pub track_count: Option<i32>,
    pub disc_index: Option<i32>,
    pub disc_count: Option<i32>,
    pub music_tempo: Option<Beats>,
    pub music_key: Option<i16>,
}

impl<'a> InsertableTracksBrief<'a> {
    pub fn bind(track_id: StorageId, track: &'a Track) -> Self {
        Self {
            track_id,
            track_title: Titles::main_title(&track.titles).map(|title| title.name.as_str()),
            track_artist: Actors::main_actor(&track.actors, ActorRole::Artist)
                .map(|actor| actor.name.as_str()),
            track_composer: Actors::main_actor(&track.actors, ActorRole::Composer)
                .map(|actor| actor.name.as_str()),
            album_title: track
                .album
                .as_ref()
                .and_then(|album| Titles::main_title(&album.titles))
                .map(|title| title.name.as_str()),
            album_artist: track
                .album
                .as_ref()
                .and_then(|album| Actors::main_actor(&album.actors, ActorRole::Artist))
                .map(|actor| actor.name.as_str()),
            release_year: track
                .release
                .as_ref()
                .and_then(|release| release.released_at)
                .map(|released_at| released_at.date().naive_utc().year()),
            track_index: track.track_numbers.index().map(|index| index as i32),
            track_count: track.track_numbers.count().map(|count| count as i32),
            disc_index: track.disc_numbers.index().map(|index| index as i32),
            disc_count: track.disc_numbers.count().map(|count| count as i32),
            music_tempo: if track.music.tempo.is_valid() {
                Some(track.music.tempo.0)
            } else {
                None
            },
            music_key: if track.music.key.is_valid() {
                Some(i16::from(track.music.key.code()))
            } else {
                None
            },
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_tag_label"]
pub struct InsertableTagLabel<'a> {
    pub label: &'a str,
}

impl<'a> InsertableTagLabel<'a> {
    pub fn bind(label: &'a Label) -> Self {
        Self {
            label: label.as_ref(),
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_tag_facet"]
pub struct InsertableTagFacet<'a> {
    pub facet: &'a str,
}

impl<'a> InsertableTagFacet<'a> {
    pub fn bind(facet: &'a Facet) -> Self {
        Self {
            facet: facet.as_ref(),
        }
    }
}

#[derive(Debug, Clone, Copy, Insertable)]
#[table_name = "aux_track_tag"]
pub struct InsertableTracksTag {
    pub track_id: StorageId,
    pub facet_id: Option<StorageId>,
    pub label_id: StorageId,
    pub score: ScoreValue,
}

impl InsertableTracksTag {
    pub fn bind(
        track_id: StorageId,
        facet_id: Option<StorageId>,
        label_id: StorageId,
        score: Score,
    ) -> Self {
        Self {
            track_id,
            facet_id,
            label_id,
            score: score.into(),
        }
    }
}
