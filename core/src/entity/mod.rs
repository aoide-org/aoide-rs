// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use super::*;

use chrono::{DateTime, NaiveDateTime, Utc};

use failure::bail;

use rand::{thread_rng, RngCore};

use serde::{
    de::{self, Visitor as SerdeDeserializeVisitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

use std::{fmt, mem, str};

///////////////////////////////////////////////////////////////////////
/// Modules
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;

///////////////////////////////////////////////////////////////////////
/// EntityUid
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityUid([u8; 24]);

impl EntityUid {
    pub const SLICE_LEN: usize = mem::size_of::<Self>();
    pub const MIN_STR_LEN: usize = 32;
    pub const MAX_STR_LEN: usize = 33;
    pub const BASE58_ALPHABET: &'static [u8; 58] = bs58::alphabet::BITCOIN;

    pub fn random() -> Self {
        // Generate 24 random bytes
        let mut new = Self::default();
        thread_rng().fill_bytes(&mut new.0);
        new
    }

    pub fn copy_from_slice(&mut self, slice: &[u8]) {
        assert!(self.0.len() == Self::SLICE_LEN);
        assert!(slice.len() == Self::SLICE_LEN);
        self.as_mut().copy_from_slice(&slice[0..Self::SLICE_LEN]);
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        let mut result = Self::default();
        result.copy_from_slice(slice);
        result
    }

    pub fn decode_str(mut self, encoded: &str) -> Result<Self, failure::Error> {
        let decoded_len =
            bs58::decode::decode_into(encoded.as_bytes(), &mut self.0, Self::BASE58_ALPHABET)?;
        if decoded_len != self.0.len() {
            bail!(
                "Failed to decode '{}': expected bytes = {}, decoded bytes = {}",
                encoded,
                self.0.len(),
                decoded_len
            );
        }
        Ok(self)
    }

    pub fn decode_from_str(encoded: &str) -> Result<Self, failure::Error> {
        Self::default().decode_str(encoded)
    }

    pub fn encode_to_string(&self) -> String {
        bs58::encode(self.0)
            .with_alphabet(Self::BASE58_ALPHABET)
            .into_string()
    }
}

impl IsValid for EntityUid {
    fn is_valid(&self) -> bool {
        self != &Self::default()
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
        // TODO: Avoid creating a temporary string
        let encoded = self.encode_to_string();
        serializer.serialize_str(&encoded)
    }
}

#[derive(Debug, Clone, Copy)]
struct EntityUidDeserializeVisitor;

impl<'de> SerdeDeserializeVisitor<'de> for EntityUidDeserializeVisitor {
    type Value = EntityUid;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_fmt(format_args!("a base58 encoded string"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        EntityUid::decode_from_str(value).map_err(|e| E::custom(e.to_string()))
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
        write!(f, "{}", self.encode_to_string())
    }
}

///////////////////////////////////////////////////////////////////////
/// EntityVersion
///////////////////////////////////////////////////////////////////////

pub type EntityVersionNumber = u32;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EntityVersion {
    major: EntityVersionNumber,

    minor: EntityVersionNumber,
}

impl EntityVersion {
    pub fn new(major: EntityVersionNumber, minor: EntityVersionNumber) -> Self {
        EntityVersion { major, minor }
    }

    pub fn next_major(self) -> Self {
        EntityVersion {
            major: self.major + 1,
            minor: 0,
        }
    }

    pub fn next_minor(self) -> Self {
        EntityVersion {
            major: self.major,
            minor: self.minor + 1,
        }
    }

    pub fn major(self) -> EntityVersionNumber {
        self.major
    }

    pub fn minor(self) -> EntityVersionNumber {
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
    const fn initial_ordinal() -> EntityRevisionOrdinal {
        1
    }

    pub fn new<I1: Into<EntityRevisionOrdinal>, I2: Into<EntityRevisionTimestamp>>(
        ordinal: I1,
        timestamp: I2,
    ) -> Self {
        EntityRevision(ordinal.into(), timestamp.into())
    }

    pub fn initial() -> Self {
        EntityRevision(Self::initial_ordinal(), Utc::now())
    }

    pub fn next(&self) -> Self {
        debug_assert!(self.is_valid());
        self.0
            .checked_add(1)
            .map(|ordinal| EntityRevision(ordinal, Utc::now()))
            // TODO: Return `Option<Self>`?
            .unwrap()
    }

    pub fn is_initial(&self) -> bool {
        self.ordinal() == Self::initial_ordinal()
    }

    pub fn ordinal(&self) -> EntityRevisionOrdinal {
        self.0
    }

    pub fn timestamp(&self) -> EntityRevisionTimestamp {
        self.1
    }
}

impl Default for EntityRevision {
    fn default() -> EntityRevision {
        EntityRevision::new(
            0 as EntityRevisionOrdinal,
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc),
        )
    }
}

impl IsValid for EntityRevision {
    fn is_valid(&self) -> bool {
        self.ordinal() >= Self::initial_ordinal()
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

    pub fn initial() -> Self {
        Self::initial_with_uid(EntityUid::random())
    }

    pub fn initial_with_uid<T: Into<EntityUid>>(uid: T) -> Self {
        Self {
            uid: uid.into(),
            revision: EntityRevision::initial(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.uid.is_valid() && self.revision.is_valid()
    }

    pub fn uid(&self) -> &EntityUid {
        &self.uid
    }

    pub fn revision(&self) -> &EntityRevision {
        &self.revision
    }
}

///////////////////////////////////////////////////////////////////////
/// Entity
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Entity<T> {
    header: EntityHeader,

    body: T,
}

impl<T> Entity<T> {
    pub fn new(header: EntityHeader, body: T) -> Self {
        Self { header, body }
    }

    pub fn header(&self) -> &EntityHeader {
        &self.header
    }

    pub fn body(&self) -> &T {
        &self.body
    }

    pub fn body_mut(&mut self) -> &mut T {
        &mut self.body
    }

    pub fn replace_header_revision(self, revision: EntityRevision) -> Self {
        let header = EntityHeader::new(*self.header.uid(), revision);
        Self {
            header,
            body: self.body,
        }
    }

    pub fn replace_body(self, body: T) -> Self {
        Self {
            header: self.header,
            body,
        }
    }
}
