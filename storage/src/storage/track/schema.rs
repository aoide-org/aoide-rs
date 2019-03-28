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

use crate::storage::collection::schema::tbl_collection;

///////////////////////////////////////////////////////////////////////

table! {
    tbl_track (id) {
        id -> BigInt,
        uid -> Binary,
        rev_no -> BigInt,
        rev_ts -> BigInt,
        ser_fmt -> SmallInt,
        ser_vmaj -> Integer,
        ser_vmin -> Integer,
        ser_blob -> Binary,
    }
}

table! {
    aux_track_collection (id) {
        id -> BigInt,
        track_id -> BigInt,
        collection_uid -> Binary,
        since -> Timestamp,
        color_code -> Nullable<Integer>,
        play_count -> Nullable<Integer>,
    }
}

joinable!(aux_track_collection -> tbl_track (track_id));

allow_tables_to_appear_in_same_query!(aux_track_collection, tbl_collection);

table! {
    aux_track_source (id) {
        id -> BigInt,
        track_id -> BigInt,
        uri -> Text,
        uri_decoded -> Text,
        content_type -> Text,
        audio_channel_count -> Nullable<SmallInt>,
        audio_duration -> Nullable<Double>,
        audio_samplerate -> Nullable<Integer>,
        audio_bitrate -> Nullable<Integer>,
        audio_loudness -> Nullable<Double>,
        audio_enc_name -> Nullable<Text>,
        audio_enc_settings -> Nullable<Text>,
    }
}

joinable!(aux_track_source -> tbl_track (track_id));

table! {
    aux_track_brief (id) {
        id -> BigInt,
        track_id -> BigInt,
        track_title -> Nullable<Text>,
        track_artist -> Nullable<Text>,
        track_composer -> Nullable<Text>,
        album_title -> Nullable<Text>,
        album_artist -> Nullable<Text>,
        release_year -> Nullable<SmallInt>,
        track_index -> Nullable<SmallInt>,
        track_count -> Nullable<SmallInt>,
        disc_index -> Nullable<SmallInt>,
        disc_count -> Nullable<SmallInt>,
        music_tempo -> Nullable<Double>,
        music_key -> Nullable<SmallInt>,
    }
}

joinable!(aux_track_brief -> tbl_track (track_id));

table! {
    aux_tag_label (id) {
        id -> BigInt,
        label -> Text,
    }
}

table! {
    aux_tag_facet (id) {
        id -> BigInt,
        facet -> Text,
    }
}

table! {
    aux_track_tag (id) {
        id -> BigInt,
        track_id -> BigInt,
        facet_id -> Nullable<BigInt>,
        label_id -> Nullable<BigInt>,
        score -> Double,
    }
}

joinable!(aux_track_tag -> tbl_track (track_id));
joinable!(aux_track_tag -> aux_tag_label (label_id));
joinable!(aux_track_tag -> aux_tag_facet (facet_id));

allow_tables_to_appear_in_same_query!(
    tbl_track,
    aux_track_collection,
    aux_track_source,
    aux_track_brief,
    aux_track_tag,
    aux_tag_label,
    aux_tag_facet,
);
