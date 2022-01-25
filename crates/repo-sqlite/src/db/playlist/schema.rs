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

use crate::db::collection::schema::*;

table! {
    playlist (row_id) {
        row_id -> BigInt,
        row_created_ms -> BigInt,
        row_updated_ms -> BigInt,
        entity_uid -> Binary,
        entity_rev -> BigInt,
        collection_id -> BigInt,
        collected_at -> Text,
        collected_ms -> BigInt,
        title -> Text,
        kind -> Nullable<Text>,
        notes -> Nullable<Text>,
        color_rgb -> Nullable<Integer>,
        color_idx -> Nullable<SmallInt>,
        flags -> SmallInt,
    }
}

joinable!(playlist -> collection (collection_id));
