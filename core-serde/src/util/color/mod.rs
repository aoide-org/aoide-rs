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

mod _core {
    pub use aoide_core::util::color::*;
}

use serde::{
    de::{self, Visitor as SerdeDeserializeVisitor},
    Deserializer, Serializer,
};
use std::{fmt, str::FromStr};

///////////////////////////////////////////////////////////////////////
// ColorRgb
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct ColorRgb(_core::ColorRgb);

impl From<_core::ColorRgb> for ColorRgb {
    fn from(from: _core::ColorRgb) -> Self {
        Self(from)
    }
}

impl From<ColorRgb> for _core::ColorRgb {
    fn from(from: ColorRgb) -> Self {
        from.0
    }
}

impl Serialize for ColorRgb {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

struct ColorDeserializeVisitor;

impl<'de> SerdeDeserializeVisitor<'de> for ColorDeserializeVisitor {
    type Value = ColorRgb;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a color code string '#AARRGGBB'")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        _core::ColorRgb::from_str(value)
            .map(Into::into)
            .map_err(|e| E::custom(e.to_string()))
    }
}

impl<'de> Deserialize<'de> for ColorRgb {
    fn deserialize<D>(deserializer: D) -> Result<ColorRgb, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ColorDeserializeVisitor)
    }
}
