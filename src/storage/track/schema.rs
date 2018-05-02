// Aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom>
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

table! {
    track_entity (id) {
        id -> BigInt,
        uid -> Text,
        rev_ordinal -> BigInt,
        rev_timestamp -> Timestamp,
        entity_fmt -> SmallInt,
        entity_ver_major -> Integer,
        entity_ver_minor -> Integer,
        entity_blob -> Binary,
    }
}

table! {
    track_collection_resource (id) {
        id -> BigInt,
        track_id -> BigInt,
        collection_uid -> Text,
        media_uri -> Text,
        media_content_type -> Text,
        media_sync_rev_ordinal -> Nullable<BigInt>,
        media_sync_rev_timestamp -> Nullable<Timestamp>,
        audio_duration -> Nullable<BigInt>,
        audio_channels -> Nullable<SmallInt>,
        audio_samplerate -> Nullable<Integer>,
        audio_bitrate -> Nullable<Integer>,
        audio_enc_name -> Nullable<Text>,
        audio_enc_settings -> Nullable<Text>,
    }
}
