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

use crate::prelude::*;

mod _core {
    pub use aoide_core::util::color::{Color, RgbColor};
}

use aoide_core::util::color::ColorIndex;

use schemars::{gen::SchemaGenerator, schema::Schema};
use serde::{
    de::{self, Visitor as SerdeDeserializeVisitor},
    Deserializer, Serializer,
};
use std::{fmt, str::FromStr};

///////////////////////////////////////////////////////////////////////
// Color
///////////////////////////////////////////////////////////////////////

/// Either a color code or a color index.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum Color {
    #[serde(rename = "rgb")]
    Rgb(RgbColor),

    #[serde(rename = "idx")]
    Index(ColorIndex),
}

impl From<_core::Color> for Color {
    fn from(from: _core::Color) -> Self {
        use _core::Color::*;
        match from {
            Rgb(rgb) => Color::Rgb(rgb.into()),
            Index(idx) => Color::Index(idx),
        }
    }
}

impl From<Color> for _core::Color {
    fn from(from: Color) -> Self {
        use _core::Color::*;
        match from {
            Color::Rgb(rgb) => Rgb(rgb.into()),
            Color::Index(idx) => Index(idx),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// RgbColor
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RgbColor(_core::RgbColor);

impl JsonSchema for RgbColor {
    fn schema_name() -> String {
        "RgbColor".to_string()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let mut schema = gen.subschema_for::<String>();
        if let Schema::Object(mut schema_object) = schema {
            schema_object.metadata().title = Some("RGB color code".into());
            schema_object.metadata().description = Some("A hexadecimal RGB color code \"#RRGGBB\" encoded as a string with 8 bits per channel.".into());
            schema_object.metadata().examples = vec!["#808080".into()];
            schema = Schema::Object(schema_object)
        }
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

impl<'de> SerdeDeserializeVisitor<'de> for ColorDeserializeVisitor {
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
