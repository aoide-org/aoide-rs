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

use failure;
use serde::{
    de::{self, Visitor as SerdeDeserializeVisitor},
    Deserializer, Serializer,
};
use std::{fmt, str::FromStr};

///////////////////////////////////////////////////////////////////////
/// ColorArgb
///////////////////////////////////////////////////////////////////////

pub type ColorCode = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ColorArgb(ColorCode); // 0xAARRGGBB

impl ColorArgb {
    const STRING_PREFIX: &'static str = "#";
    const STRING_LEN: usize = 9;

    pub const ALPHA_MASK: ColorCode = 0xff_00_00_00;
    pub const RED_MASK: ColorCode = 0x00_ff_00_00;
    pub const GREEN_MASK: ColorCode = 0x00_00_ff_00;
    pub const BLUE_MASK: ColorCode = 0x00_00_00_ff;

    pub const BLACK: Self = ColorArgb(Self::ALPHA_MASK);
    pub const RED: Self = ColorArgb(Self::ALPHA_MASK | Self::RED_MASK);
    pub const GREEN: Self = ColorArgb(Self::ALPHA_MASK | Self::GREEN_MASK);
    pub const BLUE: Self = ColorArgb(Self::ALPHA_MASK | Self::BLUE_MASK);
    pub const YELLOW: Self = ColorArgb(Self::ALPHA_MASK | Self::RED_MASK | Self::GREEN_MASK);
    pub const MAGENTA: Self = ColorArgb(Self::ALPHA_MASK | Self::RED_MASK | Self::BLUE_MASK);
    pub const CYAN: Self = ColorArgb(Self::ALPHA_MASK | Self::GREEN_MASK | Self::BLUE_MASK);
    pub const WHITE: Self =
        ColorArgb(Self::ALPHA_MASK | Self::RED_MASK | Self::GREEN_MASK | Self::BLUE_MASK);

    pub fn code(self) -> ColorCode {
        self.0
    }

    pub fn to_opaque(self) -> Self {
        ColorArgb(self.code() | Self::ALPHA_MASK)
    }

    pub fn to_transparent(self) -> Self {
        ColorArgb(self.code() & !Self::ALPHA_MASK)
    }
}

impl IsValid for ColorArgb {
    fn is_valid(&self) -> bool {
        true
    }
}

impl fmt::Display for ColorArgb {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{:08X}", Self::STRING_PREFIX, self.code())
    }
}

impl FromStr for ColorArgb {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == Self::STRING_LEN {
            let (prefix, hex_code) = s.split_at(1);
            if prefix == Self::STRING_PREFIX {
                return u32::from_str_radix(&hex_code, 16)
                    .map(ColorArgb)
                    .map_err(Into::into);
            }
        }
        Err(failure::format_err!("Invalid color code '{}'", s))
    }
}

impl Serialize for ColorArgb {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, Copy)]
struct ColorDeserializeVisitor;

impl<'de> SerdeDeserializeVisitor<'de> for ColorDeserializeVisitor {
    type Value = ColorArgb;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a color code string '#AARRGGBB'")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        ColorArgb::from_str(value).map_err(|e| E::custom(e.to_string()))
    }
}

impl<'de> Deserialize<'de> for ColorArgb {
    fn deserialize<D>(deserializer: D) -> Result<ColorArgb, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ColorDeserializeVisitor)
    }
}
