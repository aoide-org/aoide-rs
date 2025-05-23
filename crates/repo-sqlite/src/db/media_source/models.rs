// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::str::FromStr;

use diesel::prelude::*;
use mime::Mime;
use semval::prelude::*;

use aoide_core::{
    audio::{
        BitrateBps, BitrateBpsValue, ChannelCount, ChannelFlags, Channels, DurationMs,
        DurationMsValue, LoudnessLufs, LoudnessLufsValue, SampleRateHz, SampleRateHzValue,
    },
    media::{
        Content, Source,
        artwork::{
            ApicType, Artwork, ArtworkImage, EmbeddedArtwork, ImageDimension, ImageSize,
            LinkedArtwork,
        },
        content::{
            AudioContentMetadata, ContentLink, ContentMetadata, ContentMetadataFlags,
            ContentRevision, ContentRevisionSignedValue,
        },
    },
    util::{
        clock::*,
        color::{RgbColor, RgbColorCode},
    },
};
use aoide_repo::{CollectionId, media::source::RecordHeader};

use crate::{RowId, db::media_source::ArtworkSource, util::clock::parse_datetime};

use super::{decode_apic_type, encode_apic_type, schema::*};

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = media_source, primary_key(row_id))]
pub struct QueryableRecord {
    pub row_id: RowId,
    pub row_created_ms: TimestampMillis,
    pub row_updated_ms: TimestampMillis,
    pub collection_id: RowId,
    pub collected_at: String,
    pub collected_ms: TimestampMillis,
    pub content_link_path: String,
    pub content_link_rev: Option<ContentRevisionSignedValue>,
    pub content_type: String,
    pub content_digest: Option<Vec<u8>>,
    pub content_metadata_flags: i16,
    pub audio_duration_ms: Option<DurationMsValue>,
    pub audio_channel_count: Option<i16>,
    pub audio_channel_mask: Option<i32>,
    pub audio_samplerate_hz: Option<SampleRateHzValue>,
    pub audio_bitrate_bps: Option<BitrateBpsValue>,
    pub audio_loudness_lufs: Option<LoudnessLufsValue>,
    pub audio_encoder: Option<String>,
    pub artwork_source: Option<i16>,
    pub artwork_uri: Option<String>,
    pub artwork_apic_type: Option<i16>,
    pub artwork_media_type: Option<String>,
    pub artwork_data_size: Option<i64>,
    pub artwork_digest: Option<Vec<u8>>,
    pub artwork_image_width: Option<i16>,
    pub artwork_image_height: Option<i16>,
    pub artwork_color: Option<i32>,
    pub artwork_thumbnail: Option<Vec<u8>>,
}

impl TryFrom<QueryableRecord> for (RecordHeader, Source) {
    type Error = anyhow::Error;

    #[expect(clippy::too_many_lines)] // TODO
    fn try_from(from: self::QueryableRecord) -> anyhow::Result<Self> {
        let self::QueryableRecord {
            row_id,
            row_created_ms,
            row_updated_ms,
            collection_id: _,
            collected_at,
            collected_ms,
            content_link_path,
            content_link_rev,
            content_type,
            content_digest,
            content_metadata_flags,
            audio_duration_ms,
            audio_channel_count,
            audio_channel_mask,
            audio_samplerate_hz,
            audio_bitrate_bps,
            audio_loudness_lufs,
            audio_encoder,
            artwork_source,
            artwork_uri,
            artwork_apic_type,
            artwork_media_type,
            artwork_data_size,
            artwork_digest,
            artwork_image_width,
            artwork_image_height,
            artwork_color,
            artwork_thumbnail,
        } = from;
        let channel_flags =
            audio_channel_mask.map(|val| ChannelFlags::from_bits_truncate(val as _));
        let channel_count = audio_channel_count.map(|val| ChannelCount::new(val as _));
        let channels = Channels::try_from_flags_or_count(channel_flags, channel_count);
        let audio_metadata = AudioContentMetadata {
            duration: audio_duration_ms.map(DurationMs::new),
            channels,
            sample_rate: audio_samplerate_hz.map(SampleRateHz::new),
            bitrate: audio_bitrate_bps.map(|val| BitrateBps::new(val as BitrateBpsValue)),
            loudness: audio_loudness_lufs.map(LoudnessLufs::new),
            encoder: audio_encoder,
        };
        let artwork = if let Some(source) = artwork_source
            .map(ArtworkSource::decode)
            .transpose()
            .unwrap_or_else(|err| {
                log::error!("{err}");
                None
            }) {
            match source {
                ArtworkSource::Missing => Some(Artwork::Missing),
                ArtworkSource::Linked if artwork_uri.is_none() => {
                    anyhow::bail!("missing URI for linked artwork");
                }
                _ => {
                    let media_type = artwork_media_type
                        .as_deref()
                        .map(Mime::from_str)
                        .transpose()?;
                    if let Some(media_type) = media_type {
                        let apic_type = artwork_apic_type
                            .map(decode_apic_type)
                            .transpose()?
                            .unwrap_or(ApicType::Other);
                        let data_size = artwork_data_size.map_or(0, |size| size as _);
                        let digest = artwork_digest.and_then(|bytes| bytes.try_into().ok());
                        let image_size = if let (Some(width), Some(height)) =
                            (artwork_image_width, artwork_image_height)
                        {
                            Some(ImageSize {
                                width: width as ImageDimension,
                                height: height as ImageDimension,
                            })
                        } else {
                            None
                        };
                        let color = artwork_color.map(|code| {
                            let color = RgbColor::new(code as RgbColorCode);
                            debug_assert!(color.is_valid());
                            color
                        });
                        let thumbnail = artwork_thumbnail.and_then(|bytes| bytes.try_into().ok());
                        let image = ArtworkImage {
                            apic_type,
                            media_type,
                            data_size,
                            digest,
                            image_size,
                            color,
                            thumbnail,
                        };
                        if source == ArtworkSource::Embedded {
                            let embedded = EmbeddedArtwork { image };
                            Some(Artwork::Embedded(embedded))
                        } else {
                            let linked = LinkedArtwork {
                                uri: artwork_uri.unwrap(),
                                image,
                            };
                            Some(Artwork::Linked(linked))
                        }
                    } else {
                        debug_assert!(artwork_apic_type.is_none());
                        debug_assert!(artwork_color.is_none());
                        debug_assert!(artwork_data_size.is_none());
                        debug_assert!(artwork_digest.is_none());
                        debug_assert!(artwork_image_height.is_none());
                        debug_assert!(artwork_image_width.is_none());
                        debug_assert!(artwork_thumbnail.is_none());
                        debug_assert!(artwork_uri.is_none());
                        None
                    }
                }
            }
        } else {
            None
        };
        debug_assert!(artwork_image_width.is_some() == artwork_image_height.is_some());

        let header = RecordHeader {
            id: row_id.into(),
            created_at: UtcDateTimeMs::from_unix_timestamp_millis(row_created_ms),
            updated_at: UtcDateTimeMs::from_unix_timestamp_millis(row_updated_ms),
        };

        let collected_at = parse_datetime(&collected_at, collected_ms);
        let content_type = content_type.parse()?;
        let content_link = ContentLink {
            path: content_link_path.into(),
            rev: content_link_rev.map(ContentRevision::from_signed_value),
        };
        let content_metadata_flags =
            ContentMetadataFlags::from_bits_truncate(content_metadata_flags as u8);
        let content_metadata = ContentMetadata::Audio(audio_metadata);
        let source = Source {
            collected_at,
            content: Content {
                link: content_link,
                r#type: content_type,
                digest: content_digest,
                metadata: content_metadata,
                metadata_flags: content_metadata_flags,
            },
            artwork,
        };

        Ok((header, source))
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = media_source)]
pub struct InsertableRecord<'a> {
    pub row_created_ms: TimestampMillis,
    pub row_updated_ms: TimestampMillis,
    pub collection_id: RowId,
    pub collected_at: String,
    pub collected_ms: TimestampMillis,
    pub content_link_rev: Option<ContentRevisionSignedValue>,
    pub content_link_path: &'a str,
    pub content_type: String,
    pub content_digest: Option<&'a [u8]>,
    pub content_metadata_flags: i16,
    pub audio_duration_ms: Option<DurationMsValue>,
    pub audio_channel_count: Option<i16>,
    pub audio_samplerate_hz: Option<SampleRateHzValue>,
    pub audio_bitrate_bps: Option<BitrateBpsValue>,
    pub audio_loudness_lufs: Option<LoudnessLufsValue>,
    pub audio_encoder: Option<&'a str>,
    pub artwork_source: Option<i16>,
    pub artwork_uri: Option<&'a str>,
    pub artwork_apic_type: Option<i16>,
    pub artwork_media_type: Option<String>,
    pub artwork_data_size: Option<i64>,
    pub artwork_digest: Option<&'a [u8]>,
    pub artwork_image_width: Option<i16>,
    pub artwork_image_height: Option<i16>,
    pub artwork_color: Option<i32>,
    pub artwork_thumbnail: Option<&'a [u8]>,
}

impl<'a> InsertableRecord<'a> {
    #[expect(clippy::too_many_lines)] // TODO
    pub fn bind(
        created_at: UtcDateTimeMs,
        collection_id: CollectionId,
        created_source: &'a Source,
    ) -> Self {
        let Source {
            collected_at,
            content:
                Content {
                    link:
                        ContentLink {
                            path: content_link_path,
                            rev: content_link_rev,
                        },
                    r#type: content_type,
                    digest: content_digest,
                    metadata: content_metadata,
                    metadata_flags: content_metadata_flags,
                },
            artwork,
        } = created_source;
        let audio_metadata = {
            match content_metadata {
                ContentMetadata::Audio(audio_metadata) => Some(audio_metadata),
            }
        };
        let (artwork_source, artwork_uri, artwork_image) =
            artwork
                .as_ref()
                .map_or((None, None, None), |artwork| match artwork {
                    Artwork::Missing => (Some(ArtworkSource::Missing), None, None),
                    Artwork::Unsupported => (Some(ArtworkSource::Unsupported), None, None),
                    Artwork::Irregular => (Some(ArtworkSource::Irregular), None, None),
                    Artwork::Embedded(EmbeddedArtwork { image }) => {
                        (Some(ArtworkSource::Embedded), None, Some(image))
                    }
                    Artwork::Linked(LinkedArtwork { uri, image }) => {
                        (Some(ArtworkSource::Linked), Some(uri.as_str()), Some(image))
                    }
                });
        let artwork_apic_type;
        let artwork_media_type;
        let artwork_data_size;
        let artwork_digest;
        let artwork_image_width;
        let artwork_image_height;
        let artwork_color;
        let artwork_thumbnail;
        if let Some(image) = artwork_image {
            let ArtworkImage {
                apic_type,
                media_type,
                data_size,
                digest,
                image_size,
                color,
                thumbnail,
            } = image;
            artwork_apic_type = Some(encode_apic_type(*apic_type));
            artwork_media_type = Some(media_type.to_string());
            artwork_data_size = Some(*data_size as _);
            artwork_digest = digest.as_ref().map(|x| &x[..]);
            artwork_image_width = image_size.map(|size| size.width as _);
            artwork_image_height = image_size.map(|size| size.height as _);
            artwork_color = color.map(|color| color.code() as _);
            artwork_thumbnail = thumbnail.as_ref().map(|x| &x[..]);
        } else {
            artwork_apic_type = None;
            artwork_media_type = None;
            artwork_data_size = None;
            artwork_digest = None;
            artwork_image_width = None;
            artwork_image_height = None;
            artwork_color = None;
            artwork_thumbnail = None;
        }
        let row_created_updated_ms = created_at.unix_timestamp_millis();
        Self {
            row_created_ms: row_created_updated_ms,
            row_updated_ms: row_created_updated_ms,
            collection_id: collection_id.into(),
            collected_at: collected_at.to_string(),
            collected_ms: collected_at.to_utc().unix_timestamp_millis(),
            content_link_path: content_link_path.as_str(),
            content_link_rev: content_link_rev.map(ContentRevision::to_signed_value),
            content_type: content_type.to_string(),
            content_digest: content_digest.as_ref().map(Vec::as_slice),
            content_metadata_flags: i16::from(content_metadata_flags.bits()),
            audio_duration_ms: audio_metadata
                .and_then(|audio| audio.duration)
                .map(DurationMs::value),
            audio_channel_count: audio_metadata
                .and_then(|audio| audio.channels)
                .map(|channels| channels.count().value().cast_signed()),
            audio_samplerate_hz: audio_metadata
                .and_then(|audio| audio.sample_rate)
                .map(SampleRateHz::value),
            audio_bitrate_bps: audio_metadata
                .and_then(|audio| audio.bitrate)
                .map(BitrateBps::value),
            audio_loudness_lufs: audio_metadata
                .and_then(|audio| audio.loudness)
                .map(LoudnessLufs::value),
            audio_encoder: audio_metadata.and_then(|audio| audio.encoder.as_deref()),
            artwork_source: artwork_source.map(ArtworkSource::encode),
            artwork_uri,
            artwork_apic_type,
            artwork_media_type,
            artwork_data_size,
            artwork_digest,
            artwork_image_width,
            artwork_image_height,
            artwork_color,
            artwork_thumbnail,
        }
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = media_source, treat_none_as_null = true)]
pub struct UpdatableRecord<'a> {
    pub row_updated_ms: TimestampMillis,
    pub collected_at: String,
    pub collected_ms: TimestampMillis,
    pub content_link_rev: Option<ContentRevisionSignedValue>,
    pub content_link_path: &'a str,
    pub content_type: String,
    pub content_digest: Option<&'a [u8]>,
    pub content_metadata_flags: i16,
    pub audio_duration_ms: Option<DurationMsValue>,
    pub audio_channel_count: Option<i16>,
    pub audio_channel_mask: Option<i32>,
    pub audio_samplerate_hz: Option<SampleRateHzValue>,
    pub audio_bitrate_bps: Option<BitrateBpsValue>,
    pub audio_loudness_lufs: Option<LoudnessLufsValue>,
    pub audio_encoder: Option<&'a str>,
    pub artwork_source: Option<i16>,
    pub artwork_uri: Option<&'a str>,
    pub artwork_apic_type: Option<i16>,
    pub artwork_media_type: Option<String>,
    pub artwork_digest: Option<&'a [u8]>,
    pub artwork_data_size: Option<i64>,
    pub artwork_image_width: Option<i16>,
    pub artwork_image_height: Option<i16>,
    pub artwork_color: Option<i32>,
    pub artwork_thumbnail: Option<&'a [u8]>,
}

#[expect(clippy::too_many_lines)] // TODO
impl<'a> UpdatableRecord<'a> {
    pub fn bind(updated_at: UtcDateTimeMs, updated_source: &'a Source) -> Self {
        let Source {
            collected_at,
            content:
                Content {
                    link:
                        ContentLink {
                            path: content_link_path,
                            rev: content_link_rev,
                        },
                    r#type: content_type,
                    digest: content_digest,
                    metadata: content_metadata,
                    metadata_flags: content_metadata_flags,
                },
            artwork,
        } = updated_source;
        let audio_metadata = {
            match content_metadata {
                ContentMetadata::Audio(audio_metadata) => Some(audio_metadata),
            }
        };
        let (artwork_source, artwork_uri, artwork_image) =
            artwork
                .as_ref()
                .map_or((None, None, None), |artwork| match artwork {
                    Artwork::Missing => (Some(ArtworkSource::Missing), None, None),
                    Artwork::Unsupported => (Some(ArtworkSource::Unsupported), None, None),
                    Artwork::Irregular => (Some(ArtworkSource::Irregular), None, None),
                    Artwork::Embedded(EmbeddedArtwork { image }) => {
                        (Some(ArtworkSource::Embedded), None, Some(image))
                    }
                    Artwork::Linked(LinkedArtwork { uri, image }) => {
                        (Some(ArtworkSource::Linked), Some(uri.as_str()), Some(image))
                    }
                });
        let artwork_apic_type;
        let artwork_media_type;
        let artwork_data_size;
        let artwork_digest;
        let artwork_image_width;
        let artwork_image_height;
        let artwork_color;
        let artwork_thumbnail;
        if let Some(image) = artwork_image {
            let ArtworkImage {
                apic_type,
                media_type,
                data_size,
                digest,
                image_size,
                color,
                thumbnail,
            } = image;
            artwork_apic_type = Some(*apic_type as _);
            artwork_media_type = Some(media_type.to_string());
            artwork_data_size = Some(*data_size as _);
            artwork_digest = digest.as_ref().map(|x| &x[..]);
            artwork_image_width = image_size.map(|size| size.width as _);
            artwork_image_height = image_size.map(|size| size.height as _);
            artwork_color = color.map(|color| color.code() as _);
            artwork_thumbnail = thumbnail.as_ref().map(|x| &x[..]);
        } else {
            artwork_apic_type = None;
            artwork_media_type = None;
            artwork_data_size = None;
            artwork_digest = None;
            artwork_image_width = None;
            artwork_image_height = None;
            artwork_color = None;
            artwork_thumbnail = None;
        }
        Self {
            row_updated_ms: updated_at.unix_timestamp_millis(),
            collected_at: collected_at.to_string(),
            collected_ms: collected_at.to_utc().unix_timestamp_millis(),
            content_link_path: content_link_path.as_str(),
            content_link_rev: content_link_rev.map(ContentRevision::to_signed_value),
            content_type: content_type.to_string(),
            content_digest: content_digest.as_ref().map(Vec::as_slice),
            content_metadata_flags: i16::from(content_metadata_flags.bits()),
            audio_duration_ms: audio_metadata
                .and_then(|audio| audio.duration)
                .map(DurationMs::value),
            audio_channel_count: audio_metadata
                .and_then(|audio| audio.channels)
                .map(|channels| channels.count().value() as _),
            audio_channel_mask: audio_metadata
                .and_then(|audio| audio.channels)
                .and_then(Channels::flags)
                .map(|flags| flags.bits() as _),
            audio_samplerate_hz: audio_metadata
                .and_then(|audio| audio.sample_rate)
                .map(SampleRateHz::value),
            audio_bitrate_bps: audio_metadata
                .and_then(|audio| audio.bitrate)
                .map(BitrateBps::value),
            audio_loudness_lufs: audio_metadata
                .and_then(|audio| audio.loudness)
                .map(LoudnessLufs::value),
            audio_encoder: audio_metadata.and_then(|audio| audio.encoder.as_deref()),
            artwork_source: artwork_source.map(ArtworkSource::encode),
            artwork_uri,
            artwork_apic_type,
            artwork_media_type,
            artwork_data_size,
            artwork_digest,
            artwork_image_width,
            artwork_image_height,
            artwork_color,
            artwork_thumbnail,
        }
    }
}
