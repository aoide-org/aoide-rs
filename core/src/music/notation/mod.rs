// aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use std::{f64, fmt};

///////////////////////////////////////////////////////////////////////
/// Modules
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;

///////////////////////////////////////////////////////////////////////
/// Tempo
///////////////////////////////////////////////////////////////////////

pub type Beats = f64;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct TempoBpm(Beats);

impl TempoBpm {
    pub const UNIT_OF_MEASURE: &'static str = "bpm";

    pub const MIN: Self = TempoBpm(f64::MIN_POSITIVE);
    pub const MAX: Self = TempoBpm(f64::MAX);

    pub fn from_bpm(bpm: Beats) -> Self {
        TempoBpm(bpm)
    }

    pub fn bpm(self) -> Beats {
        self.0
    }

    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    pub fn is_valid(&self) -> bool {
        *self >= Self::MIN && *self <= Self::MAX
    }
}

impl fmt::Display for TempoBpm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.bpm(), TempoBpm::UNIT_OF_MEASURE)
    }
}

///////////////////////////////////////////////////////////////////////
/// KeySignature
///////////////////////////////////////////////////////////////////////

pub type KeyCode = u8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum KeyMode {
    #[serde(rename = "maj")]
    Major,

    #[serde(rename = "min")]
    Minor,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct KeySignature(KeyCode);

impl KeySignature {
    pub const MIN_CODE: KeyCode = 1;
    pub const MAX_CODE: KeyCode = 24;

    pub fn is_valid_code(code: KeyCode) -> bool {
        code >= KeySignature::MIN_CODE && code <= KeySignature::MAX_CODE
    }

    pub fn from_code(code: KeyCode) -> Self {
        debug_assert!(Self::is_valid_code(code));
        KeySignature(code)
    }

    pub fn code(self) -> KeyCode {
        self.0
    }

    pub fn mode(self) -> KeyMode {
        match self.code() % 2 {
            0 => KeyMode::Minor,
            1 => KeyMode::Major,
            _ => unreachable!(),
        }
    }

    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    pub fn is_valid(&self) -> bool {
        Self::is_valid_code(self.code())
    }
}

impl fmt::Display for KeySignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

///////////////////////////////////////////////////////////////////////
/// OpenKeySignature
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct OpenKeySignature(KeySignature);

impl OpenKeySignature {
    pub const MIN_CODE: KeyCode = 1;
    pub const MAX_CODE: KeyCode = 12;

    pub fn is_valid_code(code: KeyCode) -> bool {
        code >= KeySignature::MIN_CODE && code <= KeySignature::MAX_CODE
    }

    pub fn new(code: KeyCode, mode: KeyMode) -> Self {
        debug_assert!(Self::is_valid_code(code));
        let key_sig = KeySignature::from_code(
            2 * code
                - match mode {
                    KeyMode::Major => 1,
                    KeyMode::Minor => 0,
                },
        );
        OpenKeySignature(key_sig)
    }

    pub fn is_valid(&self) -> bool {
        self.0.is_valid()
    }

    pub fn code(self) -> KeyCode {
        1 + (self.0.code() - 1) / 2
    }

    pub fn mode(self) -> KeyMode {
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
pub struct LancelotKeySignature(KeySignature);

impl LancelotKeySignature {
    pub const MIN_CODE: KeyCode = 1;
    pub const MAX_CODE: KeyCode = 12;

    pub fn is_valid_code(code: KeyCode) -> bool {
        code >= KeySignature::MIN_CODE && code <= KeySignature::MAX_CODE
    }

    pub fn new(code: KeyCode, mode: KeyMode) -> Self {
        debug_assert!(Self::is_valid_code(code));
        let key_sig = KeySignature::from_code(
            ((code * 2 + 9) % 24)
                + match mode {
                    KeyMode::Major => 0,
                    KeyMode::Minor => 1,
                },
        );
        LancelotKeySignature(key_sig)
    }

    pub fn is_valid(&self) -> bool {
        self.0.is_valid()
    }

    pub fn code(self) -> KeyCode {
        1 + ((self.0.code() + 13) / 2) % 12
    }

    pub fn mode(self) -> KeyMode {
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
pub struct EngineKeySignature(KeySignature);

impl EngineKeySignature {
    pub const MIN_CODE: KeyCode = 1;
    pub const MAX_CODE: KeyCode = 24;

    pub fn is_valid_code(code: KeyCode) -> bool {
        code >= KeySignature::MIN_CODE && code <= KeySignature::MAX_CODE
    }

    pub fn from_code(code: KeyCode) -> Self {
        debug_assert!(Self::is_valid_code(code));
        EngineKeySignature(KeySignature::from_code(code % 24 + 1))
    }

    pub fn is_valid(&self) -> bool {
        self.0.is_valid()
    }

    pub fn code(self) -> KeyCode {
        match self.0.code() {
            1 => 24,
            code => code - 1,
        }
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
/// TimeSignature
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TimeSignature(u16, u16);

impl TimeSignature {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    pub fn new(top: u16, bottom: u16) -> Self {
        TimeSignature(top, bottom)
    }

    // number of beats in each measure unit or bar, 0 = default/undefined
    pub fn top(self) -> u16 {
        self.0
    }

    pub fn beats_per_measure(self) -> u16 {
        self.top()
    }

    // 0 = default/undefined
    pub fn bottom(self) -> u16 {
        self.1
    }

    pub fn measure_unit(self) -> u16 {
        self.bottom()
    }

    pub fn is_valid(&self) -> bool {
        (self.top() > 0) && (self.bottom() > 0)
    }
}

impl fmt::Display for TimeSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.top(), self.bottom())
    }
}
