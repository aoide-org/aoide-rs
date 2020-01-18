// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use aoide_core::{
    actor::*,
    media,
    music::time::Beats,
    tag::*,
    title::*,
    track::{
        self,
        marker::{beat, key},
        release::YYYYMMDD,
        *,
    },
    util::clock::*,
};

use aoide_repo::{entity::*, RepoId};

use chrono::{naive::NaiveDateTime, DateTime};

use percent_encoding::percent_decode;

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Insertable)]
#[table_name = "tbl_track"]
pub struct InsertableEntity<'a> {
    pub uid: &'a [u8],
    pub rev_no: i64,
    pub rev_ts: TickType,
    pub data_fmt: i16,
    pub data_vmaj: i16,
    pub data_vmin: i16,
    pub data_blob: &'a [u8],
}

impl<'a> InsertableEntity<'a> {
    pub fn bind(
        hdr: &'a EntityHeader,
        data_fmt: EntityDataFormat,
        data_ver: EntityDataVersion,
        data_blob: &'a [u8],
    ) -> Self {
        Self {
            uid: hdr.uid.as_ref(),
            rev_no: hdr.rev.no as i64,
            rev_ts: (hdr.rev.ts.0).0,
            data_fmt: data_fmt as i16,
            data_vmaj: data_ver.major as i16,
            data_vmin: data_ver.minor as i16,
            data_blob,
        }
    }
}

#[derive(Debug, AsChangeset)]
#[table_name = "tbl_track"]
pub struct UpdatableEntity<'a> {
    pub rev_no: i64,
    pub rev_ts: TickType,
    pub data_fmt: i16,
    pub data_vmaj: i16,
    pub data_vmin: i16,
    pub data_blob: &'a [u8],
}

impl<'a> UpdatableEntity<'a> {
    pub fn bind(
        next_rev: &'a EntityRevision,
        data_fmt: EntityDataFormat,
        data_ver: EntityDataVersion,
        data_blob: &'a [u8],
    ) -> Self {
        Self {
            rev_no: next_rev.no as i64,
            rev_ts: (next_rev.ts.0).0,
            data_fmt: data_fmt as i16,
            data_vmaj: data_ver.major as i16,
            data_vmin: data_ver.minor as i16,
            data_blob,
        }
    }
}

#[derive(Debug, Queryable, Identifiable)]
#[table_name = "tbl_track"]
pub struct QueryableEntityData {
    pub id: RepoId,
    pub uid: Vec<u8>,
    pub rev_no: i64,
    pub rev_ts: TickType,
    pub data_fmt: i16,
    pub data_vmaj: i16,
    pub data_vmin: i16,
    pub data_blob: Vec<u8>,
}

impl From<QueryableEntityData> for EntityData {
    fn from(from: QueryableEntityData) -> Self {
        let rev = EntityRevision {
            no: from.rev_no as u64,
            ts: TickInstant(Ticks(from.rev_ts)),
        };
        let hdr = EntityHeader {
            uid: EntityUid::from_slice(&from.uid),
            rev,
        };
        let fmt = if from.data_fmt == EntityDataFormat::JSON as i16 {
            EntityDataFormat::JSON
        } else {
            // TODO: How to handle unexpected/invalid values?
            unreachable!()
        };
        let ver = EntityDataVersion {
            major: from.data_vmaj as EntityDataVersionNumber,
            minor: from.data_vmin as EntityDataVersionNumber,
        };
        (hdr, (fmt, ver, from.data_blob))
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_track_collection"]
pub struct InsertableCollection<'a> {
    pub track_id: RepoId,
    pub collection_uid: &'a [u8],
    pub since: NaiveDateTime,
    pub comment: Option<&'a str>,
    pub color_code: Option<i32>,
    pub play_count: Option<i32>,
}

impl<'a> InsertableCollection<'a> {
    pub fn bind(track_id: RepoId, collection: &'a track::collection::Collection) -> Self {
        Self {
            track_id,
            collection_uid: collection.uid.as_ref(),
            since: DateTime::from(collection.since).naive_utc(),
            comment: collection.comment.as_ref().map(String::as_str),
            color_code: collection.color.map(|color| color.code() as i32),
            play_count: collection.play_count.map(|count| count as i32),
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_track_media"]
pub struct InsertableSource<'a> {
    pub track_id: RepoId,
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

impl<'a> InsertableSource<'a> {
    pub fn bind(track_id: RepoId, media_source: &'a media::Source) -> Self {
        let audio_content = {
            match media_source.content {
                media::Content::Audio(ref audio_content) => Some(audio_content),
            }
        };
        Self {
            track_id,
            uri: media_source.uri.as_str(),
            uri_decoded: percent_decode(media_source.uri.as_bytes())
                .decode_utf8_lossy()
                .into(),
            content_type: media_source.content_type.as_str(),
            audio_channel_count: audio_content.map(|audio| audio.channels.count().0 as i16),
            audio_duration: audio_content.as_ref().map(|audio| audio.duration.0),
            audio_samplerate: audio_content.map(|audio| audio.sample_rate.0 as i32),
            audio_bitrate: audio_content.map(|audio| audio.bit_rate.0 as i32),
            audio_loudness: audio_content
                .and_then(|audio| audio.loudness)
                .map(|loudness| loudness.0),
            audio_enc_name: audio_content
                .and_then(|audio| audio.encoder.as_ref())
                .map(|enc| enc.name.as_str()),
            audio_enc_settings: audio_content
                .and_then(|audio| audio.encoder.as_ref())
                .and_then(|enc| enc.settings.as_ref())
                .map(String::as_str),
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_track_location"]
pub struct InsertableLocation<'a> {
    pub track_id: RepoId,
    pub collection_uid: &'a [u8],
    pub uri: &'a str,
}

impl<'a> InsertableLocation<'a> {
    pub fn bind(track_id: RepoId, collection_uid: &'a [u8], uri: &'a str) -> Self {
        Self {
            track_id,
            collection_uid,
            uri,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_track_brief"]
pub struct InsertableBrief<'a> {
    pub track_id: RepoId,
    pub track_title: Option<&'a str>,
    pub track_artist: Option<&'a str>,
    pub track_composer: Option<&'a str>,
    pub album_title: Option<&'a str>,
    pub album_artist: Option<&'a str>,
    pub release_date: Option<YYYYMMDD>,
    pub track_number: Option<i16>,
    pub track_total: Option<i16>,
    pub disc_number: Option<i16>,
    pub disc_total: Option<i16>,
    pub music_tempo: Option<Beats>,
    pub music_key: Option<i16>,
}

impl<'a> InsertableBrief<'a> {
    pub fn bind(track_id: RepoId, track: &'a Track) -> Self {
        Self {
            track_id,
            track_title: Titles::main_title(&track.titles).map(|title| title.name.as_str()),
            track_artist: Actors::main_actor(track.actors.iter(), ActorRole::Artist)
                .map(|actor| actor.name.as_str()),
            track_composer: Actors::main_actor(track.actors.iter(), ActorRole::Composer)
                .map(|actor| actor.name.as_str()),
            album_title: track
                .album
                .as_ref()
                .and_then(|album| Titles::main_title(album.titles.iter()))
                .map(|title| title.name.as_str()),
            album_artist: track
                .album
                .as_ref()
                .and_then(|album| Actors::main_actor(album.actors.iter(), ActorRole::Artist))
                .map(|actor| actor.name.as_str()),
            release_date: track
                .release
                .as_ref()
                .and_then(|release| release.date())
                .map(Into::into),
            track_number: track.indexes.track.number().map(|idx| idx as i16),
            track_total: track.indexes.track.total().map(|cnt| cnt as i16),
            disc_number: track.indexes.disc.number().map(|idx| idx as i16),
            disc_total: track.indexes.disc.total().map(|cnt| cnt as i16),
            music_tempo: beat::Markers::uniform_tempo(&track.markers.beats).map(|tempo| tempo.0),
            music_key: key::Markers::uniform_key(&track.markers.keys)
                .map(|key| i16::from(key.code())),
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

#[derive(Debug, Copy, Clone, Insertable)]
#[table_name = "aux_track_tag"]
pub struct InsertableTracksTag {
    pub track_id: RepoId,
    pub facet_id: Option<RepoId>,
    pub label_id: Option<RepoId>,
    pub score: ScoreValue,
}

impl InsertableTracksTag {
    pub fn bind(
        track_id: RepoId,
        facet_id: Option<RepoId>,
        label_id: Option<RepoId>,
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

#[derive(Debug, Insertable)]
#[table_name = "aux_marker_label"]
pub struct InsertableMarkerLabel<'a> {
    pub label: &'a str,
}

impl<'a> InsertableMarkerLabel<'a> {
    pub fn bind(label: &'a str) -> Self {
        Self { label }
    }
}

#[derive(Debug, Copy, Clone, Insertable)]
#[table_name = "aux_track_marker"]
pub struct InsertableTracksMarker {
    pub track_id: RepoId,
    pub label_id: RepoId,
}

impl InsertableTracksMarker {
    pub fn bind(track_id: RepoId, label_id: RepoId) -> Self {
        Self { track_id, label_id }
    }
}
