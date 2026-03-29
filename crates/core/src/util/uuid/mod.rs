// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{fmt, hash::Hash, ops::Deref, str};

use anyhow::bail;
use data_encoding::{BASE32HEX_NOPAD, DecodePartial, Encoding};
use semval::prelude::*;

/// UUID v7 with base32hex string representation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(transparent)
)]
pub struct Uuid {
    #[cfg_attr(feature = "json-schema", schemars(with = "String"))]
    uuid: uuid::Uuid,
}

impl Uuid {
    const ENCODING: &'static Encoding = &BASE32HEX_NOPAD;

    /// UUID encoded as ASCII string with Self::ENCODING.
    pub const STR_LEN: usize = 26;

    pub const NIL: Self = Self {
        uuid: uuid::Uuid::nil(),
    };

    #[must_use]
    pub const fn is_nil(self) -> bool {
        let Self { uuid } = self;
        uuid.is_nil()
    }

    #[must_use]
    pub fn random() -> Self {
        Self {
            uuid: uuid::Uuid::now_v7(),
        }
    }

    fn decode_ascii(input: &[u8]) -> anyhow::Result<Self> {
        const DECODED_LEN: usize = 16;
        debug_assert_eq!(DECODED_LEN, uuid::Uuid::nil().as_bytes().len());
        if input.len() != Self::STR_LEN {
            bail!("invalid input");
        }
        let mut decode_buf = [0; DECODED_LEN];
        match Self::ENCODING.decode_mut(input, &mut decode_buf) {
            Ok(decode_len) => {
                debug_assert!(decode_len <= DECODED_LEN);
                if decode_len < DECODED_LEN {
                    bail!("insufficient input");
                }
            }
            Err(DecodePartial {
                error,
                read,
                written,
            }) => {
                debug_assert!(read <= input.len());
                debug_assert!(written <= decode_buf.len());
                bail!("invalid input: {error:#}");
            }
        }
        let uuid = uuid::Uuid::from_bytes(decode_buf);
        Ok(Self { uuid })
    }

    fn decode_str(input: &str) -> anyhow::Result<Self> {
        Self::decode_ascii(input.as_bytes())
    }

    #[must_use]
    fn encode_str_impl(self, output: &mut [u8; Self::STR_LEN]) -> &str {
        let Self { uuid } = self;
        let uuid_bytes = uuid.as_bytes();
        let encoded_str = Self::ENCODING.encode_mut_str(uuid_bytes, output);
        debug_assert_eq!(encoded_str.len(), Self::STR_LEN);
        encoded_str
    }

    #[must_use]
    pub fn encode_str(self) -> UuidEncodedStr {
        UuidEncodedStr::from(self)
    }
}

impl AsRef<uuid::Uuid> for Uuid {
    fn as_ref(&self) -> &uuid::Uuid {
        &self.uuid
    }
}

impl Deref for Uuid {
    type Target = uuid::Uuid;

    fn deref(&self) -> &uuid::Uuid {
        self.as_ref()
    }
}

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut encode_buf = [0; Self::STR_LEN];
        let encoded_str = self.encode_str_impl(&mut encode_buf);
        debug_assert_eq!(encoded_str, self.encode_str().as_str());
        encoded_str.fmt(f)
    }
}

impl std::str::FromStr for Uuid {
    type Err = anyhow::Error;

    fn from_str(encoded: &str) -> Result<Self, Self::Err> {
        Uuid::decode_str(encoded)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Uuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Uuid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(UuidDeserializeFromStr)
    }
}

#[cfg(feature = "serde")]
struct UuidDeserializeFromStr;

#[cfg(feature = "serde")]
impl serde::de::Visitor<'_> for UuidDeserializeFromStr {
    type Value = Uuid;

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
pub enum UuidInvalidity {
    Nil,
}

impl Validate for Uuid {
    type Invalidity = UuidInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.is_nil(), Self::Invalidity::Nil)
            .into()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct UuidEncodedStr([u8; Uuid::STR_LEN]);

impl UuidEncodedStr {
    pub const NIL: Self = Self([0; Uuid::STR_LEN]);

    #[must_use]
    #[expect(unsafe_code)]
    pub const fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

impl Default for UuidEncodedStr {
    fn default() -> Self {
        Self::NIL
    }
}

impl From<Uuid> for UuidEncodedStr {
    fn from(from: Uuid) -> Self {
        let mut encode_buf = [0u8; Uuid::STR_LEN];
        let encode_len = from.encode_str_impl(&mut encode_buf).len();
        debug_assert_eq!(encode_len, encode_buf.len());
        Self(encode_buf)
    }
}

impl From<UuidEncodedStr> for Uuid {
    fn from(from: UuidEncodedStr) -> Self {
        Uuid::decode_str(from.as_str()).unwrap()
    }
}

impl AsRef<str> for UuidEncodedStr {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for UuidEncodedStr {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_ref()
    }
}

impl fmt::Display for UuidEncodedStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
