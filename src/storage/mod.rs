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

use chrono::{DateTime, Utc};
use chrono::naive::NaiveDateTime;

use failure;

use self::serde::{SerializationFormat, SerializedEntity};

use aoide_core::domain::entity::*;

pub mod collections;

pub mod tracks;

pub mod serde;

pub type StorageId = i64;

#[derive(Debug, Queryable)]
pub struct QueryableStorageId {
    pub id: StorageId,
}

#[derive(Debug, Queryable)]
pub struct QueryableSerializedEntity {
    pub id: StorageId,
    pub uid: String,
    pub rev_ordinal: i64,
    pub rev_timestamp: NaiveDateTime,
    pub ser_fmt: i16,
    pub ser_ver_major: i32,
    pub ser_ver_minor: i32,
    pub ser_blob: Vec<u8>,
}

impl From<QueryableSerializedEntity> for SerializedEntity {
    fn from(from: QueryableSerializedEntity) -> Self {
        let uid: EntityUid = from.uid.into();
        let revision = EntityRevision::new(
            from.rev_ordinal as u64,
            DateTime::from_utc(from.rev_timestamp, Utc),
        );
        let header = EntityHeader::new(uid, revision);
        let format = SerializationFormat::from(from.ser_fmt).unwrap();
        assert!(from.ser_ver_major >= 0);
        assert!(from.ser_ver_minor >= 0);
        let version = EntityVersion::new(from.ser_ver_major as u32, from.ser_ver_minor as u32);
        SerializedEntity {
            header,
            format,
            version,
            blob: from.ser_blob,
        }
    }
}

pub type EntityStorageResult<T> = Result<T, failure::Error>;

pub trait EntityStorage {
    fn find_storage_id(&self, uid: &EntityUid) -> EntityStorageResult<Option<StorageId>>;
}
