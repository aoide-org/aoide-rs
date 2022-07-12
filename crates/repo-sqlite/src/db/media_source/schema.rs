// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

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
        content_link_path -> Text,
        content_link_rev -> Nullable<BigInt>,
        content_type -> Text,
        content_digest -> Nullable<Binary>,
        content_metadata_flags -> SmallInt,
        audio_duration_ms -> Nullable<Double>,
        audio_channel_count -> Nullable<SmallInt>,
        audio_samplerate_hz -> Nullable<Double>,
        audio_bitrate_bps -> Nullable<Double>,
        audio_loudness_lufs -> Nullable<Double>,
        audio_encoder -> Nullable<Text>,
        artwork_source -> Nullable<SmallInt>,
        artwork_uri -> Nullable<Text>,
        artwork_apic_type -> Nullable<SmallInt>,
        artwork_media_type -> Nullable<Text>,
        artwork_digest -> Nullable<Binary>,
        artwork_size_width -> Nullable<SmallInt>,
        artwork_size_height -> Nullable<SmallInt>,
        artwork_thumbnail -> Nullable<Binary>,
        advisory_rating -> Nullable<SmallInt>,
    }
}

joinable!(media_source -> collection (collection_id));
