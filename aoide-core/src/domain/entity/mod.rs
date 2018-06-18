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

#[cfg(test)]
mod tests;

use base64;

use chrono::{DateTime, TimeZone, Utc};

use failure;

use rand::{thread_rng, AsByteSliceMut, RngCore};

use ring::digest;

use serde::de;
use serde::de::Visitor as SerdeDeserializeVisitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::fmt;

use std::mem;

use std::str;

///////////////////////////////////////////////////////////////////////
/// EntityUid
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityUid([u8; 24]);

impl EntityUid {
    const SLICE_LEN: usize = mem::size_of::<Self>();
    const STR_LEN: usize = (Self::SLICE_LEN * 4) / 3;
    const STR_ENCODING: base64::Config = base64::URL_SAFE_NO_PAD;

    pub fn is_valid(&self) -> bool {
        self != &Self::default()
    }

    pub fn copy_from_slice(&mut self, slice: &[u8]) {
        assert!(slice.len() == Self::SLICE_LEN);
        self.as_mut().copy_from_slice(&slice[0..Self::SLICE_LEN]);
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        let mut result = Self::default();
        result.copy_from_slice(slice);
        result
    }

    pub fn decode_str(mut self, encoded: &str) -> Result<Self, failure::Error> {
        ensure!(
            encoded.len() == Self::STR_LEN,
            "Wrong encoded string slice length: expected = {}, actual = {}",
            Self::STR_LEN,
            encoded.len()
        );
        let decoded_len = base64::decode_config_slice(encoded, Self::STR_ENCODING, self.as_mut())?;
        debug_assert!(decoded_len == Self::SLICE_LEN);
        Ok(self)
    }

    pub fn encode_slice(&self, encoded: &mut [u8]) -> Result<(), failure::Error> {
        ensure!(
            encoded.len() == Self::STR_LEN,
            "Wrong encoded string slice length: expected = {}, actual = {}",
            Self::STR_LEN,
            encoded.len()
        );
        let encoded_len = base64::encode_config_slice(self.as_ref(), Self::STR_ENCODING, encoded);
        debug_assert!(encoded_len == Self::STR_LEN);
        Ok(())
    }

    pub fn encode_str(&self, encoded: &mut str) -> Result<(), failure::Error> {
        unsafe { self.encode_slice(&mut encoded.as_bytes_mut()) }
    }

    pub fn decode_from_str(encoded: &str) -> Result<Self, failure::Error> {
        Self::default().decode_str(encoded)
    }

    pub fn encode_to_slice(&self) -> [u8; Self::STR_LEN] {
        let mut encoded = [0u8; Self::STR_LEN];
        self.encode_slice(&mut encoded).unwrap();
        encoded
    }

    pub fn encode_to_string(&self) -> String {
        let mut encoded = String::with_capacity(Self::STR_LEN);
        base64::encode_config_buf(self.as_ref(), Self::STR_ENCODING, &mut encoded);
        debug_assert!(encoded.len() == Self::STR_LEN);
        encoded
    }
}

impl AsRef<[u8]> for EntityUid {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsMut<[u8]> for EntityUid {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

// Serialize (and deserialize) as string for maximum compatibility and portability
impl Serialize for EntityUid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let encoded = self.encode_to_slice();
        unsafe { serializer.serialize_str(str::from_utf8_unchecked(&encoded)) }
    }
}

#[derive(Debug, Clone, Copy)]
struct EntityUidDeserializeVisitor;

impl<'de> SerdeDeserializeVisitor<'de> for EntityUidDeserializeVisitor {
    type Value = EntityUid;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_fmt(format_args!(
            "an URL-safe Base64 encoded string of length {}",
            EntityUid::STR_LEN
        ))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match EntityUid::decode_from_str(value) {
            Ok(result) => Ok(result),
            Err(e) => Err(E::custom(e.to_string())),
        }
    }
}

impl<'de> Deserialize<'de> for EntityUid {
    fn deserialize<D>(deserializer: D) -> Result<EntityUid, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(EntityUidDeserializeVisitor)
    }
}

impl fmt::Display for EntityUid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let encoded = self.encode_to_slice();
        unsafe { write!(f, "{}", str::from_utf8_unchecked(&encoded)) }
    }
}

///////////////////////////////////////////////////////////////////////
/// EntityUidGenerator
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Default)]
pub struct EntityUidGenerator;

fn digest_timestamp<T: TimeZone>(
    digest_ctx: &mut digest::Context,
    dt: DateTime<T>,
) -> &mut digest::Context {
    let mut buf_timestamp = [dt.timestamp(); 1];
    buf_timestamp.to_le();
    digest_ctx.update(buf_timestamp.as_byte_slice_mut());
    let mut buf_subsec = [dt.timestamp_subsec_nanos(); 1];
    buf_subsec.to_le();
    digest_ctx.update(buf_subsec.as_byte_slice_mut());
    digest_ctx
}

impl EntityUidGenerator {
    pub fn generate_uid() -> EntityUid {
        let mut digest_ctx = digest::Context::new(&digest::SHA256);
        // 12 bytes from current timestamp
        digest_timestamp(&mut digest_ctx, Utc::now());
        // 16 random bytes
        let mut buf_random = [0u8, 16];
        thread_rng().fill_bytes(&mut buf_random);
        digest_ctx.update(&buf_random);
        // Calculate SHA256 of generated 32 bytes -> 32 bytes
        let digest = digest_ctx.finish();
        // Use only the first 24 bytes
        EntityUid::from_slice(&digest.as_ref()[0..EntityUid::SLICE_LEN])
    }
}

///////////////////////////////////////////////////////////////////////
/// EntityVersion
///////////////////////////////////////////////////////////////////////

pub type EntityVersionNumber = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EntityVersion {
    major: EntityVersionNumber,

    minor: EntityVersionNumber,
}

impl EntityVersion {
    pub fn new(major: EntityVersionNumber, minor: EntityVersionNumber) -> Self {
        EntityVersion { major, minor }
    }

    pub fn next_major(&self) -> Self {
        EntityVersion {
            major: self.major + 1,
            minor: 0,
        }
    }

    pub fn next_minor(&self) -> Self {
        EntityVersion {
            major: self.major,
            minor: self.minor + 1,
        }
    }

    pub fn major(&self) -> EntityVersionNumber {
        self.major
    }

    pub fn minor(&self) -> EntityVersionNumber {
        self.minor
    }
}

impl fmt::Display for EntityVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

///////////////////////////////////////////////////////////////////////
/// EntityRevision
///////////////////////////////////////////////////////////////////////

pub type EntityRevisionOrdinal = u64;

pub type EntityRevisionTimestamp = DateTime<Utc>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EntityRevision(EntityRevisionOrdinal, EntityRevisionTimestamp);

impl EntityRevision {
    pub fn new<I1: Into<EntityRevisionOrdinal>, I2: Into<EntityRevisionTimestamp>>(
        ordinal: I1,
        timestamp: I2,
    ) -> Self {
        EntityRevision(ordinal.into(), timestamp.into())
    }

    pub fn initial() -> Self {
        Self::new(1 as EntityRevisionOrdinal, Utc::now())
    }

    pub fn next(&self) -> Self {
        debug_assert!(self.is_valid());
        Self::new(self.ordinal() + 1, Utc::now())
    }

    pub fn is_valid(&self) -> bool {
        self.ordinal() > 0
    }

    pub fn is_initial(&self) -> bool {
        self.ordinal() == 1
    }

    pub fn ordinal(&self) -> EntityRevisionOrdinal {
        self.0
    }

    pub fn timestamp(&self) -> EntityRevisionTimestamp {
        self.1
    }
}

impl fmt::Display for EntityRevision {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}@{}", self.ordinal(), self.timestamp())
    }
}

///////////////////////////////////////////////////////////////////////
/// EntityHeader
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EntityHeader {
    uid: EntityUid,

    revision: EntityRevision,
}

impl EntityHeader {
    pub fn new<I1: Into<EntityUid>, I2: Into<EntityRevision>>(uid: I1, revision: I2) -> Self {
        Self {
            uid: uid.into(),
            revision: revision.into(),
        }
    }

    pub fn with_uid<T: Into<EntityUid>>(uid: T) -> Self {
        let revision = EntityRevision::initial();
        Self {
            uid: uid.into(),
            revision,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.uid.is_valid() && self.revision.is_valid()
    }

    pub fn uid<'a>(&'a self) -> &'a EntityUid {
        &self.uid
    }

    pub fn revision(&self) -> EntityRevision {
        self.revision
    }
}
