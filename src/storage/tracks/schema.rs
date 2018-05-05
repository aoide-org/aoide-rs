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
    tracks_resource (id) {
        id -> BigInt,
        track_id -> BigInt,
        collection_uid -> Text,
        uri -> Text,
        sync_rev_ordinal -> Nullable<BigInt>,
        sync_rev_timestamp -> Nullable<Timestamp>,
        content_type -> Text,
        audio_duration -> Nullable<BigInt>,
        audio_channels -> Nullable<SmallInt>,
        audio_samplerate -> Nullable<Integer>,
        audio_bitrate -> Nullable<Integer>,
        audio_enc_name -> Nullable<Text>,
        audio_enc_settings -> Nullable<Text>,
    }
}

joinable!(tracks_resource -> tracks_entity (track_id));
allow_tables_to_appear_in_same_query!(tracks_resource, tracks_entity);

table! {
    tracks_tag (id) {
        id -> BigInt,
        track_id -> BigInt,
        facet -> Nullable<Text>,
        term -> Text,
        confidence -> Double,
    }
}

joinable!(tracks_tag -> tracks_entity (track_id));
allow_tables_to_appear_in_same_query!(tracks_tag, tracks_entity);

table! {
    tracks_comment (id) {
        id -> BigInt,
        track_id -> BigInt,
        owner -> Nullable<Text>,
        comment -> Text,
    }
}

joinable!(tracks_comment -> tracks_entity (track_id));
allow_tables_to_appear_in_same_query!(tracks_comment, tracks_entity);

table! {
    tracks_rating (id) {
        id -> BigInt,
        track_id -> BigInt,
        owner -> Nullable<Text>,
        rating -> Double,
    }
}

joinable!(tracks_rating -> tracks_entity (track_id));
allow_tables_to_appear_in_same_query!(tracks_rating, tracks_entity);
