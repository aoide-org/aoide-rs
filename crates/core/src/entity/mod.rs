// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    str,
};

use anyhow::bail;
use data_encoding::{BASE32HEX, DecodePartial, Encoding};
use nonicle::{Canonicalize, IsCanonical};
use semval::prelude::*;
use uuid::Uuid;

///////////////////////////////////////////////////////////////////////
// EntityUid
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(transparent)
)]
pub struct EntityUid {
    #[cfg_attr(feature = "json-schema", schemars(with = "String"))]
    uuid: Uuid,
}

impl EntityUid {
    const ENCODING: &'static Encoding = &BASE32HEX;

    // UUID encoded as string with Self::ENCODING.
    pub const STR_LEN: usize = 26;

    // Encoding requires some extra space for padding.
    const ENCODE_LEN: usize = 32;

    // Decoding requires some extra space to account for padding.
    const DECODE_LEN: usize = 20;

    pub const NIL: Self = Self { uuid: Uuid::nil() };

    #[must_use]
    pub const fn is_nil(&self) -> bool {
        let Self { uuid } = self;
        uuid.is_nil()
    }

    #[must_use]
    pub fn random() -> Self {
        Self {
            uuid: Uuid::now_v7(),
        }
    }

    fn decode_ascii(input: &[u8]) -> anyhow::Result<Self> {
        const DECODED_LEN: usize = 16;
        debug_assert_eq!(DECODED_LEN, Uuid::nil().as_bytes().len());
        if input.len() != Self::STR_LEN {
            bail!("invalid input");
        }
        // Pad input.
        let input: [_; Self::ENCODE_LEN] = std::array::from_fn(|index| {
            if index < Self::STR_LEN {
                // Input character.
                input[index]
            } else {
                // Padding.
                b'='
            }
        });
        let mut decode_buf = [0; Self::DECODE_LEN];
        let decoded = match Self::ENCODING.decode_mut(&input, &mut decode_buf) {
            Ok(decode_len) => {
                debug_assert!(decode_len <= DECODED_LEN);
                if decode_len < DECODED_LEN {
                    bail!("insufficient input");
                }
                decode_buf[..DECODED_LEN].try_into().unwrap()
            }
            Err(DecodePartial {
                error,
                read,
                written,
            }) => {
                debug_assert!(read <= input.len());
                debug_assert!(written <= decode_buf.len());
                if written != DECODED_LEN || read != Self::STR_LEN {
                    bail!("invalid input: {error:#}");
                }
                decode_buf[..DECODED_LEN].try_into().unwrap()
            }
        };
        let uuid = Uuid::from_bytes(decoded);
        Ok(Self { uuid })
    }

    fn decode_str(input: &str) -> anyhow::Result<Self> {
        Self::decode_ascii(input.as_bytes())
    }

    #[expect(clippy::assertions_on_constants)]
    fn encode_str<'a>(&self, output: &'a mut [u8; Self::ENCODE_LEN]) -> &'a str {
        let Self { uuid } = self;
        let uuid_bytes = uuid.as_bytes();
        let encoded_str = Self::ENCODING.encode_mut_str(uuid_bytes, output);
        // The length of the returned string matches that of the encode buffer
        // and needs to be adjusted to the actual length.
        debug_assert_eq!(encoded_str.len(), Self::ENCODE_LEN);
        debug_assert!(Self::STR_LEN <= Self::ENCODE_LEN);
        &encoded_str[..Self::STR_LEN]
    }
}

impl fmt::Display for EntityUid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut encode_buf = [0; Self::ENCODE_LEN];
        let encoded_str = self.encode_str(&mut encode_buf);
        debug_assert_eq!(encoded_str, EncodedEntityUid::from(self).to_string());
        encoded_str.fmt(f)
    }
}

impl std::str::FromStr for EntityUid {
    type Err = anyhow::Error;

    fn from_str(encoded: &str) -> Result<Self, Self::Err> {
        EntityUid::decode_str(encoded)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for EntityUid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for EntityUid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(EntityUidDeserializeFromStr)
    }
}

#[cfg(feature = "serde")]
struct EntityUidDeserializeFromStr;

#[cfg(feature = "serde")]
impl serde::de::Visitor<'_> for EntityUidDeserializeFromStr {
    type Value = EntityUid;

    fn visit_str<E>(self, input: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        input
            .parse()
            .map_err(|_| serde::de::Error::invalid_value(serde::de::Unexpected::Str(input), &self))
    }

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a base32hex encoded UUID v7")
    }
}

#[derive(Copy, Clone, Debug)]
pub enum EntityUidInvalidity {
    Nil,
}

impl Validate for EntityUid {
    type Invalidity = EntityUidInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.is_nil(), Self::Invalidity::Nil)
            .into()
    }
}

#[repr(transparent)]
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

impl<T> AsRef<EntityUid> for EntityUidTyped<T> {
    fn as_ref(&self) -> &EntityUid {
        &self.untyped
    }
}

impl<T> Deref for EntityUidTyped<T> {
    type Target = EntityUid;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
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
    type Err = anyhow::Error;

    fn from_str(encoded: &str) -> Result<Self, Self::Err> {
        EntityUid::from_str(encoded).map(Self::from_untyped)
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
        self.deref().eq(other)
    }
}

impl<T> Eq for EntityUidTyped<T> {}

impl<T> PartialOrd for EntityUidTyped<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for EntityUidTyped<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.deref().cmp(other)
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct EncodedEntityUid([u8; EntityUid::STR_LEN]);

impl EncodedEntityUid {
    #[must_use]
    #[expect(unsafe_code)]
    pub const fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

impl Default for EncodedEntityUid {
    fn default() -> Self {
        Self::from(&EntityUid::NIL)
    }
}

impl From<&EntityUid> for EncodedEntityUid {
    fn from(from: &EntityUid) -> Self {
        let mut encode_buf = [0u8; EntityUid::ENCODE_LEN];
        let encoded_len = from.encode_str(&mut encode_buf).len();
        debug_assert_eq!(encoded_len, EntityUid::STR_LEN);
        Self(encode_buf[..EntityUid::STR_LEN].try_into().unwrap())
    }
}

impl<T: 'static> From<&EntityUidTyped<T>> for EncodedEntityUid {
    fn from(from: &EntityUidTyped<T>) -> Self {
        Self::from(from.as_ref())
    }
}

impl From<&EncodedEntityUid> for EntityUid {
    fn from(from: &EncodedEntityUid) -> Self {
        EntityUid::decode_str(from.as_str()).unwrap()
    }
}

impl<T: 'static> From<&EncodedEntityUid> for EntityUidTyped<T> {
    fn from(from: &EncodedEntityUid) -> Self {
        Self::from_untyped(EntityUid::from(from))
    }
}

impl AsRef<str> for EncodedEntityUid {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for EncodedEntityUid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

///////////////////////////////////////////////////////////////////////
// EntityRevision
///////////////////////////////////////////////////////////////////////

// A 1-based, non-negative, monotonously increasing number
pub type EntityRevisionValue = u64;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(transparent)
)]
pub struct EntityRevision(EntityRevisionValue);

impl EntityRevision {
    #[must_use]
    pub const fn new_unchecked(value: EntityRevisionValue) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> EntityRevisionValue {
        let Self(value) = self;
        value
    }

    #[cfg(test)]
    const RESERVED_DEFAULT: Self = Self(0);

    const INITIAL: Self = Self(1);

    #[must_use]
    pub fn is_initial(self) -> bool {
        self == Self::INITIAL
    }

    #[must_use]
    pub fn prev(self) -> Option<Self> {
        debug_assert!(self.is_valid());
        let Self(value) = self;
        let prev = Self::new_unchecked(value.checked_sub(1)?);
        #[cfg(not(test))] // Allow for testing with invalid revisions
        debug_assert!(prev.is_valid());
        Some(prev)
    }

    #[must_use]
    pub fn next(self) -> Option<Self> {
        debug_assert!(self.is_valid());
        let Self(value) = self;
        let next = Self::new_unchecked(value.checked_add(1)?);
        #[cfg(not(test))] // Allow for testing with invalid revisions
        debug_assert!(next.is_valid());
        Some(next)
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        <Self as IsValid>::is_valid(self)
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
            .invalidate_if(*self < Self::INITIAL, Self::Invalidity::OutOfRange)
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
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
    pub fn initial_with_uid<T: Into<EntityUid>>(uid: T) -> Self {
        Self {
            uid: uid.into(),
            rev: EntityRevision::INITIAL,
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

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
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
        self.deref().eq(other)
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
