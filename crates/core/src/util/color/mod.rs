// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::prelude::*;

use std::{fmt, num::ParseIntError, str::FromStr};

///////////////////////////////////////////////////////////////////////
// Color
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    Rgb(RgbColor),
    Index(ColorIndex),
}

#[derive(Copy, Clone, Debug)]
pub enum ColorInvalidity {
    Rgb(RgbColorInvalidity),
}

impl Validate for Color {
    type Invalidity = ColorInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let context = ValidationContext::new();
        match self {
            Color::Rgb(rgb_color) => context.validate_with(rgb_color, Self::Invalidity::Rgb),
            Color::Index(_) => context,
        }
        .into()
    }
}

///////////////////////////////////////////////////////////////////////
// ColorIndex
///////////////////////////////////////////////////////////////////////

pub type ColorIndex = i16;

///////////////////////////////////////////////////////////////////////
// RgbColor
///////////////////////////////////////////////////////////////////////

pub type RgbColorCode = u32;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

    #[must_use]
    pub const fn code(self) -> RgbColorCode {
        self.0
    }

    #[must_use]
    pub const fn min_code() -> RgbColorCode {
        0x00_00_00
    }

    #[must_use]
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
        u32::from_str_radix(hex_code, 16)
            .map(RgbColor)
            .map_err(ParseError::ParseIntError)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum RgbColorInvalidity {
    CodeOutOfRange,
}

impl Validate for RgbColor {
    type Invalidity = RgbColorInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                self.code() < Self::min_code() || self.code() > Self::max_code(),
                Self::Invalidity::CodeOutOfRange,
            )
            .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
