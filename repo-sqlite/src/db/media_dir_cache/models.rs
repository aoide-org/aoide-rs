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

use aoide_core::util::clock::{DateTime, TimestampMillis};

use aoide_repo::{collection::RecordId as CollectionId, media::dir_cache::*};

use num_traits::ToPrimitive as _;

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Queryable, Identifiable)]
#[table_name = "media_dir_cache"]
pub struct QueryableRecord {
    pub id: RowId,
    pub row_created_ms: TimestampMillis,
    pub row_updated_ms: TimestampMillis,
    pub collection_id: RowId,
    pub uri: String,
    pub status: i16,
    pub digest: EntryDigest,
}

#[derive(Debug, Insertable)]
#[table_name = "media_dir_cache"]
pub struct InsertableRecord<'a> {
    pub row_created_ms: TimestampMillis,
    pub row_updated_ms: TimestampMillis,
    pub collection_id: RowId,
    pub uri: &'a str,
    pub status: i16,
    pub digest: &'a [u8],
}

impl<'a> InsertableRecord<'a> {
    pub fn bind(
        created_at: DateTime,
        collection_id: CollectionId,
        uri: &'a str,
        status: EntryStatus,
        digest: &'a EntryDigest,
    ) -> Self {
        let row_created_ms = created_at.timestamp_millis();
        Self {
            row_created_ms,
            row_updated_ms: row_created_ms,
            collection_id: RowId::from(collection_id),
            uri,
            status: status.to_i16().expect("status"),
            digest: &digest[..],
        }
    }
}

#[derive(Debug, AsChangeset)]
#[changeset_options(treat_none_as_null = "true")]
#[table_name = "media_dir_cache"]
pub struct UpdateDigest<'a> {
    pub row_updated_ms: TimestampMillis,
    pub status: i16,
    pub digest: &'a [u8],
}

impl<'a> UpdateDigest<'a> {
    pub fn bind(updated_at: DateTime, status: EntryStatus, digest: &'a EntryDigest) -> Self {
        Self {
            row_updated_ms: updated_at.timestamp_millis(),
            status: status.to_i16().expect("status"),
            digest: &digest[..],
        }
    }
}
