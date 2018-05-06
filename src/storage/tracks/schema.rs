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
    aux_tracks_identity (id) {
        id -> BigInt,
        track_id -> BigInt,
        track_isrc -> Nullable<Text>,
        track_acoust_id -> Nullable<Text>,
        track_mbrainz_id -> Nullable<Text>,
        track_spotify_id -> Nullable<Text>,
        album_mbrainz_id -> Nullable<Text>,
        album_spotify_id -> Nullable<Text>,
        release_ean -> Nullable<Text>,
        release_upc -> Nullable<Text>,
        release_asin -> Nullable<Text>,
    }
}

joinable!(aux_tracks_identity -> tracks_entity (track_id));

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
        audio_duration -> Nullable<BigInt>,
        audio_channels -> Nullable<SmallInt>,
        audio_samplerate -> Nullable<Integer>,
        audio_bitrate -> Nullable<Integer>,
        audio_enc_name -> Nullable<Text>,
        audio_enc_settings -> Nullable<Text>,
        color_code -> Nullable<Integer>,
    }
}

joinable!(aux_tracks_resource -> tracks_entity (track_id));

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
        album_title -> Nullable<Text>,
        album_subtitle -> Nullable<Text>,
        album_grouping -> Nullable<Text>,
        album_compilation -> Nullable<Bool>,
        release_date -> Nullable<Date>,
        release_label -> Nullable<Text>,
        lyrics_explicit -> Nullable<Bool>,
    }
}

joinable!(aux_tracks_overview -> tracks_entity (track_id));

table! {
    aux_tracks_summary (id) {
        id -> BigInt,
        track_id -> BigInt,
        track_artists -> Nullable<Text>,
        track_composers -> Nullable<Text>,
        track_conductors -> Nullable<Text>,
        track_performers -> Nullable<Text>,
        track_producers -> Nullable<Text>,
        track_remixers -> Nullable<Text>,
        album_artists -> Nullable<Text>,
        album_composers -> Nullable<Text>,
        album_conductors -> Nullable<Text>,
        album_performers -> Nullable<Text>,
        album_producers -> Nullable<Text>,
        ratings_min -> Nullable<Double>,
        ratings_max -> Nullable<Double>,
    }
}

joinable!(aux_tracks_summary -> tracks_entity (track_id));

table! {
    aux_tracks_music (id) {
        id -> BigInt,
        track_id -> BigInt,
        music_loudness -> Nullable<Double>,
        music_tempo -> Nullable<Double>,
        music_time_sig_num -> Nullable<Tinyint>,
        music_time_sig_denom -> Nullable<Tinyint>,
        music_key_sig -> Nullable<Tinyint>,
        music_acousticness -> Nullable<Double>,
        music_danceability -> Nullable<Double>,
        music_energy -> Nullable<Double>,
        music_instrumentalness -> Nullable<Double>,
        music_liveness -> Nullable<Double>,
        music_popularity -> Nullable<Double>,
        music_positivity -> Nullable<Double>,
        music_speechiness -> Nullable<Double>,
    }
}

joinable!(aux_tracks_music -> tracks_entity (track_id));

table! {
    aux_tracks_tag (id) {
        id -> BigInt,
        track_id -> BigInt,
        facet -> Nullable<Text>,
        term -> Text,
        confidence -> Double,
    }
}

joinable!(aux_tracks_tag -> tracks_entity (track_id));

table! {
    aux_tracks_comment (id) {
        id -> BigInt,
        track_id -> BigInt,
        owner -> Nullable<Text>,
        comment -> Text,
    }
}

joinable!(aux_tracks_comment -> tracks_entity (track_id));

table! {
    aux_tracks_rating (id) {
        id -> BigInt,
        track_id -> BigInt,
        owner -> Nullable<Text>,
        rating -> Double,
    }
}

joinable!(aux_tracks_rating -> tracks_entity (track_id));

allow_tables_to_appear_in_same_query!(tracks_entity, aux_tracks_resource, aux_tracks_overview, aux_tracks_summary, aux_tracks_music, aux_tracks_tag, aux_tracks_comment);
allow_tables_to_appear_in_same_query!(aux_tracks_identity, tracks_entity);
allow_tables_to_appear_in_same_query!(aux_tracks_rating, tracks_entity);
