// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use std::{
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    str,
};

use rand::RngCore;
use thiserror::Error;

use crate::{
    prelude::*,
    util::canonical::{Canonicalize, IsCanonical},
};

///////////////////////////////////////////////////////////////////////
// EntityUid
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityUid([u8; 24]);

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error(transparent)]
    InvalidInput(#[from] bs58::decode::Error),

    #[error("invalid length")]
    InvalidLength,
}

impl EntityUid {
    pub const SLICE_LEN: usize = mem::size_of::<Self>();
    pub const MIN_STR_LEN: usize = 32;
    pub const MAX_STR_LEN: usize = 33;
    pub const BASE58_ALPHABET: &'static bs58::alphabet::Alphabet = bs58::Alphabet::BITCOIN;

    #[cfg(target_family = "wasm")]
    #[must_use]
    pub fn random() -> Self {
        let mut new = Self::default();
        // Generate 24 random bytes
        getrandom::getrandom(&mut new.0).expect("random bytes");
        new
    }

    #[cfg(not(target_family = "wasm"))]
    #[must_use]
    pub fn random() -> Self {
        Self::random_with(&mut crate::util::random::adhoc_rng())
    }

    #[must_use]
    pub fn random_with<T: RngCore>(rng: &mut T) -> Self {
        let mut new = Self::default();
        // Generate 24 random bytes
        rand::RngCore::fill_bytes(rng, &mut new.0);
        new
    }

    pub fn copy_from_slice(&mut self, slice: &[u8]) {
        assert!(self.0.len() == Self::SLICE_LEN);
        assert!(slice.len() == Self::SLICE_LEN);
        self.as_mut().copy_from_slice(&slice[0..Self::SLICE_LEN]);
    }

    #[must_use]
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

    #[must_use]
    pub fn encode_to_string(&self) -> String {
        bs58::encode(self.0)
            .with_alphabet(Self::BASE58_ALPHABET)
            .into_string()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum EntityUidInvalidity {
    Invalid,
}

impl Validate for EntityUid {
    type Invalidity = EntityUidInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self == &Self::default(), Self::Invalidity::Invalid)
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

pub struct EntityUidTyped<T: 'static> {
    untyped: EntityUid,
    typed_marker: PhantomData<&'static T>,
}

impl<T> EntityUidTyped<T> {
    #[must_use]
    pub const fn from_untyped(untyped: EntityUid) -> Self {
        Self {
            untyped,
            typed_marker: PhantomData,
        }
    }

    #[must_use]
    pub const fn into_untyped(self) -> EntityUid {
        let Self {
            untyped,
            typed_marker: _,
        } = self;
        untyped
    }
}

impl<T> From<EntityUidTyped<T>> for EntityUid {
    fn from(from: EntityUidTyped<T>) -> EntityUid {
        from.into_untyped()
    }
}

impl<T> Deref for EntityUidTyped<T> {
    type Target = EntityUid;

    fn deref(&self) -> &Self::Target {
        &self.untyped
    }
}

impl<T> DerefMut for EntityUidTyped<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.untyped
    }
}

impl<T> fmt::Display for EntityUidTyped<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl<T> std::str::FromStr for EntityUidTyped<T> {
    type Err = DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        EntityUid::from_str(s).map(Self::from_untyped)
    }
}

impl<T> Default for EntityUidTyped<T> {
    fn default() -> Self {
        Self {
            untyped: Default::default(),
            typed_marker: PhantomData,
        }
    }
}

impl<T> fmt::Debug for EntityUidTyped<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl<T> Clone for EntityUidTyped<T> {
    fn clone(&self) -> Self {
        Self {
            untyped: self.untyped.clone(),
            typed_marker: PhantomData,
        }
    }
}

impl<T> PartialEq for EntityUidTyped<T> {
    fn eq(&self, other: &Self) -> bool {
        self.deref().eq(&*other)
    }
}

impl<T> Eq for EntityUidTyped<T> {}

impl<T> PartialOrd for EntityUidTyped<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.deref().partial_cmp(&*other)
    }
}

impl<T> Ord for EntityUidTyped<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.deref().cmp(&*other)
    }
}

impl<T> Hash for EntityUidTyped<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}

impl<T> Validate for EntityUidTyped<T> {
    type Invalidity = EntityUidInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        self.deref().validate()
    }
}

///////////////////////////////////////////////////////////////////////
// EntityRevision
///////////////////////////////////////////////////////////////////////

// A 1-based, non-negative, monotone increasing number
pub type EntityRevisionNumber = u64;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityRevision(EntityRevisionNumber);

impl EntityRevision {
    const fn initial() -> Self {
        Self(1)
    }

    #[must_use]
    pub fn is_initial(self) -> bool {
        self == Self::initial()
    }

    pub fn prev(self) -> Option<Self> {
        debug_assert!(self.validate().is_ok());
        let Self(next) = self;
        next.checked_sub(1).map(Self::from_inner)
    }

    pub fn next(self) -> Option<Self> {
        debug_assert!(self.validate().is_ok());
        let Self(prev) = self;
        prev.checked_add(1).map(Self::from_inner)
    }

    #[must_use]
    pub const fn from_inner(inner: EntityRevisionNumber) -> Self {
        Self(inner)
    }

    #[must_use]
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

#[derive(Copy, Clone, Debug)]
pub enum EntityRevisionInvalidity {
    OutOfRange,
}

impl Validate for EntityRevision {
    type Invalidity = EntityRevisionInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(*self < Self::initial(), Self::Invalidity::OutOfRange)
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EntityHeader {
    pub uid: EntityUid,
    pub rev: EntityRevision,
}

impl EntityHeader {
    #[must_use]
    pub fn initial_random() -> Self {
        Self::initial_with_uid(EntityUid::random())
    }

    #[must_use]
    pub fn initial_random_with<T: RngCore>(rng: &mut T) -> Self {
        Self::initial_with_uid(EntityUid::random_with(rng))
    }

    #[must_use]
    pub fn initial_with_uid<T: Into<EntityUid>>(uid: T) -> Self {
        let initial_rev = EntityRevision::initial();
        Self {
            uid: uid.into(),
            rev: initial_rev,
        }
    }

    #[must_use]
    pub fn next_rev(self) -> Option<Self> {
        let Self { uid, rev } = self;
        rev.next().map(|rev| Self { uid, rev })
    }

    #[must_use]
    pub fn prev_rev(self) -> Option<Self> {
        let Self { uid, rev } = self;
        rev.prev().map(|rev| Self { uid, rev })
    }
}

#[derive(Copy, Clone, Debug)]
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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EntityHeaderTyped<T: 'static> {
    pub uid: EntityUidTyped<T>,
    pub rev: EntityRevision,
}

impl<T> EntityHeaderTyped<T> {
    #[must_use]
    pub const fn from_untyped(untyped: EntityHeader) -> Self {
        let EntityHeader { uid, rev } = untyped;
        Self {
            uid: EntityUidTyped::from_untyped(uid),
            rev,
        }
    }

    #[must_use]
    pub const fn into_untyped(self) -> EntityHeader {
        let Self { uid, rev } = self;
        EntityHeader {
            uid: uid.into_untyped(),
            rev,
        }
    }

    #[must_use]
    pub fn initial_random() -> Self {
        Self::from_untyped(EntityHeader::initial_random())
    }

    #[must_use]
    pub fn initial_with_uid<U: Into<EntityUidTyped<T>>>(uid: U) -> Self {
        Self::from_untyped(EntityHeader::initial_with_uid(uid.into()))
    }

    #[must_use]
    pub fn next_rev(self) -> Option<Self> {
        self.into_untyped().next_rev().map(Self::from_untyped)
    }

    #[must_use]
    pub fn prev_rev(self) -> Option<Self> {
        self.into_untyped().prev_rev().map(Self::from_untyped)
    }
}

impl<T> From<EntityHeaderTyped<T>> for EntityHeader {
    fn from(from: EntityHeaderTyped<T>) -> EntityHeader {
        from.into_untyped()
    }
}

impl<T> Validate for EntityHeaderTyped<T> {
    type Invalidity = EntityHeaderInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&*self.uid, Self::Invalidity::Uid)
            .validate_with(&self.rev, Self::Invalidity::Revision)
            .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Entity
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RawEntity<T: 'static, B> {
    pub hdr: EntityHeaderTyped<T>,
    pub body: B,
}

impl<T, B> RawEntity<T, B> {
    #[must_use]
    pub fn new(hdr: impl Into<EntityHeaderTyped<T>>, body: impl Into<B>) -> Self {
        Self {
            hdr: hdr.into(),
            body: body.into(),
        }
    }

    pub fn try_new<TryIntoB>(
        hdr: impl Into<EntityHeaderTyped<T>>,
        body: TryIntoB,
    ) -> Result<Self, TryIntoB::Error>
    where
        TryIntoB: TryInto<B>,
    {
        Ok(Self {
            hdr: hdr.into(),
            body: body.try_into()?,
        })
    }
}

impl<T, B> From<RawEntity<T, B>> for (EntityHeaderTyped<T>, B) {
    fn from(from: RawEntity<T, B>) -> Self {
        let RawEntity { hdr, body } = from;
        (hdr, body)
    }
}

impl<'a, T, B> From<&'a RawEntity<T, B>> for (&'a EntityHeaderTyped<T>, &'a B) {
    fn from(from: &'a RawEntity<T, B>) -> Self {
        let RawEntity { hdr, body } = from;
        (hdr, body)
    }
}

impl<T, B> IsCanonical for RawEntity<T, B>
where
    B: IsCanonical,
{
    fn is_canonical(&self) -> bool {
        self.body.is_canonical()
    }
}

impl<T, B> Canonicalize for RawEntity<T, B>
where
    B: Canonicalize,
{
    fn canonicalize(&mut self) {
        self.body.canonicalize();
    }
}

pub struct Entity<T: 'static, B, I: 'static> {
    pub raw: RawEntity<T, B>,
    // https://doc.rust-lang.org/std/marker/struct.PhantomData.html#ownership-and-the-drop-check
    invalidity_marker: PhantomData<&'static I>,
}

impl<T, B, I> Entity<T, B, I> {
    #[must_use]
    pub fn new(hdr: impl Into<EntityHeaderTyped<T>>, body: impl Into<B>) -> Self {
        Self {
            raw: RawEntity::new(hdr, body),
            invalidity_marker: PhantomData,
        }
    }

    pub fn try_new<TryIntoB>(
        hdr: impl Into<EntityHeaderTyped<T>>,
        body: TryIntoB,
    ) -> Result<Self, TryIntoB::Error>
    where
        TryIntoB: TryInto<B>,
    {
        Ok(Self {
            raw: RawEntity::try_new(hdr, body)?,
            invalidity_marker: PhantomData,
        })
    }
}

impl<T, B, I> From<Entity<T, B, I>> for (EntityHeaderTyped<T>, B) {
    fn from(from: Entity<T, B, I>) -> Self {
        let Entity {
            raw,
            invalidity_marker: _,
        } = from;
        raw.into()
    }
}

impl<'a, T, B, I> From<&'a Entity<T, B, I>> for (&'a EntityHeaderTyped<T>, &'a B) {
    fn from(from: &'a Entity<T, B, I>) -> Self {
        from.deref().into()
    }
}

impl<T, B, I> Deref for Entity<T, B, I> {
    type Target = RawEntity<T, B>;

    fn deref(&self) -> &Self::Target {
        let Self {
            raw,
            invalidity_marker: _,
        } = self;
        raw
    }
}

impl<T, B, I> DerefMut for Entity<T, B, I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let Self {
            raw,
            invalidity_marker: _,
        } = self;
        raw
    }
}

impl<T, B, I> Default for Entity<T, B, I>
where
    T: Default,
    B: Default,
{
    fn default() -> Self {
        Self {
            raw: Default::default(),
            invalidity_marker: PhantomData,
        }
    }
}

impl<T, B, I> fmt::Debug for Entity<T, B, I>
where
    T: fmt::Debug,
    B: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl<T, B, I> Clone for Entity<T, B, I>
where
    T: Clone,
    B: Clone,
{
    fn clone(&self) -> Self {
        Self {
            raw: self.deref().clone(),
            invalidity_marker: PhantomData,
        }
    }
}

impl<T, B, I> PartialEq for Entity<T, B, I>
where
    T: PartialEq,
    B: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.deref().eq(&*other)
    }
}

impl<T, B, I> Eq for Entity<T, B, I>
where
    T: Eq,
    B: Eq,
{
}

impl<T, B, I> IsCanonical for Entity<T, B, I>
where
    B: IsCanonical,
{
    fn is_canonical(&self) -> bool {
        self.deref().is_canonical()
    }
}

impl<T, B, I> Canonicalize for Entity<T, B, I>
where
    B: Canonicalize,
{
    fn canonicalize(&mut self) {
        self.deref_mut().canonicalize();
    }
}

#[derive(Debug, Clone)]
pub enum EntityInvalidity<I: Invalidity> {
    Header(EntityHeaderInvalidity),
    Body(I),
}

impl<T, B, I> Validate for Entity<T, B, I>
where
    I: Invalidity,
    B: Validate<Invalidity = I>,
{
    type Invalidity = EntityInvalidity<I>;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.hdr, Self::Invalidity::Header)
            .validate_with(&self.body, Self::Invalidity::Body)
            .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
