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

use super::{schema::*, *};

use crate::prelude::*;

use aoide_core::{
    audio::{
        channel::{ChannelCount, NumberOfChannels},
        signal::{BitrateBps, BitsPerSecond, LoudnessLufs, SampleRateHz, SamplesPerSecond},
        AudioContent, DurationInMilliseconds, DurationMs,
    },
    media::{Artwork, Content, ContentMetadataFlags, ImageDimension, ImageSize, Source},
    util::{
        clock::*,
        color::{RgbColor, RgbColorCode},
    },
};

use aoide_repo::collection::RecordId as CollectionId;

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Queryable, Identifiable)]
#[table_name = "media_source"]
pub struct QueryableRecord {
    pub id: RowId,
    pub row_created_ms: TimestampMillis,
    pub row_updated_ms: TimestampMillis,
    pub collection_id: RowId,
    pub collected_at: String,
    pub collected_ms: TimestampMillis,
    pub synchronized_at: Option<String>,
    pub synchronized_ms: Option<TimestampMillis>,
    pub uri: String,
    pub content_type: String,
    pub content_digest: Option<Vec<u8>>,
    pub content_metadata_flags: i16,
    pub audio_duration_ms: Option<f64>,
    pub audio_channel_count: Option<i16>,
    pub audio_samplerate_hz: Option<f64>,
    pub audio_bitrate_bps: Option<f64>,
    pub audio_loudness_lufs: Option<f64>,
    pub audio_encoder: Option<String>,
    pub artwork_uri: Option<String>,
    pub artwork_type: Option<String>,
    pub artwork_digest: Option<Vec<u8>>,
    pub artwork_size_width: Option<i16>,
    pub artwork_size_height: Option<i16>,
    pub artwork_color_rgb: Option<i32>,
}

impl From<QueryableRecord> for (RecordHeader, Source) {
    fn from(from: self::QueryableRecord) -> Self {
        let self::QueryableRecord {
            id,
            row_created_ms,
            row_updated_ms,
            collection_id: _,
            collected_at,
            collected_ms,
            synchronized_at,
            synchronized_ms,
            uri,
            content_type,
            content_digest,
            content_metadata_flags,
            audio_duration_ms,
            audio_channel_count,
            audio_samplerate_hz,
            audio_bitrate_bps,
            audio_loudness_lufs,
            audio_encoder,
            artwork_uri,
            artwork_type,
            artwork_digest,
            artwork_size_width,
            artwork_size_height,
            artwork_color_rgb,
        } = from;
        let audio_content = AudioContent {
            duration: audio_duration_ms.map(|val| DurationMs(val as DurationInMilliseconds)),
            channels: audio_channel_count.map(|val| ChannelCount(val as NumberOfChannels).into()),
            sample_rate: audio_samplerate_hz.map(|val| SampleRateHz(val as SamplesPerSecond)),
            bitrate: audio_bitrate_bps.map(|val| BitrateBps(val as BitsPerSecond)),
            loudness: audio_loudness_lufs.map(LoudnessLufs),
            encoder: audio_encoder,
        };
        debug_assert!(artwork_size_width.is_some() == artwork_size_height.is_some());
        let image_size =
            if let (Some(width), Some(height)) = (artwork_size_width, artwork_size_height) {
                Some(ImageSize {
                    width: width as ImageDimension,
                    height: height as ImageDimension,
                })
            } else {
                None
            };
        let artwork = Artwork {
            uri: artwork_uri,
            media_type: artwork_type,
            digest: artwork_digest,
            size: image_size,
            color_rgb: artwork_color_rgb.map(|code| RgbColor(code as RgbColorCode)),
        };
        let header = RecordHeader {
            id: id.into(),
            created_at: DateTime::new_timestamp_millis(row_created_ms),
            updated_at: DateTime::new_timestamp_millis(row_updated_ms),
        };
        let source = Source {
            collected_at: parse_datetime(&collected_at, collected_ms),
            synchronized_at: parse_datetime_opt(&synchronized_at, synchronized_ms),
            uri,
            content_type,
            content_digest,
            content_metadata_flags: ContentMetadataFlags::from_bits_truncate(
                content_metadata_flags as u8,
            ),
            content: Content::Audio(audio_content),
            artwork,
        };
        (header, source)
    }
}

#[derive(Debug, Insertable)]
#[table_name = "media_source"]
pub struct InsertableRecord<'a> {
    pub row_created_ms: TimestampMillis,
    pub row_updated_ms: TimestampMillis,
    pub collection_id: RowId,
    pub collected_at: String,
    pub collected_ms: TimestampMillis,
    pub synchronized_at: Option<String>,
    pub synchronized_ms: Option<TimestampMillis>,
    pub uri: &'a str,
    pub content_type: &'a str,
    pub content_digest: Option<&'a [u8]>,
    pub content_metadata_flags: i16,
    pub audio_duration_ms: Option<f64>,
    pub audio_channel_count: Option<i16>,
    pub audio_samplerate_hz: Option<f64>,
    pub audio_bitrate_bps: Option<f64>,
    pub audio_loudness_lufs: Option<f64>,
    pub audio_encoder: Option<&'a str>,
    pub artwork_uri: Option<&'a str>,
    pub artwork_type: Option<&'a str>,
    pub artwork_digest: Option<&'a [u8]>,
    pub artwork_size_width: Option<i16>,
    pub artwork_size_height: Option<i16>,
    pub artwork_color_rgb: Option<i32>,
}

impl<'a> InsertableRecord<'a> {
    pub fn bind(
        created_at: DateTime,
        collection_id: CollectionId,
        created_source: &'a Source,
    ) -> Self {
        let Source {
            collected_at,
            synchronized_at,
            uri,
            content_type,
            content_digest,
            content_metadata_flags,
            content,
            artwork,
        } = created_source;
        let audio_content = {
            match content {
                Content::Audio(ref audio_content) => Some(audio_content),
            }
        };
        let Artwork {
            uri: artwork_uri,
            media_type: artwork_type,
            digest: artwork_digest,
            size: artwork_size,
            color_rgb: artwork_color_rgb,
        } = artwork;
        let row_created_updated_ms = created_at.timestamp_millis();
        Self {
            row_created_ms: row_created_updated_ms,
            row_updated_ms: row_created_updated_ms,
            collection_id: collection_id.into(),
            collected_at: collected_at.to_string(),
            collected_ms: collected_at.timestamp_millis(),
            synchronized_at: synchronized_at.as_ref().map(ToString::to_string),
            synchronized_ms: synchronized_at.map(DateTime::timestamp_millis),
            uri: uri.as_str(),
            content_type: content_type.as_str(),
            content_digest: content_digest.as_ref().map(Vec::as_slice),
            content_metadata_flags: content_metadata_flags.bits() as i16,
            audio_duration_ms: audio_content
                .and_then(|audio| audio.duration)
                .map(|sample_rate| sample_rate.0),
            audio_channel_count: audio_content
                .and_then(|audio| audio.channels)
                .map(|channels| channels.count().0 as i16),
            audio_samplerate_hz: audio_content
                .and_then(|audio| audio.sample_rate)
                .map(|sample_rate| sample_rate.0),
            audio_bitrate_bps: audio_content
                .and_then(|audio| audio.bitrate)
                .map(|bitrate| bitrate.0),
            audio_loudness_lufs: audio_content
                .and_then(|audio| audio.loudness)
                .map(|loudness| loudness.0),
            audio_encoder: audio_content
                .and_then(|audio| audio.encoder.as_ref().map(|s| s.as_str())),
            artwork_uri: artwork_uri.as_ref().map(String::as_str),
            artwork_type: artwork_type.as_ref().map(String::as_str),
            artwork_digest: artwork_digest.as_ref().map(Vec::as_slice),
            artwork_size_width: artwork_size.map(|size| size.width as i16),
            artwork_size_height: artwork_size.map(|size| size.height as i16),
            artwork_color_rgb: artwork_color_rgb.map(|color| color.code() as i32),
        }
    }
}

#[derive(Debug, AsChangeset)]
#[table_name = "media_source"]
#[changeset_options(treat_none_as_null = "true")]
pub struct UpdatableRecord<'a> {
    pub row_updated_ms: TimestampMillis,
    pub collected_at: String,
    pub collected_ms: TimestampMillis,
    pub synchronized_at: Option<String>,
    pub synchronized_ms: Option<TimestampMillis>,
    pub uri: &'a str,
    pub content_type: &'a str,
    pub content_digest: Option<&'a [u8]>,
    pub content_metadata_flags: i16,
    pub audio_duration_ms: Option<f64>,
    pub audio_channel_count: Option<i16>,
    pub audio_samplerate_hz: Option<f64>,
    pub audio_bitrate_bps: Option<f64>,
    pub audio_loudness_lufs: Option<f64>,
    pub audio_encoder: Option<&'a str>,
    pub artwork_uri: Option<&'a str>,
    pub artwork_type: Option<&'a str>,
    pub artwork_digest: Option<&'a [u8]>,
    pub artwork_size_width: Option<i16>,
    pub artwork_size_height: Option<i16>,
    pub artwork_color_rgb: Option<i32>,
}

impl<'a> UpdatableRecord<'a> {
    pub fn bind(updated_at: DateTime, updated_source: &'a Source) -> Self {
        let Source {
            collected_at,
            synchronized_at,
            uri,
            content_type,
            content_digest,
            content_metadata_flags,
            content,
            artwork,
        } = updated_source;
        let audio_content = {
            match content {
                Content::Audio(ref audio_content) => Some(audio_content),
            }
        };
        let Artwork {
            uri: artwork_uri,
            media_type: artwork_type,
            digest: artwork_digest,
            size: artwork_size,
            color_rgb: artwork_color_rgb,
        } = artwork;
        Self {
            row_updated_ms: updated_at.timestamp_millis(),
            collected_at: collected_at.to_string(),
            collected_ms: collected_at.timestamp_millis(),
            synchronized_at: synchronized_at.as_ref().map(ToString::to_string),
            synchronized_ms: synchronized_at.map(DateTime::timestamp_millis),
            uri: uri.as_str(),
            content_type: content_type.as_str(),
            content_digest: content_digest.as_ref().map(Vec::as_slice),
            content_metadata_flags: content_metadata_flags.bits() as i16,
            audio_duration_ms: audio_content
                .and_then(|audio| audio.duration)
                .map(|sample_rate| sample_rate.0),
            audio_channel_count: audio_content
                .and_then(|audio| audio.channels)
                .map(|channels| channels.count().0 as i16),
            audio_samplerate_hz: audio_content
                .and_then(|audio| audio.sample_rate)
                .map(|sample_rate| sample_rate.0),
            audio_bitrate_bps: audio_content
                .and_then(|audio| audio.bitrate)
                .map(|bitrate| bitrate.0),
            audio_loudness_lufs: audio_content
                .and_then(|audio| audio.loudness)
                .map(|loudness| loudness.0),
            audio_encoder: audio_content
                .and_then(|audio| audio.encoder.as_ref().map(|s| s.as_str())),
            artwork_uri: artwork_uri.as_ref().map(String::as_str),
            artwork_type: artwork_type.as_ref().map(String::as_str),
            artwork_digest: artwork_digest.as_ref().map(Vec::as_slice),
            artwork_size_width: artwork_size.map(|size| size.width as i16),
            artwork_size_height: artwork_size.map(|size| size.height as i16),
            artwork_color_rgb: artwork_color_rgb.map(|color| color.code() as i32),
        }
    }
}
