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

table! {
    tbl_collection (id) {
        id -> BigInt,
        uid -> Binary,
        rev_no -> BigInt,
        rev_ts -> BigInt,
        name -> Text,
        desc -> Nullable<Text>,
    }
}

table! {
    tbl_collection_track (id) {
        id -> BigInt,
        collection_id -> BigInt,
        track_id -> BigInt,
        added_ts -> BigInt,
        play_count -> Nullable<BigInt>,
        last_played_ts -> Nullable<BigInt>,
    }
}
