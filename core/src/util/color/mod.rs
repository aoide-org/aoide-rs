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

use std::{fmt, num::ParseIntError, str::FromStr};

///////////////////////////////////////////////////////////////////////
// ColorRgb
///////////////////////////////////////////////////////////////////////

pub type ColorCode = u32;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ColorRgb(pub ColorCode); // 0xRRGGBB

impl ColorRgb {
    const STRING_PREFIX: &'static str = "#";
    const STRING_LEN: usize = 7;

    pub const RED_MASK: ColorCode = 0x00_ff_00_00;
    pub const GREEN_MASK: ColorCode = 0x00_00_ff_00;
    pub const BLUE_MASK: ColorCode = 0x00_00_00_ff;

    pub const BLACK: Self = ColorRgb(0);
    pub const RED: Self = ColorRgb(Self::RED_MASK);
    pub const GREEN: Self = ColorRgb(Self::GREEN_MASK);
    pub const BLUE: Self = ColorRgb(Self::BLUE_MASK);
    pub const YELLOW: Self = ColorRgb(Self::RED_MASK | Self::GREEN_MASK);
    pub const MAGENTA: Self = ColorRgb(Self::RED_MASK | Self::BLUE_MASK);
    pub const CYAN: Self = ColorRgb(Self::GREEN_MASK | Self::BLUE_MASK);
    pub const WHITE: Self = ColorRgb(Self::RED_MASK | Self::GREEN_MASK | Self::BLUE_MASK);

    pub fn code(self) -> ColorCode {
        self.0
    }
}

impl fmt::Display for ColorRgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // "#RRGGBB"
        write!(f, "{}{:06X}", Self::STRING_PREFIX, self.code())
    }
}

#[derive(Clone, Debug)]
pub enum ParseError {
    InputLen,
    InputPrefix,
    ParseIntError(ParseIntError),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ParseError::*;
        match self {
            InputLen => write!(
                f,
                "Invalid input length: expected = {}",
                ColorRgb::STRING_LEN
            ),
            InputPrefix => write!(
                f,
                "Invalid input prefix: expected = {}",
                ColorRgb::STRING_PREFIX
            ),
            ParseIntError(err) => f.write_str(&err.to_string()),
        }
    }
}

impl FromStr for ColorRgb {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != Self::STRING_LEN {
            return Err(ParseError::InputLen);
        }
        let (prefix, hex_code) = s.split_at(1);
        if prefix != Self::STRING_PREFIX {
            return Err(ParseError::InputPrefix);
        }
        u32::from_str_radix(&hex_code, 16)
            .map(ColorRgb)
            .map_err(ParseError::ParseIntError)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
