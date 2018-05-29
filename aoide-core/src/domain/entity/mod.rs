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

use chrono::{DateTime, Utc};

use ring::*;

use std::fmt;

use std::ops::Deref;

use uuid::Uuid;

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

impl EntityUidGenerator {
    pub fn generate_uid() -> EntityUid {
        let mut digest_ctx = digest::Context::new(&digest::SHA256);
        // TODO: Generate UUID v1 based on MAC address
        let uuid_v1 = Uuid::nil();
        digest_ctx.update(uuid_v1.as_bytes());
        let uuid_v4 = Uuid::new_v4();
        digest_ctx.update(uuid_v4.as_bytes());
        let now = Utc::now();
        // TODO: Avoid temporary string formatting
        digest_ctx.update(format!("{}", now).as_bytes());
        let digest = digest_ctx.finish();
        base64::encode_config(&digest, base64::URL_SAFE_NO_PAD).into()
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
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EntityRevision {
    ordinal: EntityRevisionOrdinal,

    timestamp: EntityRevisionTimestamp,
}

impl EntityRevision {
    pub fn new<I1: Into<EntityRevisionOrdinal>, I2: Into<EntityRevisionTimestamp>>(
        ordinal: I1,
        timestamp: I2,
    ) -> Self {
        Self {
            ordinal: ordinal.into(),
            timestamp: timestamp.into(),
        }
    }

    pub fn initial() -> Self {
        Self {
            ordinal: 1,
            timestamp: Utc::now(),
        }
    }

    pub fn next(&self) -> Self {
        debug_assert!(self.is_valid());
        Self {
            ordinal: self.ordinal + 1,
            timestamp: Utc::now(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.ordinal > 0
    }

    pub fn is_initial(&self) -> bool {
        self.ordinal == 1
    }

    pub fn ordinal(&self) -> EntityRevisionOrdinal {
        self.ordinal
    }

    pub fn timestamp(&self) -> EntityRevisionTimestamp {
        self.timestamp
    }
}

impl fmt::Display for EntityRevision {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}@{}", self.ordinal, self.timestamp)
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
