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

#![warn(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))]
#![deny(rust_2018_idioms)]
#![deny(rust_2021_compatibility)]
#![deny(missing_debug_implementations)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(clippy::all)]
#![deny(clippy::explicit_deref_methods)]
#![deny(clippy::explicit_into_iter_loop)]
#![deny(clippy::explicit_iter_loop)]
#![deny(clippy::must_use_candidate)]
#![cfg_attr(not(test), deny(clippy::panic_in_result_fn))]
#![cfg_attr(not(debug_assertions), deny(clippy::used_underscore_binding))]

use chrono::{NaiveDateTime, Utc};
use tantivy::{
    schema::{Field, Schema, INDEXED, STORED, STRING, TEXT},
    Document,
};

use aoide_core::{
    media::content::ContentMetadata,
    tag::{FacetedTags, PlainTag},
    track::{
        self,
        tag::{FACET_COMMENT, FACET_GENRE, FACET_MOOD},
    },
    util::clock::{DateTime, DateYYYYMMDD},
};

const UID: &str = "uid";
const CONTENT_PATH: &str = "content-path";
const CONTENT_TYPE: &str = "content-type";
const COLLECTED_AT: &str = "collected-at";
const DURATION_MS: &str = "duration-ms";
const TRACK_TITLE: &str = "track-title";
const TRACK_ARTIST: &str = "track-artist";
const ALBUM_TITLE: &str = "album-title";
const ALBUM_ARTIST: &str = "album-artist";
const GENRE: &str = "genre";
const MOOD: &str = "mood";
const COMMENT: &str = "comment";
// const PLAIN_TAG: &str = "tag";
// const FACETED_TAG: &str = "facet";
const RECORDED_AT_YYYYMMDD: &str = "recorded-at-yyyymmdd";
const RELEASED_AT_YYYYMMDD: &str = "released-at-yyyymmdd";
// const RATING: &str = "rating";
// const ACOUSTICNESS: &str = "acousticness";
// const AROUSAL: &str = "arousal";
// const DANCEABILITY: &str = "danceability";
// const ENERGY: &str = "energy";
// const INSTRUMENTALNESS: &str = "instrumentalness";
// const LIVENESS: &str = "liveness";
// const POPULARITY: &str = "popularity";
// const SPEECHINESS: &str = "speechiness";
// const VALENCE: &str = "valence";
const TIMES_PLAYED: &str = "times-played";
const LAST_PLAYED_AT: &str = "last-played-at";

#[derive(Debug, Clone)]
pub struct TrackFields {
    pub uid: Field,
    pub content_path: Field,
    pub content_type: Field,
    pub collected_at: Field,
    pub duration_ms: Field,
    pub track_title: Field,
    pub track_artist: Field,
    pub album_title: Field,
    pub album_artist: Field,
    pub genre: Field,
    pub mood: Field,
    pub comment: Field,
    pub recorded_at_yyyymmdd: Field,
    pub released_at_yyyymmdd: Field,
    pub times_played: Field,
    pub last_played_at: Field,
}

fn tantivy_date_time(input: DateTime) -> tantivy::DateTime {
    let nanos = input.unix_timestamp_nanos();
    let secs = (nanos / 1_000_000_000) as i64;
    let nsecs = (nanos % 1_000_000_000) as u32;
    tantivy::DateTime::from_utc(NaiveDateTime::from_timestamp(secs, nsecs), Utc)
}

impl TrackFields {
    #[must_use]
    pub fn create_document(&self, entity: &track::Entity) -> Document {
        let mut doc = Document::new();
        doc.add_bytes(self.uid, entity.hdr.uid.as_ref());
        doc.add_text(
            self.content_path,
            &entity.body.track.media_source.content_link.path,
        );
        doc.add_date(
            self.collected_at,
            &tantivy_date_time(entity.body.track.media_source.collected_at),
        );
        let ContentMetadata::Audio(audio_metadata) =
            &entity.body.track.media_source.content_metadata;
        if let Some(duration) = audio_metadata.duration {
            doc.add_f64(self.duration_ms, duration.to_inner());
        }
        if let Some(track_title) = entity.body.track.track_title() {
            doc.add_text(self.track_title, track_title);
        }
        if let Some(track_artist) = entity.body.track.track_artist() {
            doc.add_text(self.track_artist, track_artist);
        }
        if let Some(album_title) = entity.body.track.album_title() {
            doc.add_text(self.album_title, album_title);
        }
        if let Some(album_artist) = entity.body.track.album_artist() {
            doc.add_text(self.album_artist, album_artist);
        }
        if let Some(recorded_at_yyyymmdd) = entity.body.track.recorded_at.map(DateYYYYMMDD::from) {
            doc.add_i64(self.album_artist, recorded_at_yyyymmdd.to_inner().into());
        }
        if let Some(released_at_yyyymmdd) = entity.body.track.released_at.map(DateYYYYMMDD::from) {
            doc.add_i64(self.album_artist, released_at_yyyymmdd.to_inner().into());
        }
        if let Some(times_played) = entity.body.track.play_counter.times_played {
            doc.add_u64(self.times_played, times_played);
        }
        if let Some(last_played_at) = entity.body.track.play_counter.last_played_at {
            doc.add_date(self.times_played, &tantivy_date_time(last_played_at));
        }
        for faceted_tags in &entity.body.track.tags.facets {
            let FacetedTags { facet_id, tags } = faceted_tags;
            let field = match facet_id.as_str() {
                FACET_GENRE => self.genre,
                FACET_MOOD => self.mood,
                FACET_COMMENT => self.comment,
                _ => continue,
            };
            for tag in tags {
                let PlainTag {
                    label,
                    score: _, // TODO: How to take the score into account?
                } = tag;
                if let Some(label) = &label {
                    doc.add_text(field, label)
                }
            }
        }
        doc
    }
}

#[must_use]
pub fn build_schema_for_tracks() -> (Schema, TrackFields) {
    let mut schema_builder = Schema::builder();
    let uid = schema_builder.add_bytes_field(UID, STORED);
    let content_path = schema_builder.add_text_field(CONTENT_PATH, STRING | STORED);
    let content_type = schema_builder.add_text_field(CONTENT_TYPE, STRING);
    let collected_at = schema_builder.add_date_field(COLLECTED_AT, INDEXED);
    let duration_ms = schema_builder.add_f64_field(DURATION_MS, INDEXED);
    let track_title = schema_builder.add_text_field(TRACK_TITLE, TEXT);
    let track_artist = schema_builder.add_text_field(TRACK_ARTIST, TEXT);
    let album_title = schema_builder.add_text_field(ALBUM_TITLE, TEXT);
    let album_artist = schema_builder.add_text_field(ALBUM_ARTIST, TEXT);
    let genre = schema_builder.add_text_field(GENRE, TEXT);
    let mood = schema_builder.add_text_field(MOOD, TEXT);
    let comment = schema_builder.add_text_field(COMMENT, TEXT);
    // schema_builder.add_text_field(PLAIN_TAG, TEXT);
    // schema_builder.add_facet_field(FACETED_TAG, INDEXED);
    let recorded_at_yyyymmdd = schema_builder.add_i64_field(RECORDED_AT_YYYYMMDD, INDEXED);
    let released_at_yyyymmdd = schema_builder.add_i64_field(RELEASED_AT_YYYYMMDD, INDEXED);
    // schema_builder.add_f64_field(RATING, INDEXED);
    // schema_builder.add_f64_field(ACOUSTICNESS, INDEXED);
    // schema_builder.add_f64_field(AROUSAL, INDEXED);
    // schema_builder.add_f64_field(DANCEABILITY, INDEXED);
    // schema_builder.add_f64_field(ENERGY, INDEXED);
    // schema_builder.add_f64_field(INSTRUMENTALNESS, INDEXED);
    // schema_builder.add_f64_field(LIVENESS, INDEXED);
    // schema_builder.add_f64_field(POPULARITY, INDEXED);
    // schema_builder.add_f64_field(SPEECHINESS, INDEXED);
    // schema_builder.add_f64_field(VALENCE, INDEXED);
    let times_played = schema_builder.add_u64_field(TIMES_PLAYED, INDEXED);
    let last_played_at = schema_builder.add_date_field(LAST_PLAYED_AT, INDEXED);
    let schema = schema_builder.build();
    let fields = TrackFields {
        uid,
        content_path,
        content_type,
        collected_at,
        duration_ms,
        track_artist,
        track_title,
        album_artist,
        album_title,
        genre,
        mood,
        comment,
        recorded_at_yyyymmdd,
        released_at_yyyymmdd,
        times_played,
        last_played_at,
    };
    (schema, fields)
}
