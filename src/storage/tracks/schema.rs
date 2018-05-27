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

use super::collections::schema::collections_entity;

table! {
    tracks_entity (id) {
        id -> BigInt,
        uid -> Text,
        rev_ordinal -> BigInt,
        rev_timestamp -> Timestamp,
        ser_fmt -> SmallInt,
        ser_ver_major -> Integer,
        ser_ver_minor -> Integer,
        ser_blob -> Binary,
    }
}

table! {
    aux_tracks_resource (id) {
        id -> BigInt,
        track_id -> BigInt,
        collection_uid -> Text,
        collection_since -> Timestamp,
        source_uri -> Text,
        source_sync_when -> Nullable<Timestamp>,
        source_sync_rev_ordinal -> Nullable<BigInt>,
        source_sync_rev_timestamp -> Nullable<Timestamp>,
        content_type -> Text,
        audio_duration_ms -> Nullable<Double>,
        audio_channels -> Nullable<SmallInt>,
        audio_samplerate_hz -> Nullable<Integer>,
        audio_bitrate_bps -> Nullable<Integer>,
        audio_enc_name -> Nullable<Text>,
        audio_enc_settings -> Nullable<Text>,
        color_code -> Nullable<Integer>,
    }
}

joinable!(aux_tracks_resource -> tracks_entity (track_id));

allow_tables_to_appear_in_same_query!(aux_tracks_resource, collections_entity);

table! {
    aux_tracks_overview (id) {
        id -> BigInt,
        track_id -> BigInt,
        track_title -> Text,
        track_subtitle -> Nullable<Text>,
        track_number -> Nullable<Integer>,
        track_total -> Nullable<Integer>,
        disc_number -> Nullable<Integer>,
        disc_total -> Nullable<Integer>,
        grouping -> Nullable<Text>,
        album_title -> Nullable<Text>,
        album_subtitle -> Nullable<Text>,
        album_compilation -> Nullable<Bool>,
        released_at -> Nullable<Date>,
        released_by -> Nullable<Text>,
        lyrics_explicit -> Nullable<Bool>,
    }
}

joinable!(aux_tracks_overview -> tracks_entity (track_id));

table! {
    aux_tracks_summary (id) {
        id -> BigInt,
        track_id -> BigInt,
        track_artist -> Nullable<Text>,
        track_composer -> Nullable<Text>,
        track_conductor -> Nullable<Text>,
        track_performer -> Nullable<Text>,
        track_producer -> Nullable<Text>,
        track_remixer -> Nullable<Text>,
        album_artist -> Nullable<Text>,
        album_composer -> Nullable<Text>,
        album_conductor -> Nullable<Text>,
        album_performer -> Nullable<Text>,
        album_producer -> Nullable<Text>,
        ratings_min -> Nullable<Double>,
        ratings_max -> Nullable<Double>,
    }
}

joinable!(aux_tracks_summary -> tracks_entity (track_id));

table! {
    aux_tracks_music (id) {
        id -> BigInt,
        track_id -> BigInt,
        loudness_db -> Nullable<Double>,
        tempo_bpm -> Nullable<Double>,
        time_sig_num -> Nullable<SmallInt>,
        time_sig_denom -> Nullable<SmallInt>,
        key_sig_code -> Nullable<SmallInt>,
        acousticness_score -> Nullable<Double>,
        danceability_score -> Nullable<Double>,
        energy_score -> Nullable<Double>,
        instrumentalness_score -> Nullable<Double>,
        liveness_score -> Nullable<Double>,
        popularity_score -> Nullable<Double>,
        speechiness_score -> Nullable<Double>,
        valence_score -> Nullable<Double>,
    }
}

joinable!(aux_tracks_music -> tracks_entity (track_id));

table! {
    aux_tracks_ref (id) {
        id -> BigInt,
        track_id -> BigInt,
        origin -> SmallInt,
        reference -> Text,
    }
}

joinable!(aux_tracks_ref -> tracks_entity (track_id));

table! {
    aux_tracks_tag (id) {
        id -> BigInt,
        track_id -> BigInt,
        score -> Double,
        term -> Text,
        facet -> Nullable<Text>,
    }
}

joinable!(aux_tracks_tag -> tracks_entity (track_id));

table! {
    aux_tracks_comment (id) {
        id -> BigInt,
        track_id -> BigInt,
        text -> Text,
        owner -> Nullable<Text>,
    }
}

joinable!(aux_tracks_comment -> tracks_entity (track_id));

allow_tables_to_appear_in_same_query!(
    tracks_entity,
    aux_tracks_resource,
    aux_tracks_overview,
    aux_tracks_summary,
    aux_tracks_music,
    aux_tracks_tag,
    aux_tracks_comment,
);

table! {
    aux_tracks_rating (id) {
        id -> BigInt,
        track_id -> BigInt,
        score -> Double,
        owner -> Nullable<Text>,
    }
}

joinable!(aux_tracks_rating -> tracks_entity (track_id));

allow_tables_to_appear_in_same_query!(aux_tracks_rating, tracks_entity);

table! {
    pending_tasks (id) {
        id -> BigInt,
        collection_uid -> Text,
        job_type -> Integer,
        job_params -> Binary,
    }
}

table! {
    pending_tasks_tracks (id) {
        id -> BigInt,
        task_id -> BigInt,
        track_id -> BigInt,
    }
}

joinable!(pending_tasks_tracks -> pending_tasks (task_id));
joinable!(pending_tasks_tracks -> tracks_entity (track_id));
