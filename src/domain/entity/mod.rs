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

impl Into<String> for EntityUid {
    fn into(self) -> String {
        self.0
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
}

impl fmt::Display for EntityUid {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", *self)
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

    pub fn major(&self) -> EntityVersionNumber {
        self.major
    }

    pub fn minor(&self) -> EntityVersionNumber {
        self.minor
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
}

impl fmt::Display for EntityVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

///////////////////////////////////////////////////////////////////////
/// EntityRevision
///////////////////////////////////////////////////////////////////////

pub type EntityRevisionNumber = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EntityRevision {
    number: EntityRevisionNumber,

    datetime: DateTime<Utc>,
}

impl EntityRevision {
    pub fn is_valid(&self) -> bool {
        self.number > 0
    }

    pub fn initial() -> Self {
        EntityRevision {
            number: 1,
            datetime: Utc::now(),
        }
    }

    pub fn is_initial(&self) -> bool {
        self.number == 1
    }

    pub fn number(&self) -> EntityRevisionNumber {
        self.number
    }

    pub fn datetime(&self) -> DateTime<Utc> {
        self.datetime
    }

    pub fn next(&self) -> Self {
        assert!(self.is_valid());
        EntityRevision {
            number: self.number + 1,
            datetime: Utc::now(),
        }
    }
}

impl fmt::Display for EntityRevision {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}@{}", self.number, self.datetime)
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
    pub fn with_uid<I: Into<EntityUid>>(uid: I) -> Self {
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

    pub fn next_revision(&self) -> EntityHeader {
        Self {
            uid: self.uid.clone(),
            revision: self.revision.next(),
        }
    }

    pub fn bump_revision(&mut self) {
        let next_revision = self.revision.next();
        self.revision = next_revision
    }
}

///////////////////////////////////////////////////////////////////////
/// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_uid() {
        assert!(!EntityUid::default().is_valid());
    }

    #[test]
    fn generate_uid() {
        assert!(EntityUidGenerator::generate_uid().is_valid());
    }

    #[test]
    fn revision_sequence() {
        let initial = EntityRevision::initial();
        assert!(initial.is_valid());
        assert!(initial.is_initial());

        let next = initial.next();
        assert!(next.is_valid());
        assert!(!next.is_initial());
        assert!(initial < next);
        assert!(initial.number() < next.number());
        assert!(initial.datetime() <= next.datetime());

        let nextnext = next.next();
        assert!(nextnext.is_valid());
        assert!(!nextnext.is_initial());
        assert!(next < nextnext);
        assert!(next.number() < nextnext.number());
        assert!(next.datetime() <= nextnext.datetime());
    }

    #[test]
    fn header_without_uid() {
        let header = EntityHeader::with_uid(EntityUid::default());
        assert!(!header.is_valid());
        assert!(header.revision().is_initial());
    }

    #[test]
    fn header_with_uid() {
        let header = EntityHeader::with_uid(EntityUidGenerator::generate_uid());
        assert!(header.is_valid());
        assert!(header.revision().is_initial());
    }

    #[test]
    fn header_next_revision() {
        let header = EntityHeader::with_uid(EntityUidGenerator::generate_uid());
        let initial_revision = header.revision();
        assert!(initial_revision.is_initial());
        let next_revision = header.next_revision().revision();
        assert!(initial_revision < next_revision);
    }

    #[test]
    fn header_bump_revision() {
        let mut header = EntityHeader::with_uid(EntityUidGenerator::generate_uid());
        let initial_revision = header.revision();
        assert!(initial_revision.is_initial());
        header.bump_revision();
        assert!(initial_revision < header.revision());
    }
}
