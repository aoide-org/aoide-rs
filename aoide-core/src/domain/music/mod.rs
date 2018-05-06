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

use domain::metadata::Confidence;

use std::f64;
use std::fmt;

///////////////////////////////////////////////////////////////////////
/// ActorRole
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum ActorRole {
    Artist,
    Arranger,
    Composer,
    Conductor,
    DjMixer,
    Engineer,
    Lyricist,
    Mixer,
    Performer,
    Producer,
    Publisher,
    Remixer,
    Writer,
}

impl Default for ActorRole {
    fn default() -> ActorRole {
        ActorRole::Artist
    }
}

///////////////////////////////////////////////////////////////////////
/// Actor
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Actor {
    role: ActorRole,
    // The prefix contains all necessary separators between the
    // preceding and this actor's name, i.e. there is no implicit
    // spacing!
    #[serde(skip_serializing_if = "String::is_empty", default)]
    prefix: String,
    name: String,
}

impl Actor {
    pub fn artist<S: Into<String>>(name: S) -> Self {
        Self {
            role: ActorRole::Artist,
            prefix: String::default(),
            name: name.into(),
        }
    }

    pub fn prefixed_artist<S1: Into<String>, S2: Into<String>>(prefix: S1, name: S2) -> Self {
        Self {
            role: ActorRole::Artist,
            prefix: prefix.into(),
            name: name.into(),
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty()
    }

    fn string_len(&self) -> usize {
        self.prefix.len() + self.name.len()
    }

    fn append_to_string(&self, builder: &mut String) {
        builder.push_str(&self.prefix);
        builder.push_str(&self.name);
    }

    pub fn actors_to_string(actors: &[Self], role_opt: Option<ActorRole>) -> Option<String> {
        let count = actors
            .iter()
            .filter(|a| role_opt.is_none() || (Some(a.role) == role_opt))
            .count();
        if count > 0 {
            let capacity = actors
                .iter()
                .filter(|a| role_opt.is_none() || (Some(a.role) == role_opt))
                .map(|a| a.string_len())
                .sum();
            let mut builder = String::with_capacity(capacity);
            actors
                .iter()
                .filter(|a| role_opt.is_none() || (Some(a.role) == role_opt))
                .for_each(|a| a.append_to_string(&mut builder));
            Some(builder)
        } else {
            None
        }
    }
}

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
/// TimeSignature
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TimeSignature {
    numerator: u8,   // number of beats in each bar, 0 = default/undefined
    denominator: u8, // symbol length of each beat, 0 = default/undefined
}

impl TimeSignature {
    pub fn is_valid(&self) -> bool {
        (self.numerator > 0) && (self.denominator > 0)
    }
}

impl fmt::Display for TimeSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

///////////////////////////////////////////////////////////////////////
/// KeySignature
///////////////////////////////////////////////////////////////////////

pub type KeyCode = u8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum KeyMode {
    Major,
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
        assert!(Self::is_valid_code(code));
        Self { code }
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
    key_signature: KeySignature,
}

impl OpenKeySignature {
    pub const MIN_CODE: KeyCode = 1;
    pub const MAX_CODE: KeyCode = 12;

    pub fn is_valid_code(code: KeyCode) -> bool {
        code >= KeySignature::MIN_CODE && code <= KeySignature::MAX_CODE
    }

    pub fn new(code: KeyCode, mode: KeyMode) -> Self {
        let key_signature = KeySignature {
            code: (code * 2 - 1) + match mode {
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
    key_signature: KeySignature,
}

impl LancelotKeySignature {
    pub const MIN_CODE: KeyCode = 1;
    pub const MAX_CODE: KeyCode = 12;

    pub fn is_valid_code(code: KeyCode) -> bool {
        code >= KeySignature::MIN_CODE && code <= KeySignature::MAX_CODE
    }

    pub fn new(code: KeyCode, mode: KeyMode) -> Self {
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
/// Classification
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum Classifier {
    Acousticness,
    Danceability,
    Energy,
    Instrumentalness,
    Liveness,
    Popularity,
    Positivity,
    Speechiness,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Classification {
    pub classifier: Classifier,
    pub confidence: Confidence,
}

impl Classification {
    pub fn new<C: Into<Confidence>>(classifier: Classifier, confidence: C) -> Self {
        Self {
            classifier,
            confidence: confidence.into(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.confidence.is_valid()
    }
}

///////////////////////////////////////////////////////////////////////
/// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actor_artists_to_string() {
        let actors = vec![
            Actor::artist("Madonna"),
            Actor {
                role: ActorRole::Producer,
                prefix: String::default(),
                name: "Martin Solveig".to_string(),
            },
            Actor {
                role: ActorRole::Remixer,
                prefix: String::default(),
                name: String::default(),
            },
            Actor::prefixed_artist(" feat. ", "M.I.A."),
            Actor::prefixed_artist(" and ", "Nicki Minaj"),
        ];
        assert_eq!(
            None,
            Actor::actors_to_string(&actors, Some(ActorRole::Composer))
        );
        assert_eq!(
            Some(String::default()),
            Actor::actors_to_string(&actors, Some(ActorRole::Remixer))
        );
        assert_eq!(
            Some("Madonna feat. M.I.A. and Nicki Minaj".to_string()),
            Actor::actors_to_string(&actors, Some(ActorRole::Artist))
        );
    }

    #[test]
    fn default_key_signature() {
        assert!(!KeySignature::default().is_valid());
        assert!(!OpenKeySignature::default().is_valid());
        assert!(!LancelotKeySignature::default().is_valid());
    }

    #[test]
    fn convert_key_signatures() {
        assert_eq!(
            KeySignature::new(1),
            OpenKeySignature::new(1, KeyMode::Major).into()
        ); // C maj
        assert_eq!(
            KeySignature::new(1),
            LancelotKeySignature::new(8, KeyMode::Major).into()
        ); // C maj
        assert_eq!(
            KeySignature::new(2),
            OpenKeySignature::new(1, KeyMode::Minor).into()
        ); // A min
        assert_eq!(
            KeySignature::new(2),
            LancelotKeySignature::new(8, KeyMode::Minor).into()
        ); // A min
        assert_eq!(
            KeySignature::new(9),
            OpenKeySignature::new(5, KeyMode::Major).into()
        ); // E maj
        assert_eq!(
            KeySignature::new(9),
            LancelotKeySignature::new(12, KeyMode::Major).into()
        ); // E maj
        assert_eq!(
            KeySignature::new(10),
            OpenKeySignature::new(5, KeyMode::Minor).into()
        ); // Db min
        assert_eq!(
            KeySignature::new(10),
            LancelotKeySignature::new(12, KeyMode::Minor).into()
        ); // Db min
        assert_eq!(
            KeySignature::new(11),
            OpenKeySignature::new(6, KeyMode::Major).into()
        ); // B maj
        assert_eq!(
            KeySignature::new(11),
            LancelotKeySignature::new(1, KeyMode::Major).into()
        ); // B maj
        assert_eq!(
            KeySignature::new(12),
            OpenKeySignature::new(6, KeyMode::Minor).into()
        ); // Ab min
        assert_eq!(
            KeySignature::new(12),
            LancelotKeySignature::new(1, KeyMode::Minor).into()
        ); // Ab min
        assert_eq!(
            KeySignature::new(23),
            OpenKeySignature::new(12, KeyMode::Major).into()
        ); // F maj
        assert_eq!(
            KeySignature::new(23),
            LancelotKeySignature::new(7, KeyMode::Major).into()
        ); // F maj
        assert_eq!(
            KeySignature::new(24),
            OpenKeySignature::new(12, KeyMode::Minor).into()
        ); // D min
        assert_eq!(
            KeySignature::new(24),
            LancelotKeySignature::new(7, KeyMode::Minor).into()
        ); // D min
    }

    #[test]
    fn display_key_signatures() {
        assert_eq!(
            "1d",
            format!("{}", OpenKeySignature::from(KeySignature::new(1)))
        ); // C maj
        assert_eq!(
            "8B",
            format!("{}", LancelotKeySignature::from(KeySignature::new(1)))
        ); // C maj
        assert_eq!(
            "1m",
            format!("{}", OpenKeySignature::from(KeySignature::new(2)))
        ); // A min
        assert_eq!(
            "8A",
            format!("{}", LancelotKeySignature::from(KeySignature::new(2)))
        ); // A min
        assert_eq!(
            "12d",
            format!("{}", OpenKeySignature::from(KeySignature::new(23)))
        ); // F maj
        assert_eq!(
            "7B",
            format!("{}", LancelotKeySignature::from(KeySignature::new(23)))
        ); // F maj
        assert_eq!(
            "12m",
            format!("{}", OpenKeySignature::from(KeySignature::new(24)))
        ); // D min
        assert_eq!(
            "7A",
            format!("{}", LancelotKeySignature::from(KeySignature::new(24)))
        ); // D min
    }
}
