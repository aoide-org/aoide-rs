// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

///////////////////////////////////////////////////////////////////////

use crate::db::track::schema::*;

diesel::table! {
    track_tag (row_id) {
        row_id -> BigInt,
        track_id -> BigInt,
        facet -> Nullable<Text>,
        label -> Nullable<Text>,
        score -> Double,
    }
}

diesel::joinable!(track_tag -> track (track_id));
