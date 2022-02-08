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

use std::str::FromStr;

use mime::Mime;
use num_traits::{FromPrimitive as _, ToPrimitive};

use aoide_core::{
    audio::{
        channel::{ChannelCount, NumberOfChannels},
        signal::{BitrateBps, BitsPerSecond, LoudnessLufs, SampleRateHz},
        DurationMs,
    },
    media::{
        artwork::{
            ApicType, Artwork, ArtworkImage, EmbeddedArtwork, ImageDimension, ImageSize,
            LinkedArtwork,
        },
        content::{
            AudioContentMetadata, ContentLink, ContentMetadata, ContentMetadataFlags,
            ContentRevision, ContentRevisionSignedValue,
        },
        AdvisoryRating, Source,
    },
    util::clock::*,
};

use aoide_repo::collection::RecordId as CollectionId;

use crate::prelude::*;

use super::{schema::*, *};

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
    pub content_link_path: String,
    pub content_link_rev: Option<ContentRevisionSignedValue>,
    pub content_type: String,
    pub content_digest: Option<Vec<u8>>,
    pub content_metadata_flags: i16,
    pub audio_duration_ms: Option<f64>,
    pub audio_channel_count: Option<i16>,
    pub audio_samplerate_hz: Option<f64>,
    pub audio_bitrate_bps: Option<f64>,
    pub audio_loudness_lufs: Option<f64>,
    pub audio_encoder: Option<String>,
    pub artwork_source: Option<i16>,
    pub artwork_uri: Option<String>,
    pub artwork_apic_type: Option<i16>,
    pub artwork_media_type: Option<String>,
    pub artwork_digest: Option<Vec<u8>>,
    pub artwork_size_width: Option<i16>,
    pub artwork_size_height: Option<i16>,
    pub artwork_thumbnail: Option<Vec<u8>>,
    pub advisory_rating: Option<i16>,
}

impl TryFrom<QueryableRecord> for (RecordHeader, Source) {
    type Error = anyhow::Error;

    fn try_from(from: self::QueryableRecord) -> anyhow::Result<Self> {
        let self::QueryableRecord {
            id,
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
            audio_samplerate_hz,
            audio_bitrate_bps,
            audio_loudness_lufs,
            audio_encoder,
            artwork_source,
            artwork_uri,
            artwork_apic_type,
            artwork_media_type,
            artwork_digest,
            artwork_size_width,
            artwork_size_height,
            artwork_thumbnail,
            advisory_rating,
        } = from;
        let audio_metadata = AudioContentMetadata {
            duration: audio_duration_ms.map(DurationMs::from_inner),
            channels: audio_channel_count.map(|val| ChannelCount(val as NumberOfChannels).into()),
            sample_rate: audio_samplerate_hz.map(SampleRateHz::from_inner),
            bitrate: audio_bitrate_bps.map(|val| BitrateBps::from_inner(val as BitsPerSecond)),
            loudness: audio_loudness_lufs.map(LoudnessLufs),
            encoder: audio_encoder,
        };
        let artwork = if let Some(source) = artwork_source.and_then(ArtworkSource::try_read) {
            match source {
                ArtworkSource::Missing => Some(Artwork::Missing),
                ArtworkSource::Linked if artwork_uri.is_none() => {
                    anyhow::bail!("Missing URI for linked artwork");
                }
                _ => {
                    let apic_type = artwork_apic_type
                        .map(|apic_type| {
                            ApicType::from_i16(apic_type)
                                .ok_or_else(|| anyhow::anyhow!("Invalid APIC type: {}", apic_type))
                        })
                        .transpose()?
                        .unwrap_or(ApicType::Other);
                    let media_type = Mime::from_str(&artwork_media_type.unwrap_or_default())?;
                    let size = if let (Some(width), Some(height)) =
                        (artwork_size_width, artwork_size_height)
                    {
                        Some(ImageSize {
                            width: width as ImageDimension,
                            height: height as ImageDimension,
                        })
                    } else {
                        None
                    };
                    let digest = artwork_digest.and_then(|bytes| bytes.try_into().ok());
                    let thumbnail = artwork_thumbnail.and_then(|bytes| bytes.try_into().ok());
                    let image = ArtworkImage {
                        apic_type,
                        media_type,
                        size,
                        digest,
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
                }
            }
        } else {
            None
        };
        debug_assert!(artwork_size_width.is_some() == artwork_size_height.is_some());

        let header = RecordHeader {
            id: id.into(),
            created_at: DateTime::new_timestamp_millis(row_created_ms),
            updated_at: DateTime::new_timestamp_millis(row_updated_ms),
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
        let advisory_rating = advisory_rating.and_then(AdvisoryRating::from_i16);
        let source = Source {
            collected_at,
            content_link,
            content_type,
            content_digest,
            content_metadata_flags,
            content_metadata,
            artwork,
            advisory_rating,
        };

        Ok((header, source))
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
    pub content_link_rev: Option<ContentRevisionSignedValue>,
    pub content_link_path: &'a str,
    pub content_type: String,
    pub advisory_rating: Option<i16>,
    pub content_digest: Option<&'a [u8]>,
    pub content_metadata_flags: i16,
    pub audio_duration_ms: Option<f64>,
    pub audio_channel_count: Option<i16>,
    pub audio_samplerate_hz: Option<f64>,
    pub audio_bitrate_bps: Option<f64>,
    pub audio_loudness_lufs: Option<f64>,
    pub audio_encoder: Option<&'a str>,
    pub artwork_source: Option<i16>,
    pub artwork_uri: Option<&'a str>,
    pub artwork_apic_type: Option<i16>,
    pub artwork_media_type: Option<String>,
    pub artwork_digest: Option<&'a [u8]>,
    pub artwork_size_width: Option<i16>,
    pub artwork_size_height: Option<i16>,
    pub artwork_thumbnail: Option<&'a [u8]>,
}

impl<'a> InsertableRecord<'a> {
    pub fn bind(
        created_at: DateTime,
        collection_id: CollectionId,
        created_source: &'a Source,
    ) -> Self {
        let Source {
            collected_at,
            content_link:
                ContentLink {
                    path: content_link_path,
                    rev: content_link_rev,
                },
            content_type,
            content_digest,
            content_metadata_flags,
            content_metadata,
            artwork,
            advisory_rating,
        } = created_source;
        let audio_metadata = {
            match content_metadata {
                ContentMetadata::Audio(ref audio_metadata) => Some(audio_metadata),
            }
        };
        let (artwork_source, artwork_uri, artwork_image) = artwork
            .as_ref()
            .map(|artwork| match artwork {
                Artwork::Missing => (Some(ArtworkSource::Missing), None, None),
                Artwork::Unsupported => (Some(ArtworkSource::Unsupported), None, None),
                Artwork::Irregular => (Some(ArtworkSource::Irregular), None, None),
                Artwork::Embedded(EmbeddedArtwork { image }) => {
                    (Some(ArtworkSource::Embedded), None, Some(image))
                }
                Artwork::Linked(LinkedArtwork { uri, image }) => {
                    (Some(ArtworkSource::Linked), Some(uri.as_str()), Some(image))
                }
            })
            .unwrap_or((None, None, None));
        let artwork_apic_type;
        let artwork_media_type;
        let artwork_size_width;
        let artwork_size_height;
        let artwork_digest;
        let artwork_thumbnail;
        if let Some(image) = artwork_image {
            let ArtworkImage {
                apic_type,
                media_type,
                size,
                digest,
                thumbnail,
            } = image;
            artwork_apic_type = apic_type.to_i16();
            artwork_media_type = Some(media_type.to_string());
            artwork_size_width = size.map(|size| size.width as i16);
            artwork_size_height = size.map(|size| size.height as i16);
            artwork_digest = digest.as_ref().map(|x| &x[..]);
            artwork_thumbnail = thumbnail.as_ref().map(|x| &x[..]);
        } else {
            artwork_apic_type = None;
            artwork_media_type = None;
            artwork_size_width = None;
            artwork_size_height = None;
            artwork_digest = None;
            artwork_thumbnail = None;
        }
        let row_created_updated_ms = created_at.timestamp_millis();
        Self {
            row_created_ms: row_created_updated_ms,
            row_updated_ms: row_created_updated_ms,
            collection_id: collection_id.into(),
            collected_at: collected_at.to_string(),
            collected_ms: collected_at.timestamp_millis(),
            content_link_path: content_link_path.as_str(),
            content_link_rev: content_link_rev.map(ContentRevision::to_signed_value),
            content_type: content_type.to_string(),
            advisory_rating: advisory_rating.as_ref().and_then(ToPrimitive::to_i16),
            content_digest: content_digest.as_ref().map(Vec::as_slice),
            content_metadata_flags: content_metadata_flags.bits() as i16,
            audio_duration_ms: audio_metadata
                .and_then(|audio| audio.duration)
                .map(DurationMs::to_inner),
            audio_channel_count: audio_metadata
                .and_then(|audio| audio.channels)
                .map(|channels| channels.count().0 as i16),
            audio_samplerate_hz: audio_metadata
                .and_then(|audio| audio.sample_rate)
                .map(SampleRateHz::to_inner),
            audio_bitrate_bps: audio_metadata
                .and_then(|audio| audio.bitrate)
                .map(BitrateBps::to_inner),
            audio_loudness_lufs: audio_metadata
                .and_then(|audio| audio.loudness)
                .map(|loudness| loudness.0),
            audio_encoder: audio_metadata.and_then(|audio| audio.encoder.as_deref()),
            artwork_source: artwork_source.map(|v| v.write() as i16),
            artwork_uri,
            artwork_apic_type,
            artwork_media_type,
            artwork_size_width,
            artwork_size_height,
            artwork_digest,
            artwork_thumbnail,
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
    pub content_link_rev: Option<ContentRevisionSignedValue>,
    pub content_link_path: &'a str,
    pub content_type: String,
    pub advisory_rating: Option<i16>,
    pub content_digest: Option<&'a [u8]>,
    pub content_metadata_flags: i16,
    pub audio_duration_ms: Option<f64>,
    pub audio_channel_count: Option<i16>,
    pub audio_samplerate_hz: Option<f64>,
    pub audio_bitrate_bps: Option<f64>,
    pub audio_loudness_lufs: Option<f64>,
    pub audio_encoder: Option<&'a str>,
    pub artwork_source: Option<i16>,
    pub artwork_uri: Option<&'a str>,
    pub artwork_apic_type: Option<i16>,
    pub artwork_media_type: Option<String>,
    pub artwork_digest: Option<&'a [u8]>,
    pub artwork_size_width: Option<i16>,
    pub artwork_size_height: Option<i16>,
    pub artwork_thumbnail: Option<&'a [u8]>,
}

impl<'a> UpdatableRecord<'a> {
    pub fn bind(updated_at: DateTime, updated_source: &'a Source) -> Self {
        let Source {
            collected_at,
            content_link:
                ContentLink {
                    path: content_link_path,
                    rev: content_link_rev,
                },
            content_type,
            content_digest,
            content_metadata_flags,
            content_metadata,
            artwork,
            advisory_rating,
        } = updated_source;
        let audio_metadata = {
            match content_metadata {
                ContentMetadata::Audio(ref audio_metadata) => Some(audio_metadata),
            }
        };
        let (artwork_source, artwork_uri, artwork_image) = artwork
            .as_ref()
            .map(|artwork| match artwork {
                Artwork::Missing => (Some(ArtworkSource::Missing), None, None),
                Artwork::Unsupported => (Some(ArtworkSource::Unsupported), None, None),
                Artwork::Irregular => (Some(ArtworkSource::Irregular), None, None),
                Artwork::Embedded(EmbeddedArtwork { image }) => {
                    (Some(ArtworkSource::Embedded), None, Some(image))
                }
                Artwork::Linked(LinkedArtwork { uri, image }) => {
                    (Some(ArtworkSource::Linked), Some(uri.as_str()), Some(image))
                }
            })
            .unwrap_or((None, None, None));
        let artwork_apic_type;
        let artwork_media_type;
        let artwork_size_width;
        let artwork_size_height;
        let artwork_digest;
        let artwork_thumbnail;
        if let Some(image) = artwork_image {
            let ArtworkImage {
                apic_type,
                media_type,
                size,
                digest,
                thumbnail,
            } = image;
            artwork_apic_type = apic_type.to_i16();
            artwork_media_type = Some(media_type.to_string());
            artwork_size_width = size.map(|size| size.width as i16);
            artwork_size_height = size.map(|size| size.height as i16);
            artwork_digest = digest.as_ref().map(|x| &x[..]);
            artwork_thumbnail = thumbnail.as_ref().map(|x| &x[..]);
        } else {
            artwork_apic_type = None;
            artwork_media_type = None;
            artwork_size_width = None;
            artwork_size_height = None;
            artwork_digest = None;
            artwork_thumbnail = None;
        }
        Self {
            row_updated_ms: updated_at.timestamp_millis(),
            collected_at: collected_at.to_string(),
            collected_ms: collected_at.timestamp_millis(),
            content_link_path: content_link_path.as_str(),
            content_link_rev: content_link_rev.map(ContentRevision::to_signed_value),
            content_type: content_type.to_string(),
            advisory_rating: advisory_rating.as_ref().and_then(ToPrimitive::to_i16),
            content_digest: content_digest.as_ref().map(Vec::as_slice),
            content_metadata_flags: content_metadata_flags.bits() as i16,
            audio_duration_ms: audio_metadata
                .and_then(|audio| audio.duration)
                .map(DurationMs::to_inner),
            audio_channel_count: audio_metadata
                .and_then(|audio| audio.channels)
                .map(|channels| channels.count().0 as i16),
            audio_samplerate_hz: audio_metadata
                .and_then(|audio| audio.sample_rate)
                .map(SampleRateHz::to_inner),
            audio_bitrate_bps: audio_metadata
                .and_then(|audio| audio.bitrate)
                .map(BitrateBps::to_inner),
            audio_loudness_lufs: audio_metadata
                .and_then(|audio| audio.loudness)
                .map(|loudness| loudness.0),
            audio_encoder: audio_metadata.and_then(|audio| audio.encoder.as_deref()),
            artwork_source: artwork_source.map(|v| v.write() as i16),
            artwork_uri,
            artwork_apic_type,
            artwork_media_type,
            artwork_size_width,
            artwork_size_height,
            artwork_digest,
            artwork_thumbnail,
        }
    }
}
