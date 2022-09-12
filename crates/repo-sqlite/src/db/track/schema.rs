// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

///////////////////////////////////////////////////////////////////////

use crate::db::media_source::schema::*;

diesel::table! {
    track (row_id) {
        row_id -> BigInt,
        row_created_ms -> BigInt,
        row_updated_ms -> BigInt,
        entity_uid -> Binary,
        entity_rev -> BigInt,
        media_source_id -> BigInt,
        last_synchronized_rev -> Nullable<BigInt>,
        recorded_at -> Nullable<Text>,
        recorded_ms -> Nullable<BigInt>,
        recorded_at_yyyymmdd -> Nullable<Integer>,
        released_at -> Nullable<Text>,
        released_ms -> Nullable<BigInt>,
        released_at_yyyymmdd -> Nullable<Integer>,
        released_orig_at -> Nullable<Text>,
        released_orig_ms -> Nullable<BigInt>,
        released_orig_at_yyyymmdd -> Nullable<Integer>,
        publisher -> Nullable<Text>,
        copyright -> Nullable<Text>,
        album_kind -> Nullable<SmallInt>,
        track_number -> Nullable<SmallInt>,
        track_total -> Nullable<SmallInt>,
        disc_number -> Nullable<SmallInt>,
        disc_total -> Nullable<SmallInt>,
        movement_number -> Nullable<SmallInt>,
        movement_total -> Nullable<SmallInt>,
        music_tempo_bpm -> Nullable<Double>,
        music_key_code -> Nullable<SmallInt>,
        music_beats_per_measure -> Nullable<SmallInt>,
        music_beat_unit -> Nullable<SmallInt>,
        music_flags -> SmallInt,
        color_rgb -> Nullable<Integer>,
        color_idx -> Nullable<SmallInt>,
    }
}

diesel::joinable!(track -> media_source (media_source_id));
