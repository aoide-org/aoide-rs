// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use num_traits::{FromPrimitive as _, ToPrimitive as _};

use aoide_core::util::clock::{DateTime, TimestampMillis};

use aoide_core_api::media::tracker::DirTrackingStatus;

use aoide_repo::{
    collection::RecordId as CollectionId,
    media::{read_digest_from_slice, tracker::*, DigestBytes},
};

use super::{schema::*, *};

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = media_tracker_directory)]
pub struct QueryableRecord {
    pub id: RowId,
    pub row_created_ms: TimestampMillis,
    pub row_updated_ms: TimestampMillis,
    pub collection_id: RowId,
    pub path: String,
    pub status: i16,
    pub digest: Vec<u8>,
}

impl TryFrom<QueryableRecord> for TrackedDirectory {
    type Error = anyhow::Error;

    fn try_from(from: QueryableRecord) -> anyhow::Result<Self> {
        let QueryableRecord {
            id: _,
            row_created_ms: _,
            row_updated_ms: _,
            collection_id: _,
            path,
            status,
            digest,
        } = from;
        let status = DirTrackingStatus::from_i16(status)
            .ok_or_else(|| anyhow::anyhow!("Invalid entry status value: {status}"))?;
        let digest = read_digest_from_slice(digest.as_slice())
            .ok_or_else(|| anyhow::anyhow!("Invalid digest: {:?}", digest.as_slice()))?;
        let into = Self {
            path: path.into(),
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
        created_at: DateTime,
        collection_id: CollectionId,
        content_path: &'a str,
        status: DirTrackingStatus,
        digest: &'a DigestBytes,
    ) -> Self {
        let row_created_ms = created_at.timestamp_millis();
        Self {
            row_created_ms,
            row_updated_ms: row_created_ms,
            collection_id: RowId::from(collection_id),
            content_path,
            status: status.to_i16().expect("status"),
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
    pub fn bind(updated_at: DateTime, status: DirTrackingStatus, digest: &'a DigestBytes) -> Self {
        Self {
            row_updated_ms: updated_at.timestamp_millis(),
            status: status.to_i16().expect("status"),
            digest: &digest[..],
        }
    }
}
