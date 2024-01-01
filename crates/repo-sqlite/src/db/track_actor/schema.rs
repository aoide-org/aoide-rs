// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

///////////////////////////////////////////////////////////////////////

use crate::db::track::schema::*;

diesel::table! {
    track_actor (row_id) {
        row_id -> BigInt,
        track_id -> BigInt,
        scope -> SmallInt,
        kind -> SmallInt,
        name -> Text,
        role -> SmallInt,
        role_notes -> Nullable<Text>,
    }
}

diesel::joinable!(track_actor -> track (track_id));
