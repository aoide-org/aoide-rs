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

use crate::db::{collection::schema::*, media_source::schema::*};

table! {
    media_tracker_directory (row_id) {
        row_id -> BigInt,
        row_created_ms -> BigInt,
        row_updated_ms -> BigInt,
        collection_id -> BigInt,
        path -> Text,
        status -> SmallInt,
        digest -> Binary,
    }
}

joinable!(media_tracker_directory -> collection (collection_id));

table! {
    media_tracker_source (row_id) {
        row_id -> BigInt,
        directory_id -> BigInt,
        source_id -> BigInt,
    }
}

joinable!(media_tracker_source -> media_tracker_directory (directory_id));
joinable!(media_tracker_source -> media_source (source_id));
