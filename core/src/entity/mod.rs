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

use crate::util::clock::TickInstant;

use rand::{thread_rng, RngCore};

use std::{fmt, mem, str};

///////////////////////////////////////////////////////////////////////
// EntityUid
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityUid([u8; 24]);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DecodeError {
    InvalidInput(bs58::decode::DecodeError),
    InvalidLength(usize),
}

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

    pub fn decode_str(mut self, encoded: &str) -> Result<Self, DecodeError> {
        let decoded_len =
            bs58::decode::decode_into(encoded.as_bytes(), &mut self.0, Self::BASE58_ALPHABET)
                .map_err(DecodeError::InvalidInput)?;
        if decoded_len != self.0.len() {
            return Err(DecodeError::InvalidLength(decoded_len));
        }
        Ok(self)
    }

    pub fn decode_from_str(encoded: &str) -> Result<Self, DecodeError> {
        Self::default().decode_str(encoded)
    }

    pub fn encode_to_string(&self) -> String {
        bs58::encode(self.0)
            .with_alphabet(Self::BASE58_ALPHABET)
            .into_string()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EntityUidValidation {
    Invalid,
}

impl Validate for EntityUid {
    type Validation = EntityUidValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        context.add_violation_if(self == &Self::default(), EntityUidValidation::Invalid);
        context.into_result()
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

impl fmt::Display for EntityUid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.encode_to_string())
    }
}

impl std::str::FromStr for EntityUid {
    type Err = DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        EntityUid::decode_from_str(s)
    }
}

///////////////////////////////////////////////////////////////////////
// EntityVersion
///////////////////////////////////////////////////////////////////////

pub type EntityVersionNumber = u32;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityVersion {
    major: EntityVersionNumber,
    minor: EntityVersionNumber,
}

impl EntityVersion {
    pub const fn new(major: EntityVersionNumber, minor: EntityVersionNumber) -> Self {
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
// EntityRevision
///////////////////////////////////////////////////////////////////////

pub type EntityRevisionOrdinal = u64;

pub type EntityRevisionInstant = TickInstant;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityRevision(EntityRevisionOrdinal, EntityRevisionInstant);

impl EntityRevision {
    const fn initial_ordinal() -> EntityRevisionOrdinal {
        1
    }

    pub fn new<I1: Into<EntityRevisionOrdinal>, I2: Into<EntityRevisionInstant>>(
        ordinal: I1,
        timestamp: I2,
    ) -> Self {
        EntityRevision(ordinal.into(), timestamp.into())
    }

    pub fn initial() -> Self {
        EntityRevision(Self::initial_ordinal(), TickInstant::now())
    }

    pub fn next(&self) -> Self {
        debug_assert!(self.validate().is_ok());
        self.0
            .checked_add(1)
            .map(|ordinal| EntityRevision(ordinal, TickInstant::now()))
            // TODO: Return `Option<Self>`?
            .unwrap()
    }

    pub fn is_initial(&self) -> bool {
        self.ordinal() == Self::initial_ordinal()
    }

    pub fn ordinal(&self) -> EntityRevisionOrdinal {
        self.0
    }

    pub fn instant(&self) -> EntityRevisionInstant {
        self.1
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EntityRevisionValidation {
    OutOfRange,
}

impl Validate for EntityRevision {
    type Validation = EntityRevisionValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        context.add_violation_if(
            self.ordinal() < Self::initial_ordinal(),
            EntityRevisionValidation::OutOfRange,
        );
        context.into_result()
    }
}

impl fmt::Display for EntityRevision {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}@{}", self.ordinal(), self.instant())
    }
}

///////////////////////////////////////////////////////////////////////
// EntityHeader
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityHeader {
    uid: EntityUid,
    rev: EntityRevision,
}

impl EntityHeader {
    pub fn new<I1: Into<EntityUid>, I2: Into<EntityRevision>>(uid: I1, rev: I2) -> Self {
        Self {
            uid: uid.into(),
            rev: rev.into(),
        }
    }

    pub fn initial() -> Self {
        Self::initial_with_uid(EntityUid::random())
    }

    pub fn initial_with_uid<T: Into<EntityUid>>(uid: T) -> Self {
        Self {
            uid: uid.into(),
            rev: EntityRevision::initial(),
        }
    }

    pub fn uid(&self) -> &EntityUid {
        &self.uid
    }

    pub fn rev(&self) -> &EntityRevision {
        &self.rev
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EntityHeaderValidation {
    Uid(EntityUidValidation),
    Revision(EntityRevisionValidation),
}

impl Validate for EntityHeader {
    type Validation = EntityHeaderValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        context.map_and_merge_result(self.uid.validate(), EntityHeaderValidation::Uid);
        context.map_and_merge_result(self.rev.validate(), EntityHeaderValidation::Revision);
        context.into_result()
    }
}

///////////////////////////////////////////////////////////////////////
// Entity
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct Entity<T, B> {
    header: EntityHeader,
    body: B,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, B> Entity<T, B>
where
    T: Validation,
    B: Validate<Validation = T>,
{
    pub fn new(header: EntityHeader, body: B) -> Self {
        Entity {
            header,
            body,
            _phantom: Default::default(),
        }
    }

    pub fn header(&self) -> &EntityHeader {
        &self.header
    }

    pub fn body(&self) -> &B {
        &self.body
    }

    pub fn body_mut(&mut self) -> &mut B {
        &mut self.body
    }

    pub fn replace_header_rev(self, rev: EntityRevision) -> Self {
        let header = EntityHeader::new(self.header.uid().clone(), rev);
        Self::new(header, self.body)
    }

    pub fn replace_body(self, body: B) -> Self {
        Self::new(self.header, body)
    }
}

#[derive(Debug)]
pub enum EntityValidation<T: Validation> {
    Header(EntityHeaderValidation),
    Body(T),
}

impl<T, B> Validate for Entity<T, B>
where
    T: Validation,
    B: Validate<Validation = T>,
{
    type Validation = EntityValidation<T>;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        context.map_and_merge_result(self.header().validate(), EntityValidation::Header);
        context.map_and_merge_result(self.body().validate(), EntityValidation::Body);
        context.into_result()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
