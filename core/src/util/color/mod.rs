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
// Color
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    Rgb(RgbColor),
    Index(ColorIndex),
}

///////////////////////////////////////////////////////////////////////
// ColorIndex
///////////////////////////////////////////////////////////////////////

pub type ColorIndex = i16;

///////////////////////////////////////////////////////////////////////
// RgbColor
///////////////////////////////////////////////////////////////////////

pub type RgbColorCode = u32;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RgbColor(pub RgbColorCode); // 0xRRGGBB

impl RgbColor {
    const STRING_PREFIX: &'static str = "#";
    const STRING_LEN: usize = 1 + 2 + 2 + 2;

    pub const RED_MASK: RgbColorCode = 0xff_00_00;
    pub const GREEN_MASK: RgbColorCode = 0x00_ff_00;
    pub const BLUE_MASK: RgbColorCode = 0x00_00_ff;

    pub const BLACK: Self = RgbColor(0);
    pub const RED: Self = RgbColor(Self::RED_MASK);
    pub const GREEN: Self = RgbColor(Self::GREEN_MASK);
    pub const BLUE: Self = RgbColor(Self::BLUE_MASK);
    pub const YELLOW: Self = RgbColor(Self::RED_MASK | Self::GREEN_MASK);
    pub const MAGENTA: Self = RgbColor(Self::RED_MASK | Self::BLUE_MASK);
    pub const CYAN: Self = RgbColor(Self::GREEN_MASK | Self::BLUE_MASK);
    pub const WHITE: Self = RgbColor(Self::RED_MASK | Self::GREEN_MASK | Self::BLUE_MASK);

    pub const fn code(self) -> RgbColorCode {
        self.0
    }

    pub const fn min_code() -> RgbColorCode {
        0x00_00_00
    }

    pub const fn max_code() -> RgbColorCode {
        0xff_ff_ff
    }
}

impl fmt::Display for RgbColor {
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
                RgbColor::STRING_LEN
            ),
            InputPrefix => write!(
                f,
                "Invalid input prefix: expected = {}",
                RgbColor::STRING_PREFIX
            ),
            ParseIntError(err) => f.write_str(&err.to_string()),
        }
    }
}

impl FromStr for RgbColor {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != Self::STRING_LEN {
            return Err(ParseError::InputLen);
        }
        let (prefix, hex_code) = s.split_at(Self::STRING_PREFIX.len());
        if prefix != Self::STRING_PREFIX {
            return Err(ParseError::InputPrefix);
        }
        u32::from_str_radix(&hex_code, 16)
            .map(RgbColor)
            .map_err(ParseError::ParseIntError)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
