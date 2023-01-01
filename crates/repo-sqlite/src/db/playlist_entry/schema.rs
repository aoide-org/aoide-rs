// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

///////////////////////////////////////////////////////////////////////

use crate::db::{playlist::schema::*, track::schema::*};

diesel::table! {
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
        item_data -> Nullable<Text>,
    }
}

diesel::joinable!(playlist_entry -> playlist (playlist_id));
diesel::joinable!(playlist_entry -> track (track_id));
