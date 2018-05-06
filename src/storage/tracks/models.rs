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

use uuid::Uuid;

use storage::StorageId;
use storage::serde::SerializationFormat;

use aoide_core::domain::entity::{EntityRevision, EntityHeader};
use aoide_core::domain::track::{TrackBody, TrackResource};
use aoide_core::domain::music::{Actor, ActorRole};
use aoide_core::domain::metadata::{Confidence, ConfidenceValue, Tag, Comment, Rating};

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
#[table_name = "aux_tracks_identity"]
pub struct InsertableTracksIdentity<'a> {
    pub track_id: StorageId,
    pub track_isrc: Option<&'a str>,
    pub track_acoust_id: Option<String>,
    pub track_mbrainz_id: Option<String>,
    pub track_spotify_id: Option<&'a str>,
    pub album_mbrainz_id: Option<String>,
    pub album_spotify_id: Option<&'a str>,
    pub release_ean: Option<&'a str>,
    pub release_upc: Option<&'a str>,
    pub release_asin: Option<&'a str>,
}

fn format_optional_uuid(uuid: &Uuid) -> Option<String> {
    if uuid.is_nil() {
        None
    } else {
        Some(format!("{}", uuid))
    }
}

fn format_optional_id<'a>(id: &'a str) -> Option<&'a str> {
    if id.is_empty() {
        None
    } else {
        Some(id)
    }
}

impl<'a> InsertableTracksIdentity<'a> {
    pub fn bind(track_id: StorageId, body: &'a TrackBody) -> Self {
        Self {
            track_id,
            track_isrc: body.identity.as_ref().and_then(|identity| format_optional_id(identity.isrc.as_str())),
            track_acoust_id: body.identity.as_ref().and_then(|identity| format_optional_uuid(&identity.acoust_id)),
            track_mbrainz_id: body.identity.as_ref().and_then(|identity| format_optional_uuid(&identity.mbrainz_id)),
            track_spotify_id: body.identity.as_ref().and_then(|identity| format_optional_id(identity.spotify_id.as_str())),
            album_mbrainz_id: body.album.as_ref().and_then(|album| album.identity.as_ref()).and_then(|identity| format_optional_uuid(&identity.mbrainz_id)),
            album_spotify_id: body.album.as_ref().and_then(|album| album.identity.as_ref()).and_then(|identity| format_optional_id(identity.spotify_id.as_ref())),
            release_ean: body.album.as_ref().and_then(|album| album.release.as_ref()).and_then(|release| release.identity.as_ref()).and_then(|identity| format_optional_id(identity.ean.as_str())),
            release_upc: body.album.as_ref().and_then(|album| album.release.as_ref()).and_then(|release| release.identity.as_ref()).and_then(|identity| format_optional_id(identity.upc.as_str())),
            release_asin: body.album.as_ref().and_then(|album| album.release.as_ref()).and_then(|release| release.identity.as_ref()).and_then(|identity| format_optional_id(identity.asin.as_str())),
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_tracks_overview"]
pub struct InsertableTracksOverview<'a> {
    pub track_id: StorageId,
    pub track_title: &'a str,
    pub track_subtitle: Option<&'a str>,
    pub track_number: Option<i32>,
    pub track_total: Option<i32>,
    pub disc_number: Option<i32>,
    pub disc_total: Option<i32>,
    pub album_title: Option<&'a str>,
    pub album_subtitle: Option<&'a str>,
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
            track_title: body.titles.title.as_str(),
            track_subtitle: body.titles.subtitle.as_ref().map(|subtitle| subtitle.as_str()),
            track_number: body.track_numbers.map(|numbers| numbers.this as i32),
            track_total: body.track_numbers.map(|numbers| numbers.total as i32),
            disc_number: body.disc_numbers.map(|numbers| numbers.this as i32),
            disc_total: body.disc_numbers.map(|numbers| numbers.total as i32),
            album_title: body.album.as_ref().map(|album| album.titles.title.as_str()),
            album_subtitle: body.album.as_ref().and_then(|album| album.titles.subtitle.as_ref()).map(|subtitle| subtitle.as_str()),
            album_grouping: body.album.as_ref().and_then(|album| album.grouping.as_ref()).map(|grouping| grouping.as_str()),
            album_compilation: body.album.as_ref().and_then(|album| album.compilation),
            release_date: body.album.as_ref().and_then(|album| album.release.as_ref()).and_then(|release| release.released).map(|released| released.date().naive_utc()),
            release_label: body.album.as_ref().and_then(|album| album.release.as_ref()).and_then(|release| release.label.as_ref()).map(|label| label.as_str()),
            lyrics_explicit: body.lyrics.as_ref().and_then(|lyrics| lyrics.explicit),
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "aux_tracks_summary"]
pub struct InsertableTracksSummary {
    pub track_id: StorageId,
    pub track_artists: Option<String>,
    pub track_composers: Option<String>,
    pub track_conductors: Option<String>,
    pub track_performers: Option<String>,
    pub track_producers: Option<String>,
    pub track_remixers: Option<String>,
    pub album_artists: Option<String>,
    pub album_composers: Option<String>,
    pub album_conductors: Option<String>,
    pub album_performers: Option<String>,
    pub album_producers: Option<String>,
    pub ratings_min: Option<ConfidenceValue>,
    pub ratings_max: Option<ConfidenceValue>,
}

impl InsertableTracksSummary {
    pub fn bind(track_id: StorageId, body: &TrackBody) -> Self {
        let (ratings_min, ratings_max) = match Rating::minmax(&body.ratings, None) {
            Some((Confidence(min), Confidence(max))) => (Some(min), Some(max)),
            None => (None, None),
        };
        Self {
            track_id,
            track_artists: Actor::actors_to_string(&body.actors, Some(ActorRole::Artist)),
            track_composers: Actor::actors_to_string(&body.actors, Some(ActorRole::Composer)),
            track_conductors: Actor::actors_to_string(&body.actors, Some(ActorRole::Conductor)),
            track_performers: Actor::actors_to_string(&body.actors, Some(ActorRole::Performer)),
            track_producers: Actor::actors_to_string(&body.actors, Some(ActorRole::Producer)),
            track_remixers: Actor::actors_to_string(&body.actors, Some(ActorRole::Remixer)),
            album_artists: body.album.as_ref().and_then(|album| Actor::actors_to_string(&album.actors, Some(ActorRole::Artist))),
            album_composers: body.album.as_ref().and_then(|album| Actor::actors_to_string(&album.actors, Some(ActorRole::Composer))),
            album_conductors: body.album.as_ref().and_then(|album| Actor::actors_to_string(&album.actors, Some(ActorRole::Conductor))),
            album_performers: body.album.as_ref().and_then(|album| Actor::actors_to_string(&album.actors, Some(ActorRole::Performer))),
            album_producers: body.album.as_ref().and_then(|album| Actor::actors_to_string(&album.actors, Some(ActorRole::Producer))),
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
