// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::db::{collection::schema::*, media_source::schema::*};

table! {
    media_tracker_directory (row_id) {
        row_id -> BigInt,
        row_created_ms -> BigInt,
        row_updated_ms -> BigInt,
        collection_id -> BigInt,
        content_path -> Text,
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
