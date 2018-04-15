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

use std::fmt;

use chrono::{DateTime, Utc};

///////////////////////////////////////////////////////////////////////
/// EntityRevision
///////////////////////////////////////////////////////////////////////

pub type RevisionNumber = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EntityRevision {
    number: RevisionNumber,

    revisioned: DateTime<Utc>,
}

impl EntityRevision {
    pub fn is_valid(&self) -> bool {
        self.number > 0
    }

    pub fn initial() -> Self {
        EntityRevision {
            number: 1,
            revisioned: Utc::now(),
        }
    }

    pub fn is_initial(&self) -> bool {
        self.number == 1
    }

    pub fn number(&self) -> RevisionNumber {
        self.number
    }

    pub fn revisioned(&self) -> DateTime<Utc> {
        self.revisioned
    }

    pub fn next(&self) -> Self {
        assert!(self.is_valid());
        EntityRevision {
            number: self.number + 1,
            revisioned: Utc::now(),
        }
    }
}

impl fmt::Display for EntityRevision {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}@{}", self.number, self.revisioned)
    }
}

///////////////////////////////////////////////////////////////////////
/// EntityHeader
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EntityHeader {
    uid: String,

    revision: EntityRevision,
}

impl EntityHeader {
    pub fn with_uid<S: Into<String>>(uid: S) -> Self {
        let revision = EntityRevision::initial();
        Self {
            uid: uid.into(),
            revision,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.uid.is_empty() && self.revision.is_valid()
    }

    pub fn uid<'a>(&'a self) -> &'a str {
        &self.uid
    }

    pub fn revision(&self) -> EntityRevision {
        self.revision
    }

    pub fn bump_revision(&mut self) {
        self.revision = self.revision.next()
    }
}

///////////////////////////////////////////////////////////////////////
/// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(initial.revisioned() <= next.revisioned());

        let nextnext = next.next();
        assert!(nextnext.is_valid());
        assert!(!nextnext.is_initial());
        assert!(next < nextnext);
        assert!(next.number() < nextnext.number());
        assert!(next.revisioned() <= nextnext.revisioned());
    }

    #[test]
    fn header_without_uid() {
        let header = EntityHeader::with_uid("");
        assert!(!header.is_valid());
        assert!(header.revision().is_initial());
    }

    #[test]
    fn header_with_uid() {
        let header = EntityHeader::with_uid("a");
        assert!(header.is_valid());
        assert!(header.revision().is_initial());
    }

    #[test]
    fn header_bump_revision() {
        let mut header = EntityHeader::with_uid("b");
        let initial_revision = header.revision();
        assert!(initial_revision.is_initial());
        header.bump_revision();
        assert!(initial_revision < header.revision());
    }
}
