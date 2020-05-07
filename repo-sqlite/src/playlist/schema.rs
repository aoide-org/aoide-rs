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

table! {
    tbl_playlist (id) {
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

table! {
    aux_playlist_brief (id) {
        id -> BigInt,
        playlist_id -> BigInt,
        name -> Text,
        playlist_type -> Nullable<Text>, // r#type doesn't work in macro expansion
        color_rgb -> Nullable<Integer>,
        color_idx -> Nullable<SmallInt>,
        geoloc_lat -> Nullable<Double>,
        geoloc_lon -> Nullable<Double>,
        desc -> Nullable<Text>,
        tracks_count -> BigInt,
        entries_count -> BigInt,
        entries_added_min -> Nullable<Timestamp>,
        entries_added_max -> Nullable<Timestamp>,
    }
}

joinable!(aux_playlist_brief -> tbl_playlist (playlist_id));

table! {
    aux_playlist_track (id) {
        id -> BigInt,
        playlist_id -> BigInt,
        track_uid -> Binary,
        track_ref_count -> BigInt,
    }
}

allow_tables_to_appear_in_same_query!(tbl_playlist, aux_playlist_brief, aux_playlist_track);
