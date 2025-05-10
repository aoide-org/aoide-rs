// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use diesel::prelude::*;

use aoide_core::util::clock::{TimestampMillis, UtcDateTimeMs};
use aoide_core_api::media::tracker::DirTrackingStatus;
use aoide_repo::{
    CollectionId,
    media::{DigestBytes, read_digest_from_slice, tracker::*},
};

use crate::RowId;

use super::{decode_dir_tracking_status, encode_dir_tracking_status, schema::*};

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = media_tracker_directory, primary_key(row_id))]
pub struct QueryableRecord {
    pub row_id: RowId,
    pub row_created_ms: TimestampMillis,
    pub row_updated_ms: TimestampMillis,
    pub collection_id: RowId,
    pub content_path: String,
    pub status: i16,
    pub digest: Vec<u8>,
}

impl TryFrom<QueryableRecord> for TrackedDirectory {
    type Error = anyhow::Error;

    fn try_from(from: QueryableRecord) -> anyhow::Result<Self> {
        let QueryableRecord {
            row_id: _,
            row_created_ms: _,
            row_updated_ms: _,
            collection_id: _,
            content_path,
            status,
            digest,
        } = from;
        let status = decode_dir_tracking_status(status)?;
        let digest = read_digest_from_slice(digest.as_slice())
            .ok_or_else(|| anyhow::anyhow!("invalid digest: {:?}", digest.as_slice()))?;
        let into = Self {
            content_path: content_path.into(),
            status,
            digest,
        };
        Ok(into)
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = media_tracker_directory)]
pub struct InsertableRecord<'a> {
    pub row_created_ms: TimestampMillis,
    pub row_updated_ms: TimestampMillis,
    pub collection_id: RowId,
    pub content_path: &'a str,
    pub status: i16,
    pub digest: &'a [u8],
}

impl<'a> InsertableRecord<'a> {
    pub fn bind(
        created_at: UtcDateTimeMs,
        collection_id: CollectionId,
        content_path: &'a str,
        status: DirTrackingStatus,
        digest: &'a DigestBytes,
    ) -> Self {
        let row_created_ms = created_at.unix_timestamp_millis();
        let collection_id = RowId::from(collection_id);
        let status = encode_dir_tracking_status(status);
        Self {
            row_created_ms,
            row_updated_ms: row_created_ms,
            collection_id,
            content_path,
            status,
            digest: &digest[..],
        }
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = media_tracker_directory, treat_none_as_null = true)]
pub struct UpdateDigest<'a> {
    pub row_updated_ms: TimestampMillis,
    pub status: i16,
    pub digest: &'a [u8],
}

impl<'a> UpdateDigest<'a> {
    pub fn bind(
        updated_at: UtcDateTimeMs,
        status: DirTrackingStatus,
        digest: &'a DigestBytes,
    ) -> Self {
        let status = encode_dir_tracking_status(status);
        Self {
            row_updated_ms: updated_at.unix_timestamp_millis(),
            status,
            digest: &digest[..],
        }
    }
}
