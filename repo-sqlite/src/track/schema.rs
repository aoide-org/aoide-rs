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

///////////////////////////////////////////////////////////////////////

use crate::collection::schema::{tbl_collection, tbl_collection_track};

table! {
    tbl_track (id) {
        id -> BigInt,
        uid -> Binary,
        rev_no -> BigInt,
        rev_ts -> BigInt,
        data_fmt -> SmallInt,
        data_vmaj -> SmallInt,
        data_vmin -> SmallInt,
        data_blob -> Binary,
    }
}

joinable!(tbl_collection_track -> tbl_track (track_id));

table! {
    aux_track_media (id) {
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

joinable!(aux_track_media -> tbl_track (track_id));

table! {
    aux_track_location (id) {
        id -> BigInt,
        track_id -> BigInt,
        collection_uid -> Binary,
        uri -> Text,
    }
}

joinable!(aux_track_location -> tbl_track (track_id));

table! {
    aux_track_brief (id) {
        id -> BigInt,
        track_id -> BigInt,
        track_title -> Nullable<Text>,
        track_artist -> Nullable<Text>,
        track_composer -> Nullable<Text>,
        album_title -> Nullable<Text>,
        album_artist -> Nullable<Text>,
        release_date -> Nullable<Integer>,
        track_number -> Nullable<SmallInt>,
        track_total -> Nullable<SmallInt>,
        disc_number -> Nullable<SmallInt>,
        disc_total -> Nullable<SmallInt>,
        music_bpm -> Nullable<Double>,
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

table! {
    aux_marker_label (id) {
        id -> BigInt,
        label -> Text,
    }
}

table! {
    aux_track_marker (id) {
        id -> BigInt,
        track_id -> BigInt,
        label_id -> BigInt,
    }
}

joinable!(aux_track_marker -> tbl_track (track_id));
joinable!(aux_track_marker -> aux_marker_label (label_id));

allow_tables_to_appear_in_same_query!(
    tbl_track,
    aux_track_brief,
    aux_track_location,
    aux_track_marker,
    aux_track_media,
    aux_track_tag,
    aux_tag_facet,
    aux_tag_label,
    aux_marker_label,
    tbl_collection,
    tbl_collection_track,
);
