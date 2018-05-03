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

use failure;

use mime;

use rmp_serde;

use serde;

use serde_cbor;

use serde_json;

use aoide_core::domain::entity::*;

pub mod collections;

pub mod tracks;

pub type StorageId = i64;

#[derive(Debug, Queryable)]
pub struct QueryableStorageId {
    pub id: StorageId,
}

pub type EntityStorageResult<T> = Result<T, failure::Error>;

pub trait EntityStorage {
    fn find_storage_id(&self, uid: &EntityUid) -> EntityStorageResult<Option<StorageId>>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SerializationFormat {
    JSON = 1,
    CBOR = 2,
    MessagePack = 3,
}

impl SerializationFormat {
    pub fn from(from: i16) -> Option<Self> {
        if from == (SerializationFormat::JSON as i16) {
            Some(SerializationFormat::JSON)
        } else if from == (SerializationFormat::CBOR as i16) {
            Some(SerializationFormat::CBOR)
        } else if from == (SerializationFormat::MessagePack as i16) {
            Some(SerializationFormat::MessagePack)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SerializedEntity {
    pub header: EntityHeader,

    pub format: SerializationFormat,

    pub version: EntityVersion,

    pub blob: Vec<u8>,
}

impl Into<mime::Mime> for SerializationFormat {
    fn into(self) -> mime::Mime {
        match self {
            SerializationFormat::JSON => mime::APPLICATION_JSON,
            SerializationFormat::CBOR => "application/cbor".parse::<mime::Mime>().unwrap(),
            SerializationFormat::MessagePack => mime::APPLICATION_MSGPACK,
            //_ => mime::APPLICATION_OCTET_STREAM,
        }
    }
}

pub fn serialize_entity<T>(entity: &T, format: SerializationFormat) -> Result<Vec<u8>, failure::Error> where T: serde::Serialize {
    let blob = match format {
        SerializationFormat::JSON => serde_json::to_vec(entity)?,
        SerializationFormat::CBOR => serde_cbor::to_vec(entity)?,
        SerializationFormat::MessagePack => rmp_serde::to_vec(entity)?,
        //_ => return Err(format_err!("Unsupported format for serialization: {:?}", format))
    };
    Ok(blob)
}
pub fn deserialize_entity<'a, T>(input: &'a SerializedEntity) -> Result<T, failure::Error> where T: serde::Deserialize<'a> {
    let entity = match input.format {
        SerializationFormat::JSON => serde_json::from_slice::<T>(&input.blob)?,
        SerializationFormat::CBOR => serde_cbor::from_slice::<T>(&input.blob)?,
        SerializationFormat::MessagePack => rmp_serde::from_slice::<T>(&input.blob)?,
        //_ => return Err(format_err!("Unsupported format for deserialization: {:?}", input.format))
    };
    Ok(entity)
}