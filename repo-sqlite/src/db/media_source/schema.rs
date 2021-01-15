// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use crate::db::collection::schema::*;

table! {
    media_source (row_id) {
        row_id -> BigInt,
        row_created_ms -> BigInt,
        row_updated_ms -> BigInt,
        collection_id -> BigInt,
        collected_at -> Text,
        collected_ms -> BigInt,
        synchronized_at -> Nullable<Text>,
        synchronized_ms -> Nullable<BigInt>,
        uri -> Text,
        uri_decoded -> Text,
        content_type -> Text,
        content_digest -> Nullable<Binary>,
        content_metadata_status -> SmallInt,
        audio_duration_ms -> Nullable<Double>,
        audio_channel_count -> Nullable<SmallInt>,
        audio_samplerate_hz -> Nullable<Double>,
        audio_bitrate_bps -> Nullable<Double>,
        audio_loudness_lufs -> Nullable<Double>,
        audio_encoder_name -> Nullable<Text>,
        audio_encoder_settings -> Nullable<Text>,
        artwork_uri -> Nullable<Text>,
        artwork_type -> Nullable<Text>,
        artwork_digest -> Nullable<Binary>,
        artwork_size_width -> Nullable<SmallInt>,
        artwork_size_height -> Nullable<SmallInt>,
        artwork_color_rgb -> Nullable<Integer>,
    }
}

joinable!(media_source -> collection (collection_id));
