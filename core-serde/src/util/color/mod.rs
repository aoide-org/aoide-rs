// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

mod _core {
    pub use aoide_core::util::color::{Color, RgbColor};
}

use aoide_core::util::color::ColorIndex;

use serde::{
    de::{self, Visitor as SerdeDeserializeVisitor},
    Deserializer, Serializer,
};
use std::{fmt, str::FromStr};

///////////////////////////////////////////////////////////////////////
// Color
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
