// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::prelude::*;

mod _core {
    pub(super) use aoide_core::util::color::{Color, RgbColor};
}

use std::{fmt, str::FromStr};

use aoide_core::util::color::ColorIndex;
use serde::{
    Deserializer, Serializer,
    de::{self, Visitor as SerdeDeserializeVisitor},
};

///////////////////////////////////////////////////////////////////////
// Color
///////////////////////////////////////////////////////////////////////

/// Either a color code or a color index.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub enum Color {
    #[serde(rename = "rgb")]
    Rgb(RgbColor),

    #[serde(rename = "idx")]
    Index(ColorIndex),
}

impl From<_core::Color> for Color {
    fn from(from: _core::Color) -> Self {
        use _core::Color as From;
        match from {
            From::Rgb(rgb) => Self::Rgb(rgb.into()),
            From::Index(idx) => Self::Index(idx),
        }
    }
}

impl From<Color> for _core::Color {
    fn from(from: Color) -> Self {
        use Color as From;
        match from {
            From::Rgb(rgb) => Self::Rgb(rgb.into()),
            From::Index(idx) => Self::Index(idx),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// RgbColor
///////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct RgbColor(_core::RgbColor);

#[cfg(feature = "json-schema")]
impl schemars::JsonSchema for RgbColor {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("RgbColor")
    }

    fn json_schema(schema_gen: &mut schemars::generate::SchemaGenerator) -> schemars::Schema {
        let mut schema = schema_gen.subschema_for::<String>();
        let schema_object = schema.ensure_object();
        schema_object.insert("title".to_owned(), "RGB color code".into());
        schema_object.insert(
            "description".to_owned(),
            "A hexadecimal RGB color code \"#RRGGBB\" encoded as a string with 8 bits per channel."
                .into(),
        );
        schema_object.insert(
            "examples".to_owned(),
            vec![serde_json::Value::String("#808080".into())].into(),
        );
        schema
    }
}

impl From<_core::RgbColor> for RgbColor {
    fn from(from: _core::RgbColor) -> Self {
        Self(from)
    }
}

impl From<RgbColor> for _core::RgbColor {
    fn from(from: RgbColor) -> Self {
        from.0
    }
}

impl Serialize for RgbColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

struct ColorDeserializeVisitor;

impl SerdeDeserializeVisitor<'_> for ColorDeserializeVisitor {
    type Value = RgbColor;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a color code string '#RRGGBB'")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        _core::RgbColor::from_str(value)
            .map(Into::into)
            .map_err(|e| E::custom(e.to_string()))
    }
}

impl<'de> Deserialize<'de> for RgbColor {
    fn deserialize<D>(deserializer: D) -> Result<RgbColor, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ColorDeserializeVisitor)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
