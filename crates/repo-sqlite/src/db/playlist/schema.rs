// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

///////////////////////////////////////////////////////////////////////

use crate::db::collection::schema::*;

diesel::table! {
    playlist (row_id) {
        row_id -> BigInt,
        row_created_ms -> BigInt,
        row_updated_ms -> BigInt,
        entity_uid -> Text,
        entity_rev -> BigInt,
        collection_id -> Nullable<BigInt>,
        title -> Text,
        kind -> Nullable<Text>,
        notes -> Nullable<Text>,
        color_rgb -> Nullable<Integer>,
        color_idx -> Nullable<SmallInt>,
        iana_tz -> Nullable<Text>,
        flags -> SmallInt,
    }
}

diesel::joinable!(playlist -> collection (collection_id));
