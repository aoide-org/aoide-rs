// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use crate::db::{playlist::schema::*, track::schema::*};

table! {
    playlist_entry (row_id) {
        row_id -> BigInt,
        row_created_ms -> BigInt,
        row_updated_ms -> BigInt,
        playlist_id -> BigInt,
        track_id -> Nullable<BigInt>,
        ordering -> BigInt,
        added_at -> Text,
        added_ms -> BigInt,
        title -> Nullable<Text>,
        notes -> Nullable<Text>,
    }
}

joinable!(playlist_entry -> playlist (playlist_id));
joinable!(playlist_entry -> track (track_id));
