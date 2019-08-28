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

use std::fmt;

pub type KeyCode = u8;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyMode {
    Major,
    Minor,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeySignature(KeyCode);

impl KeySignature {
    pub const fn min_code() -> KeyCode {
        1
    }
    pub const fn max_code() -> KeyCode {
        24
    }

    pub fn is_valid_code(code: KeyCode) -> bool {
        code >= KeySignature::min_code() && code <= KeySignature::max_code()
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
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KeySignatureInvalidity {
    Invalid,
}

impl Validate for KeySignature {
    type Invalidity = KeySignatureInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                !Self::is_valid_code(self.code()),
                KeySignatureInvalidity::Invalid,
            )
            .into()
    }
}

impl fmt::Display for KeySignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

///////////////////////////////////////////////////////////////////////
// OpenKeySignature
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct OpenKeySignature(KeySignature);

impl OpenKeySignature {
    pub const fn min_code() -> KeyCode {
        1
    }
    pub const fn max_code() -> KeyCode {
        12
    }

    pub fn is_valid_code(code: KeyCode) -> bool {
        code >= KeySignature::min_code() && code <= KeySignature::max_code()
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
// LancelotKeySignature
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct LancelotKeySignature(KeySignature);

impl LancelotKeySignature {
    pub const fn min_code() -> KeyCode {
        1
    }
    pub const fn max_code() -> KeyCode {
        12
    }

    pub fn is_valid_code(code: KeyCode) -> bool {
        code >= KeySignature::min_code() && code <= KeySignature::max_code()
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
// EngineKeySignature (as found in Denon Engine Prime Library)
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct EngineKeySignature(KeySignature);

impl EngineKeySignature {
    pub const fn min_code() -> KeyCode {
        1
    }
    pub const fn max_code() -> KeyCode {
        24
    }

    pub fn is_valid_code(code: KeyCode) -> bool {
        code >= KeySignature::min_code() && code <= KeySignature::max_code()
    }

    pub fn from_code(code: KeyCode) -> Self {
        debug_assert!(Self::is_valid_code(code));
        EngineKeySignature(KeySignature::from_code(code % 24 + 1))
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
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
