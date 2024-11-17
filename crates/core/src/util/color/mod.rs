// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{fmt, num::ParseIntError, str::FromStr};

use semval::prelude::*;

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
pub struct RgbColor(RgbColorCode); // 0xRRGGBB

impl RgbColor {
    const STRING_PREFIX: &'static str = "#";
    const STRING_LEN: usize = 1 + 2 + 2 + 2;

    pub const RED_MASK: RgbColorCode = 0xff_00_00;
    pub const GREEN_MASK: RgbColorCode = 0x00_ff_00;
    pub const BLUE_MASK: RgbColorCode = 0x00_00_ff;

    pub const MIN_CODE: RgbColorCode = 0x00_00_00;
    pub const MAX_CODE: RgbColorCode = Self::RED_MASK | Self::GREEN_MASK | Self::BLUE_MASK;

    pub const BLACK: Self = RgbColor(0);
    pub const RED: Self = RgbColor(Self::RED_MASK);
    pub const GREEN: Self = RgbColor(Self::GREEN_MASK);
    pub const BLUE: Self = RgbColor(Self::BLUE_MASK);
    pub const YELLOW: Self = RgbColor(Self::RED_MASK | Self::GREEN_MASK);
    pub const MAGENTA: Self = RgbColor(Self::RED_MASK | Self::BLUE_MASK);
    pub const CYAN: Self = RgbColor(Self::GREEN_MASK | Self::BLUE_MASK);
    pub const WHITE: Self = RgbColor(Self::RED_MASK | Self::GREEN_MASK | Self::BLUE_MASK);

    #[must_use]
    pub const fn new(code: RgbColorCode) -> Self {
        Self(code)
    }

    #[must_use]
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self(
            ((red as u32) << Self::RED_MASK.trailing_zeros())
                | ((green as u32) << Self::GREEN_MASK.trailing_zeros())
                | ((blue as u32) << Self::BLUE_MASK.trailing_zeros()),
        )
    }

    #[must_use]
    pub const fn code(self) -> RgbColorCode {
        self.0
    }

    #[must_use]
    pub const fn red(self) -> u8 {
        ((self.0 >> Self::RED_MASK.trailing_zeros()) & 0xff) as u8
    }

    #[must_use]
    pub const fn green(self) -> u8 {
        ((self.0 >> Self::GREEN_MASK.trailing_zeros()) & 0xff) as u8
    }

    #[must_use]
    pub const fn blue(self) -> u8 {
        ((self.0 >> Self::BLUE_MASK.trailing_zeros()) & 0xff) as u8
    }
}

impl fmt::Display for RgbColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // "#RRGGBB"
        write!(
            f,
            "{prefix}{code:06X}",
            prefix = Self::STRING_PREFIX,
            code = self.code()
        )
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
        match self {
            Self::InputLen => write!(
                f,
                "Invalid input length: expected = {expected}",
                expected = RgbColor::STRING_LEN,
            ),
            Self::InputPrefix => write!(
                f,
                "Invalid input prefix: expected = {expected}",
                expected = RgbColor::STRING_PREFIX,
            ),
            Self::ParseIntError(err) => err.fmt(f),
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

    #[allow(clippy::absurd_extreme_comparisons)]
    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                self.code() < Self::MIN_CODE || self.code() > Self::MAX_CODE,
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
