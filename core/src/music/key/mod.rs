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
        f.write_str(self.as_traditional_str())
    }
}

impl KeyCode {
    pub fn as_traditional_str(self) -> &'static str {
        use KeyCode::*;
        match self {
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
            Ebmin => "e♭/d♯",
            Dbmaj => "D♭/C♯",
            Bbmin => "b♭",
            Abmaj => "A♭/G♯",
            Fmin => "f",
            Ebmaj => "E♭/D♯",
            Cmin => "c",
            Bbmaj => "B♭",
            Gmin => "g",
            Fmaj => "F",
            Dmin => "d",
        }
    }

    pub fn from_traditional_str(s: &str) -> Self {
        use KeyCode::*;
        match s {
            "C" => Cmaj,
            "a" => Amin,
            "G" => Gmaj,
            "e" => Emin,
            "D" => Dmaj,
            "b" => Bmin,
            "A" => Amaj,
            "g♭" => Gbmin,
            "f♯" => Gbmin,
            "g♭/f♯" => Gbmin,
            "E" => Emaj,
            "d♭" => Dbmin,
            "c♯" => Dbmin,
            "d♭/c♯" => Dbmin,
            "B" => Bmaj,
            "a♭♯" => Abmin,
            "g♯" => Abmin,
            "a♭/g♯" => Abmin,
            "G♭" => Gbmaj,
            "F♯" => Gbmaj,
            "G♭/F♯" => Gbmaj,
            "e♭" => Ebmin,
            "d♯" => Ebmin,
            "e♭/d♯" => Ebmin,
            "D♭" => Dbmaj,
            "C♯" => Dbmaj,
            "D♭/C♯" => Dbmaj,
            "b♭" => Bbmin,
            "A♭" => Abmaj,
            "G♯" => Abmaj,
            "A♭/G♯" => Abmaj,
            "f" => Fmin,
            "E♭" => Ebmaj,
            "D♯" => Ebmaj,
            "E♭/D♯" => Ebmaj,
            "c" => Cmin,
            "B♭" => Bbmaj,
            "g" => Gmin,
            "F" => Fmaj,
            "d" => Dmin,
            _ => Unknown,
        }
    }

    pub fn as_openkey_str(self) -> &'static str {
        use KeyCode::*;
        match self {
            Unknown => "",
            Cmaj => "1d",
            Amin => "1m",
            Gmaj => "2d",
            Emin => "2m",
            Dmaj => "3d",
            Bmin => "3m",
            Amaj => "4d",
            Gbmin => "4m",
            Emaj => "5d",
            Dbmin => "5m",
            Bmaj => "6d",
            Abmin => "6m",
            Gbmaj => "7d",
            Ebmin => "7m",
            Dbmaj => "8d",
            Bbmin => "8m",
            Abmaj => "9d",
            Fmin => "9m",
            Ebmaj => "10d",
            Cmin => "10m",
            Bbmaj => "11d",
            Gmin => "11m",
            Fmaj => "12d",
            Dmin => "12m",
        }
    }

    pub fn from_openkey_str(s: &str) -> Self {
        use KeyCode::*;
        match s {
            "1d" => Cmaj,
            "1m" => Amin,
            "2d" => Gmaj,
            "2m" => Emin,
            "3d" => Dmaj,
            "3m" => Bmin,
            "4d" => Amaj,
            "4m" => Gbmin,
            "5d" => Emaj,
            "5m" => Dbmin,
            "6d" => Bmaj,
            "6m" => Abmin,
            "7d" => Gbmaj,
            "7m" => Ebmin,
            "8d" => Dbmaj,
            "8m" => Bbmin,
            "9d" => Abmaj,
            "9m" => Fmin,
            "10d" => Ebmaj,
            "10m" => Cmin,
            "11d" => Bbmaj,
            "11m" => Gmin,
            "12d" => Fmaj,
            "12m" => Dmin,
            _ => Unknown,
        }
    }

    pub fn as_lancelot_str(self) -> &'static str {
        use KeyCode::*;
        match self {
            Unknown => "",
            Cmaj => "8B",
            Amin => "8A",
            Gmaj => "9B",
            Emin => "9A",
            Dmaj => "10B",
            Bmin => "10A",
            Amaj => "11B",
            Gbmin => "11A",
            Emaj => "12B",
            Dbmin => "12A",
            Bmaj => "1B",
            Abmin => "1A",
            Gbmaj => "2B",
            Ebmin => "2A",
            Dbmaj => "3B",
            Bbmin => "3A",
            Abmaj => "4B",
            Fmin => "4A",
            Ebmaj => "5B",
            Cmin => "5A",
            Bbmaj => "6B",
            Gmin => "6A",
            Fmaj => "7B",
            Dmin => "7A",
        }
    }

    pub fn from_lancelot_str(s: &str) -> Self {
        use KeyCode::*;
        match s {
            "8A" => Cmaj,
            "8B" => Amin,
            "9A" => Gmaj,
            "9B" => Emin,
            "10A" => Dmaj,
            "10B" => Bmin,
            "11A" => Amaj,
            "11B" => Gbmin,
            "12A" => Emaj,
            "12B" => Dbmin,
            "1A" => Bmaj,
            "1B" => Abmin,
            "2A" => Gbmaj,
            "2B" => Ebmin,
            "3A" => Dbmaj,
            "3B" => Bbmin,
            "4A" => Abmaj,
            "4B" => Fmin,
            "5A" => Ebmaj,
            "5B" => Cmin,
            "6A" => Bbmaj,
            "6B" => Gmin,
            "7A" => Fmaj,
            "7B" => Dmin,
            _ => Unknown,
        }
    }

    pub fn as_traxsource_str(self) -> &'static str {
        use KeyCode::*;
        match self {
            Unknown => "",
            Cmaj => "Cmaj",
            Amin => "Amin",
            Gmaj => "Gmaj",
            Emin => "Emin",
            Dmaj => "Dmaj",
            Bmin => "Bmin",
            Amaj => "Amaj",
            Gbmin => "F#min",
            Emaj => "Emaj",
            Dbmin => "C#min",
            Bmaj => "Bmaj",
            Abmin => "G#min",
            Gbmaj => "F#maj",
            Ebmin => "D#min",
            Dbmaj => "C#maj",
            Bbmin => "A#min",
            Abmaj => "G#maj",
            Fmin => "Fmin",
            Ebmaj => "D#maj",
            Cmin => "Cmin",
            Bbmaj => "A#maj",
            Gmin => "Gmin",
            Fmaj => "Fmaj",
            Dmin => "Dmin",
        }
    }

    pub fn from_traxsource_str(s: &str) -> Self {
        use KeyCode::*;
        match s {
            "Cmaj" => Cmaj,
            "Amin" => Amin,
            "Gmaj" => Gmaj,
            "Emin" => Emin,
            "Dmaj" => Dmaj,
            "Bmin" => Bmin,
            "Amaj" => Amaj,
            "F#min" => Gbmin,
            "Emaj" => Emaj,
            "C#min" => Dbmin,
            "Bmaj" => Bmaj,
            "G#min" => Abmin,
            "F#maj" => Gbmaj,
            "D#min" => Ebmin,
            "C#maj" => Dbmaj,
            "A#min" => Bbmin,
            "G#maj" => Abmaj,
            "Fmin" => Fmin,
            "D#maj" => Ebmaj,
            "Cmin" => Cmin,
            "A#maj" => Bbmaj,
            "Gmin" => Gmin,
            "Fmaj" => Fmaj,
            "Dmin" => Dmin,
            _ => Unknown,
        }
    }

    pub fn as_beatport_str(self) -> &'static str {
        use KeyCode::*;
        match self {
            Unknown => "",
            Cmaj => "C maj",
            Amin => "A min",
            Gmaj => "G maj",
            Emin => "E min",
            Dmaj => "D maj",
            Bmin => "B min",
            Amaj => "Amaj",
            Gbmin => "G♭/F♯ min",
            Emaj => "E maj",
            Dbmin => "D♭/C♯ min",
            Bmaj => "B maj",
            Abmin => "A♭/G♯ min",
            Gbmaj => "G♭/F♯ maj",
            Ebmin => "E♭/D♯ min",
            Dbmaj => "D♭/C♯ maj",
            Bbmin => "B♭/A♯ min",
            Abmaj => "A♭/G♯ maj",
            Fmin => "F min",
            Ebmaj => "E♭/D♯ maj",
            Cmin => "C min",
            Bbmaj => "B♭/A♯ maj",
            Gmin => "G min",
            Fmaj => "F maj",
            Dmin => "D min",
        }
    }

    pub fn from_beatport_str(s: &str) -> Self {
        use KeyCode::*;
        match s {
            "C maj" => Cmaj,
            "A min" => Amin,
            "G maj" => Gmaj,
            "E min" => Emin,
            "D maj" => Dmaj,
            "B min" => Bmin,
            "A maj" => Amaj,
            "G♭ min" => Gbmin,
            "F♯ min" => Gbmin,
            "G♭/F♯ min" => Gbmin,
            "E maj" => Emaj,
            "D♭ min" => Dbmin,
            "C♯ min" => Dbmin,
            "D♭/C♯ min" => Dbmin,
            "B maj" => Bmaj,
            "A♭♯ min" => Abmin,
            "G♯ min" => Abmin,
            "A♭/G♯ min" => Abmin,
            "G♭ maj" => Gbmaj,
            "F♯ maj" => Gbmaj,
            "G♭/F♯ maj" => Gbmaj,
            "E♭ min" => Ebmin,
            "D♯ min" => Ebmin,
            "E♭/D♯ min" => Ebmin,
            "D♭ maj" => Dbmaj,
            "C♯ maj" => Dbmaj,
            "D♭/C♯ maj" => Dbmaj,
            "B♭ min" => Bbmin,
            "A♭ maj" => Abmaj,
            "G♯ maj" => Abmaj,
            "A♭/G♯ maj" => Abmaj,
            "F min" => Fmin,
            "E♭ maj" => Ebmaj,
            "D♯ maj" => Ebmaj,
            "E♭/D♯ maj" => Ebmaj,
            "C min" => Cmin,
            "B♭ maj" => Bbmaj,
            "G min" => Gmin,
            "F maj" => Fmaj,
            "D min" => Dmin,
            _ => Unknown,
        }
    }

    pub fn as_serato_str(self) -> &'static str {
        use KeyCode::*;
        match self {
            Unknown => "",
            Cmaj => "C",
            Amin => "Am",
            Gmaj => "G",
            Emin => "Em",
            Dmaj => "D",
            Bmin => "Bm",
            Amaj => "A",
            Gbmin => "F#m",
            Emaj => "E",
            Dbmin => "C#m",
            Bmaj => "B",
            Abmin => "G#m",
            Gbmaj => "F#",
            Ebmin => "Ebm",
            Dbmaj => "C#",
            Bbmin => "Bbm",
            Abmaj => "G#",
            Fmin => "Fm",
            Ebmaj => "Eb",
            Cmin => "Cm",
            Bbmaj => "Bb",
            Gmin => "Gm",
            Fmaj => "F",
            Dmin => "Dm",
        }
    }

    pub fn from_serato_str(s: &str) -> Self {
        use KeyCode::*;
        match s {
            "C" => Cmaj,
            "Am" => Amin,
            "G" => Gmaj,
            "Em" => Emin,
            "D" => Dmaj,
            "Bm" => Bmin,
            "A" => Amaj,
            "F#m" => Gbmin,
            "E" => Emaj,
            "C#m" => Dbmin,
            "B" => Bmaj,
            "G#m" => Abmin,
            "F#" => Gbmaj,
            "Ebm" => Ebmin,
            "C#" => Dbmaj,
            "Bbm" => Bbmin,
            "G#" => Abmaj,
            "Fm" => Fmin,
            "Eb" => Ebmaj,
            "Cm" => Cmin,
            "Bb" => Bbmaj,
            "Gm" => Gmin,
            "F" => Fmaj,
            "Dm" => Dmin,
            _ => Unknown,
        }
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

impl From<KeyCode> for KeySignature {
    fn from(from: KeyCode) -> Self {
        KeySignature::new(from)
    }
}

impl From<KeySignature> for KeyCode {
    fn from(from: KeySignature) -> Self {
        from.code()
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
        f.write_str(self.0.code().as_openkey_str())
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
        f.write_str(self.0.code().as_lancelot_str())
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
