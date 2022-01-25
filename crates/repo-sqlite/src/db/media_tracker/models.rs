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

use num_traits::{FromPrimitive as _, ToPrimitive as _};

use aoide_core::util::clock::{DateTime, TimestampMillis};

use aoide_core_api::media::tracker::DirTrackingStatus;

use aoide_repo::{
    collection::RecordId as CollectionId,
    media::{read_digest_from_slice, tracker::*, DigestBytes},
};

use super::{schema::*, *};

#[derive(Debug, Queryable, Identifiable)]
#[table_name = "media_tracker_directory"]
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
            .ok_or_else(|| anyhow::anyhow!("Invalid entry status value: {}", status))?;
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
#[table_name = "media_tracker_directory"]
pub struct InsertableRecord<'a> {
    pub row_created_ms: TimestampMillis,
    pub row_updated_ms: TimestampMillis,
    pub collection_id: RowId,
    pub path: &'a str,
    pub status: i16,
    pub digest: &'a [u8],
}

impl<'a> InsertableRecord<'a> {
    pub fn bind(
        created_at: DateTime,
        collection_id: CollectionId,
        path: &'a str,
        status: DirTrackingStatus,
        digest: &'a DigestBytes,
    ) -> Self {
        let row_created_ms = created_at.timestamp_millis();
        Self {
            row_created_ms,
            row_updated_ms: row_created_ms,
            collection_id: RowId::from(collection_id),
            path,
            status: status.to_i16().expect("status"),
            digest: &digest[..],
        }
    }
}

#[derive(Debug, AsChangeset)]
#[changeset_options(treat_none_as_null = "true")]
#[table_name = "media_tracker_directory"]
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
