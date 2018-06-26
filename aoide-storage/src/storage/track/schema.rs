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

use storage::collection::schema::tbl_collection;

table! {
    tbl_track (id) {
        id -> BigInt,
        uid -> Binary,
        rev_ordinal -> BigInt,
        rev_timestamp -> Timestamp,
        ser_fmt -> SmallInt,
        ser_ver_major -> Integer,
        ser_ver_minor -> Integer,
        ser_blob -> Binary,
    }
}

table! {
    aux_track_resource (id) {
        id -> BigInt,
        track_id -> BigInt,
        collection_uid -> Binary,
        collection_since -> Timestamp,
        source_uri -> Text,
        source_uri_decoded -> Text,
        source_sync_when -> Nullable<Timestamp>,
        source_sync_rev_ordinal -> Nullable<BigInt>,
        source_sync_rev_timestamp -> Nullable<Timestamp>,
        media_type -> Text,
        audio_duration_ms -> Nullable<Double>,
        audio_channels_count -> Nullable<SmallInt>,
        audio_samplerate_hz -> Nullable<Integer>,
        audio_bitrate_bps -> Nullable<Integer>,
        audio_loudness_db -> Nullable<Double>,
        audio_enc_name -> Nullable<Text>,
        audio_enc_settings -> Nullable<Text>,
        color_code -> Nullable<Integer>,
    }
}

joinable!(aux_track_resource -> tbl_track (track_id));

allow_tables_to_appear_in_same_query!(aux_track_resource, tbl_collection);

table! {
    aux_track_overview (id) {
        id -> BigInt,
        track_id -> BigInt,
        track_title -> Nullable<Text>,
        track_subtitle -> Nullable<Text>,
        track_work -> Nullable<Text>,
        track_movement -> Nullable<Text>,
        album_title -> Nullable<Text>,
        album_subtitle -> Nullable<Text>,
        released_at -> Nullable<Date>,
        released_by -> Nullable<Text>,
        release_copyright -> Nullable<Text>,
        track_index -> Integer,
        track_count -> Integer,
        disc_index -> Integer,
        disc_count -> Integer,
        movement_index -> Integer,
        movement_count -> Integer,
        lyrics_explicit -> Nullable<Bool>,
        album_compilation -> Nullable<Bool>,
    }
}

joinable!(aux_track_overview -> tbl_track (track_id));

table! {
    aux_track_summary (id) {
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

joinable!(aux_track_summary -> tbl_track (track_id));

table! {
    aux_track_profile (id) {
        id -> BigInt,
        track_id -> BigInt,
        tempo_bpm -> Double,
        timesig_top -> SmallInt,
        timesig_bottom -> SmallInt,
        keysig_code -> SmallInt,
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

joinable!(aux_track_profile -> tbl_track (track_id));

table! {
    aux_track_tag_term (id) {
        id -> BigInt,
        term -> Text,
    }
}

table! {
    aux_track_tag_facet (id) {
        // TODO: Change type of id from Nullable<BigInt> to BigInt
        // See also: https://github.com/diesel-rs/diesel/pull/1644
        id -> Nullable<BigInt>,
        facet -> Text,
    }
}

table! {
    aux_track_tag (id) {
        id -> BigInt,
        track_id -> BigInt,
        score -> Double,
        term_id -> BigInt,
        facet_id -> Nullable<BigInt>,
    }
}

joinable!(aux_track_tag -> tbl_track (track_id));

joinable!(aux_track_tag -> aux_track_tag_term (term_id));

joinable!(aux_track_tag -> aux_track_tag_facet (facet_id));

table! {
    aux_track_rating (id) {
        id -> BigInt,
        track_id -> BigInt,
        score -> Double,
        owner -> Nullable<Text>,
    }
}

joinable!(aux_track_rating -> tbl_track (track_id));

allow_tables_to_appear_in_same_query!(aux_track_rating, tbl_track);

table! {
    aux_track_comment (id) {
        id -> BigInt,
        track_id -> BigInt,
        text -> Text,
        owner -> Nullable<Text>,
    }
}

joinable!(aux_track_comment -> tbl_track (track_id));

table! {
    aux_track_xref (id) {
        id -> BigInt,
        track_id -> BigInt,
        origin -> SmallInt,
        reference -> Text,
    }
}

joinable!(aux_track_xref -> tbl_track (track_id));

allow_tables_to_appear_in_same_query!(
    tbl_track,
    aux_track_resource,
    aux_track_overview,
    aux_track_summary,
    aux_track_profile,
    aux_track_tag,
    aux_track_tag_term,
    aux_track_tag_facet,
    aux_track_comment,
);

table! {
    tbl_pending_task (id) {
        id -> BigInt,
        collection_uid -> Binary,
        job_type -> Integer,
        job_params -> Binary,
    }
}

table! {
    tbl_pending_task_track (id) {
        id -> BigInt,
        task_id -> BigInt,
        track_id -> BigInt,
    }
}

joinable!(tbl_pending_task_track -> tbl_pending_task (task_id));
joinable!(tbl_pending_task_track -> tbl_track (track_id));
