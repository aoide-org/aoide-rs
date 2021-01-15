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

use std::fmt;

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive as _, ToPrimitive as _};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, FromPrimitive, ToPrimitive)]
pub enum KeyCode {
    Unknown = 0,

    /// C major
    Cmaj = 1,

    /// A minor
    Amin = 2,

    /// G major
    Gmaj = 3,

    /// E minor
    Emin = 4,

    /// D major
    Dmaj = 5,

    /// B minor
    Bmin = 6,

    /// A major
    Amaj = 7,

    /// F♯/G♭ minor
    Gbmin = 8,

    /// E major
    Emaj = 9,

    /// D♭ minor
    Dbmin = 10,

    /// B major
    Bmaj = 11,

    /// A♭ minor
    Abmin = 12,

    /// F♯/G♭ major
    Gbmaj = 13,

    /// E♭ minor
    Ebmin = 14,

    /// D♭ major
    Dbmaj = 15,

    /// B♭ minor
    Bbmin = 16,

    /// A♭ major
    Abmaj = 17,

    /// F minor
    Fmin = 18,

    /// E♭ major
    Ebmaj = 19,

    /// C minor
    Cmin = 20,

    /// B♭ major
    Bbmaj = 21,

    /// G minor
    Gmin = 22,

    /// F major
    Fmaj = 23,

    /// D minor
    Dmin = 24,
}

impl Default for KeyCode {
    fn default() -> Self {
        Self::Unknown
    }
}

impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use KeyCode::*;
        let as_str = match self {
            Unknown => "",
            Cmaj => "C",
            Amin => "a",
            Gmaj => "G",
            Emin => "e",
            Dmaj => "D",
            Bmin => "b",
            Amaj => "A",
            Gbmin => "g♭/f♯",
            Emaj => "E",
            Dbmin => "d♭/c♯",
            Bmaj => "B",
            Abmin => "a♭/g♯",
            Gbmaj => "G♭/F♯",
            Ebmin => "e♭/d♯m",
            Dbmaj => "D♭/C♯",
            Bbmin => "b♭",
            Abmaj => "A♭/G♯",
            Fmin => "f",
            Ebmaj => "E♭/D♯m",
            Cmin => "c",
            Bbmaj => "B♭",
            Gmin => "g",
            Fmaj => "F",
            Dmin => "d",
        };
        f.write_str(as_str)
    }
}

pub type KeyCodeValue = u8;

impl KeyCode {
    pub fn to_value(self) -> KeyCodeValue {
        self.to_u8().expect("key code")
    }

    pub fn from_value(val: KeyCodeValue) -> Self {
        Self::from_u8(val).unwrap_or(Self::Unknown)
    }
}

impl From<KeyCodeValue> for KeyCode {
    fn from(from: KeyCodeValue) -> Self {
        Self::from_value(from)
    }
}

impl From<KeyCode> for KeyCodeValue {
    fn from(from: KeyCode) -> Self {
        from.to_value()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum KeyMode {
    Major,
    Minor,
}

/// The ordering numbering of the key code follows the
/// Circle of fifth / Open Key notation in clock-wise orientation,
/// alternating between major and minor keys.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct KeySignature(KeyCode);

impl KeySignature {
    pub const fn unknown() -> Self {
        Self(KeyCode::Unknown)
    }

    pub fn is_unknown(self) -> bool {
        self == Self::unknown()
    }

    pub const fn new(code: KeyCode) -> Self {
        Self(code)
    }

    pub fn code(self) -> KeyCode {
        let Self(code) = self;
        code
    }

    pub fn mode(self) -> Option<KeyMode> {
        match self.code() {
            KeyCode::Unknown => None,
            code => match code.to_value() % 2 {
                0 => Some(KeyMode::Minor),
                1 => Some(KeyMode::Major),
                _ => unreachable!(),
            },
        }
    }
}

impl fmt::Display for KeySignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.code().fmt(f)
    }
}

///////////////////////////////////////////////////////////////////////
// OpenKeySignature
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OpenKeySignature(KeySignature);

impl OpenKeySignature {
    pub const fn min_code() -> KeyCodeValue {
        1
    }
    pub const fn max_code() -> KeyCodeValue {
        12
    }

    pub fn new(code: KeyCodeValue, mode: KeyMode) -> Self {
        let code = KeyCode::from_value(
            2 * code
                - match mode {
                    KeyMode::Major => 1,
                    KeyMode::Minor => 0,
                },
        );
        Self(KeySignature::new(code))
    }

    pub fn code(self) -> KeyCodeValue {
        1 + (self.0.code().to_value() - 1) / 2
    }

    pub fn mode(self) -> Option<KeyMode> {
        self.0.mode()
    }
}

impl From<KeySignature> for OpenKeySignature {
    fn from(key_sig: KeySignature) -> OpenKeySignature {
        OpenKeySignature(key_sig)
    }
}

impl From<OpenKeySignature> for KeySignature {
    fn from(from: OpenKeySignature) -> Self {
        from.0
    }
}

impl fmt::Display for OpenKeySignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(mode) = self.mode() {
            write!(
                f,
                "{}{}",
                self.code(),
                match mode {
                    KeyMode::Major => 'd',
                    KeyMode::Minor => 'm',
                }
            )
        } else {
            // Undefined
            f.write_str("")
        }
    }
}

///////////////////////////////////////////////////////////////////////
// LancelotKeySignature
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LancelotKeySignature(KeySignature);

impl LancelotKeySignature {
    pub const fn min_code() -> KeyCodeValue {
        1
    }
    pub const fn max_code() -> KeyCodeValue {
        12
    }

    pub fn new(code: KeyCodeValue, mode: KeyMode) -> Self {
        let code = KeyCode::from_value(
            ((code * 2 + 9) % 24)
                + match mode {
                    KeyMode::Major => 0,
                    KeyMode::Minor => 1,
                },
        );
        Self(KeySignature::new(code))
    }

    pub fn code(self) -> KeyCodeValue {
        1 + ((self.0.code().to_value() + 13) / 2) % 12
    }

    pub fn mode(self) -> Option<KeyMode> {
        self.0.mode()
    }
}

impl From<KeySignature> for LancelotKeySignature {
    fn from(key_sig: KeySignature) -> Self {
        LancelotKeySignature(key_sig)
    }
}

impl From<LancelotKeySignature> for KeySignature {
    fn from(from: LancelotKeySignature) -> Self {
        from.0
    }
}

impl fmt::Display for LancelotKeySignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(mode) = self.mode() {
            write!(
                f,
                "{}{}",
                self.code(),
                match mode {
                    KeyMode::Major => 'B',
                    KeyMode::Minor => 'A',
                }
            )
        } else {
            // Undefined
            f.write_str("")
        }
    }
}

///////////////////////////////////////////////////////////////////////
// EngineKeySignature (as found in Denon Engine Prime Library)
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EngineKeySignature(KeySignature);

impl EngineKeySignature {
    pub const fn min_code() -> KeyCodeValue {
        1
    }
    pub const fn max_code() -> KeyCodeValue {
        24
    }

    pub fn from_code(code: KeyCodeValue) -> Self {
        let code = KeyCode::from_value(code % 24 + 1);
        Self(KeySignature::new(code))
    }

    pub fn code(self) -> KeyCodeValue {
        match self.0.code().to_value() {
            1 => 24,
            code => code - 1,
        }
    }

    pub fn mode(self) -> Option<KeyMode> {
        self.0.mode()
    }
}

impl From<KeySignature> for EngineKeySignature {
    fn from(key_sig: KeySignature) -> Self {
        EngineKeySignature(key_sig)
    }
}

impl From<EngineKeySignature> for KeySignature {
    fn from(from: EngineKeySignature) -> Self {
        from.0
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
