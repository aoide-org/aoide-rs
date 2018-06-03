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

use rand::{thread_rng, RngCore, AsByteSliceMut};

use ring::digest;

use std::fmt;

use std::ops::Deref;

///////////////////////////////////////////////////////////////////////
/// EntityUid
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EntityUid(String);

impl From<String> for EntityUid {
    fn from(from: String) -> Self {
        EntityUid(from)
    }
}

impl From<EntityUid> for String {
    fn from(from: EntityUid) -> Self {
        from.0
    }
}

impl Deref for EntityUid {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl EntityUid {
    pub fn is_valid(&self) -> bool {
        !(*self).is_empty()
    }

    pub fn as_str<'a>(&'a self) -> &'a str {
        &self.0
    }
}

impl fmt::Display for EntityUid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

///////////////////////////////////////////////////////////////////////
/// EntityUidGenerator
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default)]
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
        // Encode the first 24 bytes as string -> 32 URL-safe chars
        base64::encode_config(&digest.as_ref()[0..24], base64::URL_SAFE_NO_PAD).into()
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

    pub fn update_revision(&mut self, next_revision: EntityRevision) {
        debug_assert!(self.revision < next_revision);
        self.revision = next_revision;
    }
}
