// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

///////////////////////////////////////////////////////////////////////

use crate::db::media_source::schema::*;

table! {
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
        album_kind -> SmallInt,
        track_number -> Nullable<SmallInt>,
        track_total -> Nullable<SmallInt>,
        disc_number -> Nullable<SmallInt>,
        disc_total -> Nullable<SmallInt>,
        movement_number -> Nullable<SmallInt>,
        movement_total -> Nullable<SmallInt>,
        music_tempo_bpm -> Nullable<Double>,
        music_key_code -> SmallInt,
        music_beats_per_measure -> Nullable<SmallInt>,
        music_beat_unit -> Nullable<SmallInt>,
        music_flags -> SmallInt,
        color_rgb -> Nullable<Integer>,
        color_idx -> Nullable<SmallInt>,
        aux_track_title -> Nullable<Text>,
        aux_track_artist -> Nullable<Text>,
        aux_track_composer -> Nullable<Text>,
        aux_album_title -> Nullable<Text>,
        aux_album_artist -> Nullable<Text>,
    }
}

joinable!(track -> media_source (media_source_id));
