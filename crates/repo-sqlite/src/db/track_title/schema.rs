// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

///////////////////////////////////////////////////////////////////////

use crate::db::track::schema::*;

table! {
    track_title (row_id) {
        row_id -> BigInt,
        track_id -> BigInt,
        scope -> SmallInt,
        kind -> SmallInt,
        name -> Text,
    }
}

joinable!(track_title -> track (track_id));
