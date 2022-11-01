// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

///////////////////////////////////////////////////////////////////////

diesel::table! {
    view_album (phantom_id) {
        phantom_id -> BigInt,
        artist -> Text,
        title -> Text,
        kind -> Nullable<SmallInt>,
        publisher -> Nullable<Text>,
        min_recorded_at_yyyymmdd -> Nullable<Integer>,
        max_recorded_at_yyyymmdd -> Nullable<Integer>,
        min_released_at_yyyymmdd -> Nullable<Integer>,
        max_released_at_yyyymmdd -> Nullable<Integer>,
        track_count -> BigInt,
        track_id_concat -> Text,
    }
}
