// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

table! {
    collection (row_id) {
        row_id -> BigInt,
        row_created_ms -> BigInt,
        row_updated_ms -> BigInt,
        entity_uid -> Binary,
        entity_rev -> BigInt,
        title -> Text,
        kind -> Nullable<Text>,
        notes -> Nullable<Text>,
        color_rgb -> Nullable<Integer>,
        color_idx -> Nullable<SmallInt>,
        media_source_path_kind -> SmallInt,
        media_source_root_url -> Nullable<Text>,
    }
}
