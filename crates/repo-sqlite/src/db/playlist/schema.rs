// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

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
