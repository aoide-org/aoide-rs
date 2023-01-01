// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

///////////////////////////////////////////////////////////////////////

use crate::db::track::schema::*;

diesel::table! {
    track_cue (row_id) {
        row_id -> BigInt,
        track_id -> BigInt,
        bank_idx -> SmallInt,
        slot_idx -> Nullable<SmallInt>,
        in_position_ms -> Nullable<Double>,
        out_position_ms -> Nullable<Double>,
        out_mode -> Nullable<SmallInt>,
        kind -> Nullable<Text>,
        label -> Nullable<Text>,
        color_rgb -> Nullable<Integer>,
        color_idx -> Nullable<SmallInt>,
        flags -> SmallInt,
    }
}

diesel::joinable!(track_cue -> track (track_id));
