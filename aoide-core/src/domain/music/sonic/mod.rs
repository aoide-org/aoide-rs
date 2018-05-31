// Aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom>
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

#[cfg(test)]
mod tests;

use std::f64;
use std::fmt;

///////////////////////////////////////////////////////////////////////
/// Loudness
///////////////////////////////////////////////////////////////////////

pub type Decibel = f64;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LUFS {
    pub db: Decibel,
}

impl LUFS {
    pub const UNIT_OF_MEASURE: &'static str = "dB";
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum Loudness {
    EBUR128LUFS(LUFS),
}

impl Loudness {
    pub fn is_valid(&self) -> bool {
        true
    }
}

impl fmt::Display for Loudness {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Loudness::EBUR128LUFS(lufs) => write!(f, "{} {}", lufs.db, LUFS::UNIT_OF_MEASURE),
        }
    }
}

///////////////////////////////////////////////////////////////////////
/// Tempo
///////////////////////////////////////////////////////////////////////

pub type BeatsPerMinute = f64;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Tempo {
    pub bpm: BeatsPerMinute,
}

impl Tempo {
    pub const UNIT_OF_MEASURE: &'static str = "bpm";

    pub const MIN: Self = Self {
        bpm: f64::MIN_POSITIVE,
    };
    pub const MAX: Self = Self { bpm: f64::MAX };

    pub fn bpm(bpm: BeatsPerMinute) -> Self {
        Self { bpm }
    }

    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    pub fn is_valid(&self) -> bool {
        *self > Self::default()
    }
}

impl fmt::Display for Tempo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.bpm, Tempo::UNIT_OF_MEASURE)
    }
}

///////////////////////////////////////////////////////////////////////
/// KeySignature
///////////////////////////////////////////////////////////////////////

pub type KeyCode = u8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum KeyMode {
    #[serde(rename = "maj")]
    Major,

    #[serde(rename = "min")]
    Minor,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct KeySignature {
    // 0=unknown/invalid, 1=C, 2=a, 3=G, 4=e, ..., 23=F, 24=d
    pub code: KeyCode,
}

impl KeySignature {
    pub const MIN_CODE: KeyCode = 1;
    pub const MAX_CODE: KeyCode = 24;

    pub fn is_valid_code(code: KeyCode) -> bool {
        code >= KeySignature::MIN_CODE && code <= KeySignature::MAX_CODE
    }

    pub fn new(code: KeyCode) -> Self {
        debug_assert!(Self::is_valid_code(code));
        Self { code }
    }

    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    pub fn is_valid(&self) -> bool {
        Self::is_valid_code(self.code)
    }

    pub fn mode(&self) -> KeyMode {
        match self.code % 2 {
            0 => KeyMode::Minor,
            1 => KeyMode::Major,
            _ => unreachable!(),
        }
    }

    pub fn open_key(&self) -> (KeyCode, KeyMode) {
        (1 + (self.code - 1) / 2, self.mode())
    }
}

impl fmt::Display for KeySignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.code)
    }
}

///////////////////////////////////////////////////////////////////////
/// OpenKeySignature
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct OpenKeySignature {
    #[serde(rename = "keysig")]
    key_signature: KeySignature,
}

impl OpenKeySignature {
    pub const MIN_CODE: KeyCode = 1;
    pub const MAX_CODE: KeyCode = 12;

    pub fn is_valid_code(code: KeyCode) -> bool {
        code >= KeySignature::MIN_CODE && code <= KeySignature::MAX_CODE
    }

    pub fn new(code: KeyCode, mode: KeyMode) -> Self {
        debug_assert!(Self::is_valid_code(code));
        let key_signature = KeySignature {
            code: 2 * code - match mode {
                KeyMode::Major => 1,
                KeyMode::Minor => 0,
            },
        };
        Self { key_signature }
    }

    pub fn is_valid(&self) -> bool {
        self.key_signature.is_valid()
    }

    pub fn code(&self) -> KeyCode {
        1 + (self.key_signature.code - 1) / 2
    }

    pub fn mode(&self) -> KeyMode {
        self.key_signature.mode()
    }
}

impl From<KeySignature> for OpenKeySignature {
    fn from(key_signature: KeySignature) -> Self {
        Self { key_signature }
    }
}

impl From<OpenKeySignature> for KeySignature {
    fn from(from: OpenKeySignature) -> Self {
        from.key_signature
    }
}

impl fmt::Display for OpenKeySignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.code(),
            match self.mode() {
                KeyMode::Major => 'd',
                KeyMode::Minor => 'm',
            }
        )
    }
}

///////////////////////////////////////////////////////////////////////
/// LancelotKeySignature
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LancelotKeySignature {
    #[serde(rename = "keysig")]
    key_signature: KeySignature,
}

impl LancelotKeySignature {
    pub const MIN_CODE: KeyCode = 1;
    pub const MAX_CODE: KeyCode = 12;

    pub fn is_valid_code(code: KeyCode) -> bool {
        code >= KeySignature::MIN_CODE && code <= KeySignature::MAX_CODE
    }

    pub fn new(code: KeyCode, mode: KeyMode) -> Self {
        debug_assert!(Self::is_valid_code(code));
        let key_signature = KeySignature {
            code: ((code * 2 + 9) % 24) + match mode {
                KeyMode::Major => 0,
                KeyMode::Minor => 1,
            },
        };
        Self { key_signature }
    }

    pub fn is_valid(&self) -> bool {
        self.key_signature.is_valid()
    }

    pub fn code(&self) -> KeyCode {
        1 + ((self.key_signature.code + 13) / 2) % 12
    }

    pub fn mode(&self) -> KeyMode {
        self.key_signature.mode()
    }
}

impl From<KeySignature> for LancelotKeySignature {
    fn from(key_signature: KeySignature) -> Self {
        Self { key_signature }
    }
}

impl From<LancelotKeySignature> for KeySignature {
    fn from(from: LancelotKeySignature) -> Self {
        from.key_signature
    }
}

impl fmt::Display for LancelotKeySignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.code(),
            match self.mode() {
                KeyMode::Major => 'B',
                KeyMode::Minor => 'A',
            }
        )
    }
}

///////////////////////////////////////////////////////////////////////
/// EngineKeySignature (as found in Denon Engine Prime Library)
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EngineKeySignature {
    #[serde(rename = "keysig")]
    key_signature: KeySignature,
}

impl EngineKeySignature {
    pub const MIN_CODE: KeyCode = 1;
    pub const MAX_CODE: KeyCode = 24;

    pub fn is_valid_code(code: KeyCode) -> bool {
        code >= KeySignature::MIN_CODE && code <= KeySignature::MAX_CODE
    }

    pub fn new(code: KeyCode) -> Self {
        debug_assert!(Self::is_valid_code(code));
        let key_signature = KeySignature {
            code: code % 24 + 1,
        };
        Self { key_signature }
    }

    pub fn is_valid(&self) -> bool {
        self.key_signature.is_valid()
    }

    pub fn code(&self) -> KeyCode {
        match self.key_signature.code {
            1 => 24,
            code => code - 1,
        }
    }
}

impl From<KeySignature> for EngineKeySignature {
    fn from(key_signature: KeySignature) -> Self {
        Self { key_signature }
    }
}

impl From<EngineKeySignature> for KeySignature {
    fn from(from: EngineKeySignature) -> Self {
        from.key_signature
    }
}

///////////////////////////////////////////////////////////////////////
/// TimeSignature
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TimeSignature {
    #[serde(rename = "num")]
    pub numerator: u8, // number of beats in each bar, 0 = default/undefined

    #[serde(rename = "denom")]
    pub denominator: u8, // symbol length of each beat, 0 = default/undefined
}

impl TimeSignature {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    pub fn is_valid(&self) -> bool {
        (self.numerator > 0) && (self.denominator > 0)
    }
}

impl fmt::Display for TimeSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}
