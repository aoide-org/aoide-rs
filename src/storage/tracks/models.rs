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

use super::schema::*;

use chrono::naive::{NaiveDate, NaiveDateTime};

use storage::StorageId;
use storage::serde::SerializationFormat;

use aoide_core::domain::entity::{EntityHeader, EntityRevision};
use aoide_core::domain::metadata::{Comment, Score, ScoreValue, Rating, Tag};
use aoide_core::domain::music::{ActorRole, ClassificationSubject};
use aoide_core::domain::music::sonic::{BeatsPerMinute, Decibel, Loudness, LUFS};
use aoide_core::domain::track::{MusicMetadata, TrackBody, TrackResource, RefOrigin};

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
    pub fn bind(
        header: &'a EntityHeader,
        ser_fmt: SerializationFormat,
        ser_blob: &'a [u8],
    ) -> Self {
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
    pub fn bind(
        next_revision: &'a EntityRevision,
        ser_fmt: SerializationFormat,
        ser_blob: &'a [u8],
    ) -> Self {
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
#[table_name = "aux_tracks_overview"]
pub struct InsertableTracksOverview<'a> {
    pub track_id: StorageId,
    pub track_title: &'a str,
    pub track_number: Option<i32>,
    pub track_total: Option<i32>,
    pub disc_number: Option<i32>,
    pub disc_total: Option<i32>,
    pub album_title: Option<&'a str>,
    pub album_grouping: Option<&'a str>,
    pub album_compilation: Option<bool>,
    pub release_date: Option<NaiveDate>,
    pub release_label: Option<&'a str>,
    pub lyrics_explicit: Option<bool>,
}

impl<'a> InsertableTracksOverview<'a> {
    pub fn bind(track_id: StorageId, body: &'a TrackBody) -> Self {
        Self {
            track_id,
            track_title: body.main_title().map(|title| title.name.as_str()).unwrap_or(""),
            track_number: body.track_numbers.this.map(|this| this as i32),
            track_total: body.track_numbers.total.map(|total| total as i32),
            disc_number: body.disc_numbers.this.map(|this| this as i32),
            disc_total: body.disc_numbers.total.map(|total| total as i32),
            album_title: body.album_main_title().map(|title| title.name.as_str()),
            album_grouping: body.album
                .as_ref()
                .and_then(|album| album.grouping.as_ref())
                .map(|grouping| grouping.as_str()),
            album_compilation: body.album.as_ref().and_then(|album| album.compilation),
            release_date: body.album
                .as_ref()
                .and_then(|album| album.release.as_ref())
                .and_then(|release| release.released)
                .map(|released| released.date().naive_utc()),
            release_label: body.album
                .as_ref()
                .and_then(|album| album.release.as_ref())
                .and_then(|release| release.label.as_ref())
                .map(|label| label.as_str()),
            lyrics_explicit: body.lyrics.as_ref().and_then(|lyrics| lyrics.explicit),
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_tracks_summary"]
pub struct InsertableTracksSummary<'a> {
    pub track_id: StorageId,
    pub track_artist: Option<&'a str>,
    pub track_composer: Option<&'a str>,
    pub track_conductor: Option<&'a str>,
    pub track_performer: Option<&'a str>,
    pub track_producer: Option<&'a str>,
    pub track_remixer: Option<&'a str>,
    pub album_artist: Option<&'a str>,
    pub album_composer: Option<&'a str>,
    pub album_conductor: Option<&'a str>,
    pub album_performer: Option<&'a str>,
    pub album_producer: Option<&'a str>,
    pub ratings_min: Option<ScoreValue>,
    pub ratings_max: Option<ScoreValue>,
}

impl<'a> InsertableTracksSummary<'a> {
    pub fn bind(track_id: StorageId, body: &'a TrackBody) -> Self {
        let (ratings_min, ratings_max) = match Rating::minmax(&body.ratings, None) {
            Some((Score(min), Score(max))) => (Some(min), Some(max)),
            None => (None, None),
        };
        Self {
            track_id,
            track_artist: TrackBody::main_actor(&body, ActorRole::Artist).map(|actor| actor.name.as_str()),
            track_composer: TrackBody::main_actor(&body, ActorRole::Composer).map(|actor| actor.name.as_str()),
            track_conductor: TrackBody::main_actor(&body, ActorRole::Conductor).map(|actor| actor.name.as_str()),
            track_performer: TrackBody::main_actor(&body, ActorRole::Performer).map(|actor| actor.name.as_str()),
            track_producer: TrackBody::main_actor(&body, ActorRole::Producer).map(|actor| actor.name.as_str()),
            track_remixer: TrackBody::main_actor(&body, ActorRole::Remixer).map(|actor| actor.name.as_str()),
            album_artist: TrackBody::album_main_actor(&body, ActorRole::Artist).map(|actor| actor.name.as_str()),
            album_composer: TrackBody::album_main_actor(&body, ActorRole::Composer).map(|actor| actor.name.as_str()),
            album_conductor: TrackBody::album_main_actor(&body, ActorRole::Conductor).map(|actor| actor.name.as_str()),
            album_performer: TrackBody::album_main_actor(&body, ActorRole::Performer).map(|actor| actor.name.as_str()),
            album_producer: TrackBody::album_main_actor(&body, ActorRole::Producer).map(|actor| actor.name.as_str()),
            ratings_min,
            ratings_max,
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
    pub audio_duration_ms: Option<f64>,
    pub audio_channels: Option<i16>,
    pub audio_samplerate_hz: Option<i32>,
    pub audio_bitrate_bps: Option<i32>,
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
            source_sync_when: track_resource
                .source
                .synchronization
                .map(|sync| sync.when.naive_utc()),
            source_sync_rev_ordinal: track_resource
                .source
                .synchronization
                .map(|sync| sync.revision.ordinal() as i64),
            source_sync_rev_timestamp: track_resource
                .source
                .synchronization
                .map(|sync| sync.revision.timestamp().naive_utc()),
            content_type: track_resource.source.content_type.as_str(),
            audio_duration_ms: track_resource
                .source
                .audio_content
                .as_ref()
                .map(|audio| audio.duration.millis),
            audio_channels: track_resource
                .source
                .audio_content
                .as_ref()
                .map(|audio| audio.channels.count as i16),
            audio_samplerate_hz: track_resource
                .source
                .audio_content
                .as_ref()
                .map(|audio| audio.samplerate.hz as i32),
            audio_bitrate_bps: track_resource
                .source
                .audio_content
                .as_ref()
                .map(|audio| audio.bitrate.bps as i32),
            audio_enc_name: track_resource
                .source
                .audio_content
                .as_ref()
                .and_then(|audio| audio.encoder.as_ref())
                .map(|enc| enc.name.as_str()),
            audio_enc_settings: track_resource
                .source
                .audio_content
                .as_ref()
                .and_then(|audio| audio.encoder.as_ref())
                .and_then(|enc| enc.settings.as_ref())
                .map(|settings| settings.as_str()),
            color_code: track_resource.color.map(|color| color.code as i32),
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_tracks_music"]
pub struct InsertableTracksMusic {
    pub track_id: StorageId,
    pub loudness_db: Decibel,
    pub tempo_bpm: BeatsPerMinute,
    pub time_sig_num: i16,
    pub time_sig_denom: i16,
    pub key_sig_code: i16,
    pub acousticness_score: Option<ScoreValue>,
    pub danceability_score: Option<ScoreValue>,
    pub energy_score: Option<ScoreValue>,
    pub instrumentalness_score: Option<ScoreValue>,
    pub liveness_score: Option<ScoreValue>,
    pub popularity_score: Option<ScoreValue>,
    pub speechiness_score: Option<ScoreValue>,
    pub valence_score: Option<ScoreValue>,
}

impl InsertableTracksMusic {
    pub fn bind(track_id: StorageId, music: &MusicMetadata) -> Self {
        let loudness_db = match music.loudness {
            Some(Loudness::EBUR128LUFS(LUFS { db })) => db,
            None => 0 as Decibel,
        };
        Self {
            track_id,
            loudness_db: loudness_db,
            tempo_bpm: music.tempo.bpm,
            time_sig_num: music.time_signature.numerator as i16,
            time_sig_denom: music.time_signature.denominator as i16,
            key_sig_code: music.key_signature.code as i16,
            acousticness_score: music
                .classification(ClassificationSubject::Acousticness)
                .map(|classification| *classification.score),
            danceability_score: music
                .classification(ClassificationSubject::Danceability)
                .map(|classification| *classification.score),
            energy_score: music
                .classification(ClassificationSubject::Energy)
                .map(|classification| *classification.score),
            instrumentalness_score: music
                .classification(ClassificationSubject::Instrumentalness)
                .map(|classification| *classification.score),
            liveness_score: music
                .classification(ClassificationSubject::Liveness)
                .map(|classification| *classification.score),
            popularity_score: music
                .classification(ClassificationSubject::Popularity)
                .map(|classification| *classification.score),
            speechiness_score: music
                .classification(ClassificationSubject::Speechiness)
                .map(|classification| *classification.score),
            valence_score: music
                .classification(ClassificationSubject::Valence)
                .map(|classification| *classification.score),
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_tracks_ref"]
pub struct InsertableTracksRef<'a> {
    pub track_id: StorageId,
    pub origin: i16,
    pub reference: &'a str,
}

impl<'a> InsertableTracksRef<'a> {
    pub fn bind(track_id: StorageId, origin: RefOrigin, reference: &'a str) -> Self {
        Self {
            track_id,
            origin: origin as i16,
            reference,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_tracks_tag"]
pub struct InsertableTracksTag<'a> {
    pub track_id: StorageId,
    pub facet: Option<&'a str>,
    pub term: &'a str,
    pub score: ScoreValue,
}

impl<'a> InsertableTracksTag<'a> {
    pub fn bind(track_id: StorageId, tag: &'a Tag) -> Self {
        Self {
            track_id,
            facet: tag.facet.as_ref().and_then(|facet|
                // Empty strings become NULL in database
                if facet.is_empty() {
                    None
                } else {
                    Some(facet.as_str())
                }),
            term: tag.term.as_str(),
            score: *tag.score,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_tracks_comment"]
pub struct InsertableTracksComment<'a> {
    pub track_id: StorageId,
    pub owner: Option<&'a str>,
    pub text: &'a str,
}

impl<'a> InsertableTracksComment<'a> {
    pub fn bind(track_id: StorageId, comment: &'a Comment) -> Self {
        Self {
            track_id,
            owner: comment.owner.as_ref().map(|owner| owner.as_str()),
            text: comment.text.as_str(),
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_tracks_rating"]
pub struct InsertableTracksRating<'a> {
    pub track_id: StorageId,
    pub owner: Option<&'a str>,
    pub score: ScoreValue,
}

impl<'a> InsertableTracksRating<'a> {
    pub fn bind(track_id: StorageId, rating: &'a Rating) -> Self {
        Self {
            track_id,
            owner: rating.owner.as_ref().map(|owner| owner.as_str()),
            score: *rating.score,
        }
    }
}
