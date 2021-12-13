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

use std::{fmt, marker::PhantomData, mem, str};

use rand::{thread_rng, RngCore};

use crate::{
    prelude::*,
    util::canonical::{Canonicalize, IsCanonical},
};

///////////////////////////////////////////////////////////////////////
// EntityUid
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct EntityUid([u8; 24]);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DecodeError {
    InvalidInput(bs58::decode::Error),
    InvalidLength,
}

impl EntityUid {
    pub const SLICE_LEN: usize = mem::size_of::<Self>();
    pub const MIN_STR_LEN: usize = 32;
    pub const MAX_STR_LEN: usize = 33;
    pub const BASE58_ALPHABET: &'static bs58::alphabet::Alphabet = bs58::Alphabet::BITCOIN;

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
        let decoded_len = bs58::decode(encoded.as_bytes())
            .with_alphabet(Self::BASE58_ALPHABET)
            .into(&mut self.0)
            .map_err(DecodeError::InvalidInput)?;
        if decoded_len != self.0.len() {
            return Err(DecodeError::InvalidLength);
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EntityUidInvalidity {
    Invalid,
}

impl Validate for EntityUid {
    type Invalidity = EntityUidInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self == &Self::default(), EntityUidInvalidity::Invalid)
            .into()
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.encode_to_string().fmt(f)
    }
}

impl std::str::FromStr for EntityUid {
    type Err = DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        EntityUid::decode_from_str(s)
    }
}

///////////////////////////////////////////////////////////////////////
// EntityRevision
///////////////////////////////////////////////////////////////////////

// A 1-based, non-negative, monotone increasing number
pub type EntityRevisionNumber = u64;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct EntityRevision(EntityRevisionNumber);

impl EntityRevision {
    const fn initial() -> Self {
        Self(1)
    }

    pub fn is_initial(self) -> bool {
        self == Self::initial()
    }

    pub fn next(self) -> Self {
        debug_assert!(self.validate().is_ok());
        let Self(prev) = self;
        // TODO: Return `Option<Self>`?
        let next = prev.checked_add(1).unwrap();
        Self(next)
    }

    pub const fn from_inner(inner: EntityRevisionNumber) -> Self {
        Self(inner)
    }

    pub const fn to_inner(self) -> EntityRevisionNumber {
        let Self(inner) = self;
        inner
    }
}

impl From<EntityRevisionNumber> for EntityRevision {
    fn from(from: EntityRevisionNumber) -> Self {
        Self::from_inner(from)
    }
}

impl From<EntityRevision> for EntityRevisionNumber {
    fn from(from: EntityRevision) -> Self {
        from.to_inner()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EntityRevisionInvalidity {
    OutOfRange,
}

impl Validate for EntityRevision {
    type Invalidity = EntityRevisionInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                *self < Self::initial(),
                EntityRevisionInvalidity::OutOfRange,
            )
            .into()
    }
}

impl fmt::Display for EntityRevision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(number) = self;
        number.fmt(f)
    }
}

///////////////////////////////////////////////////////////////////////
// EntityHeader
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EntityHeader {
    pub uid: EntityUid,
    pub rev: EntityRevision,
}

impl EntityHeader {
    pub fn initial_random() -> Self {
        Self::initial_with_uid(EntityUid::random())
    }

    pub fn initial_with_uid<T: Into<EntityUid>>(uid: T) -> Self {
        let initial_rev = EntityRevision::initial();
        Self {
            uid: uid.into(),
            rev: initial_rev,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EntityHeaderInvalidity {
    Uid(EntityUidInvalidity),
    Revision(EntityRevisionInvalidity),
}

impl Validate for EntityHeader {
    type Invalidity = EntityHeaderInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.uid, Self::Invalidity::Uid)
            .validate_with(&self.rev, Self::Invalidity::Revision)
            .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Entity
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entity<T, B> {
    pub hdr: EntityHeader,
    pub body: B,
    _phantom: PhantomData<T>,
}

impl<T, B> Entity<T, B> {
    pub fn new(hdr: impl Into<EntityHeader>, body: impl Into<B>) -> Self {
        Entity {
            hdr: hdr.into(),
            body: body.into(),
            _phantom: PhantomData,
        }
    }

    pub fn try_new<TryIntoB>(
        hdr: impl Into<EntityHeader>,
        body: TryIntoB,
    ) -> Result<Self, TryIntoB::Error>
    where
        TryIntoB: TryInto<B>,
    {
        Ok(Entity {
            hdr: hdr.into(),
            body: body.try_into()?,
            _phantom: PhantomData,
        })
    }
}

impl<T, B> From<Entity<T, B>> for (EntityHeader, B) {
    fn from(from: Entity<T, B>) -> Self {
        let Entity {
            hdr,
            body,
            _phantom,
        } = from;
        (hdr, body)
    }
}

impl<'a, T, B> From<&'a Entity<T, B>> for (&'a EntityHeader, &'a B) {
    fn from(from: &'a Entity<T, B>) -> Self {
        let Entity {
            hdr,
            body,
            _phantom,
        } = from;
        (hdr, body)
    }
}

impl<T, B> IsCanonical for Entity<T, B>
where
    B: IsCanonical,
{
    fn is_canonical(&self) -> bool {
        self.body.is_canonical()
    }
}

impl<T, B> Canonicalize for Entity<T, B>
where
    B: Canonicalize,
{
    fn canonicalize(&mut self) {
        self.body.canonicalize();
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EntityInvalidity<T: Invalidity> {
    Header(EntityHeaderInvalidity),
    Body(T),
}

impl<T, B> Validate for Entity<T, B>
where
    T: Invalidity,
    B: Validate<Invalidity = T>,
{
    type Invalidity = EntityInvalidity<T>;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.hdr, EntityInvalidity::Header)
            .validate_with(&self.body, EntityInvalidity::Body)
            .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
