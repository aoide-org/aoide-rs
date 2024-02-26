// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt;

pub type KeyCodeValue = u8;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, strum::FromRepr, strum::EnumIter)]
#[repr(u8)]
pub enum KeyCode {
    /// Off key
    Off = 0,

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

    /// F\u{266F}/G\u{266D} minor
    Gbmin = 8,

    /// E major
    Emaj = 9,

    /// D\u{266D} minor
    Dbmin = 10,

    /// B major
    Bmaj = 11,

    /// A\u{266D} minor
    Abmin = 12,

    /// F\u{266F}/G\u{266D} major
    Gbmaj = 13,

    /// E\u{266D} minor
    Ebmin = 14,

    /// D\u{266D} major
    Dbmaj = 15,

    /// B\u{266D} minor
    Bbmin = 16,

    /// A\u{266D} major
    Abmaj = 17,

    /// F minor
    Fmin = 18,

    /// E\u{266D} major
    Ebmaj = 19,

    /// C minor
    Cmin = 20,

    /// B\u{266D} major
    Bbmaj = 21,

    /// G minor
    Gmin = 22,

    /// F major
    Fmaj = 23,

    /// D minor
    Dmin = 24,
}

impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_canonical_str())
    }
}

impl KeyCode {
    #[must_use]
    pub const fn as_canonical_str(self) -> &'static str {
        #[allow(clippy::enum_glob_use)]
        use KeyCode::*;
        match self {
            Off => "Off",
            Cmaj => "Cmaj",
            Amin => "Amin",
            Gmaj => "Gmaj",
            Emin => "Emin",
            Dmaj => "Dmaj",
            Bmin => "Bmin",
            Amaj => "Amaj",
            Gbmin => "Gbmin",
            Emaj => "Emaj",
            Dbmin => "Dbmin",
            Bmaj => "Bmaj",
            Abmin => "Abmin",
            Gbmaj => "Gbmaj",
            Ebmin => "Ebmin",
            Dbmaj => "Dbmaj",
            Bbmin => "Bbmin",
            Abmaj => "Abmaj",
            Fmin => "Fmin",
            Ebmaj => "Ebmaj",
            Cmin => "Cmin",
            Bbmaj => "Bbmaj",
            Gmin => "Gmin",
            Fmaj => "Fmaj",
            Dmin => "Dmin",
        }
    }

    #[must_use]
    pub fn try_from_canonical_str(s: &str) -> Option<Self> {
        #[allow(clippy::enum_glob_use)]
        use KeyCode::*;
        let code = match s {
            "Off" => Off,
            "Cmaj" => Cmaj,
            "Amin" => Amin,
            "Gmaj" => Gmaj,
            "Emin" => Emin,
            "Dmaj" => Dmaj,
            "Bmin" => Bmin,
            "Amaj" => Amaj,
            "Gbmin" => Gbmin,
            "Emaj" => Emaj,
            "Dbmin" => Dbmin,
            "Bmaj" => Bmaj,
            "Abmin" => Abmin,
            "Gbmaj" => Gbmaj,
            "Ebmin" => Ebmin,
            "Dbmaj" => Dbmaj,
            "Bbmin" => Bbmin,
            "Abmaj" => Abmaj,
            "Fmin" => Fmin,
            "Ebmaj" => Ebmaj,
            "Cmin" => Cmin,
            "Bbmaj" => Bbmaj,
            "Gmin" => Gmin,
            "Fmaj" => Fmaj,
            "Dmin" => Dmin,
            _ => {
                return None;
            }
        };
        Some(code)
    }

    #[must_use]
    pub const fn as_traditional_str(self) -> &'static str {
        #[allow(clippy::enum_glob_use)]
        use KeyCode::*;
        match self {
            Off => "",
            Cmaj => "C",
            Amin => "Am",
            Gmaj => "G",
            Emin => "Em",
            Dmaj => "D",
            Bmin => "Bm",
            Amaj => "A",
            Gbmin => "G\u{266D}m/F\u{266F}m",
            Emaj => "E",
            Dbmin => "D\u{266D}m/C\u{266F}m",
            Bmaj => "B",
            Abmin => "A\u{266D}m/G\u{266F}m",
            Gbmaj => "G\u{266D}/F\u{266F}",
            Ebmin => "E\u{266D}m/D\u{266F}m",
            Dbmaj => "D\u{266D}/C\u{266F}",
            Bbmin => "B\u{266D}m",
            Abmaj => "A\u{266D}/G\u{266F}",
            Fmin => "Fm",
            Ebmaj => "E\u{266D}/D\u{266F}",
            Cmin => "Cm",
            Bbmaj => "B\u{266D}",
            Gmin => "Gm",
            Fmaj => "F",
            Dmin => "Dm",
        }
    }

    #[must_use]
    pub fn try_from_traditional_str(s: &str) -> Option<Self> {
        #[allow(clippy::enum_glob_use)]
        use KeyCode::*;
        let code = match s {
            "" => Off,
            "C" => Cmaj,
            "a" | "Am" => Amin,
            "G" => Gmaj,
            "e" | "Em" => Emin,
            "D" => Dmaj,
            "b" | "Bm" => Bmin,
            "A" => Amaj,
            "G\u{266D}m/F\u{266F}m"
            | "G\u{266D}m"
            | "F\u{266F}m"
            | "F\u{266F}m/G\u{266D}m"
            | "g\u{266D}/f\u{266F}"
            | "g\u{266D}"
            | "f\u{266F}"
            | "f\u{266F}/g\u{266D}" => Gbmin,
            "E" => Emaj,
            "D\u{266D}m/C\u{266F}m"
            | "D\u{266D}m"
            | "C\u{266F}m"
            | "C\u{266F}m/D\u{266D}m"
            | "d\u{266D}/c\u{266F}"
            | "d\u{266D}"
            | "c\u{266F}"
            | "c\u{266F}/d\u{266D}" => Dbmin,
            "B" => Bmaj,
            "A\u{266D}/G\u{266F}m"
            | "A\u{266D}m"
            | "G\u{266F}m"
            | "G\u{266F}m/A\u{266D}m"
            | "a\u{266D}/g\u{266F}"
            | "a\u{266D}"
            | "g\u{266F}"
            | "g\u{266F}/a\u{266D}" => Abmin,
            "G\u{266D}/F\u{266F}" | "G\u{266D}" | "F\u{266F}" | "F\u{266F}/G\u{266D}" => Gbmaj,
            "E\u{266D}m/D\u{266F}m"
            | "E\u{266D}m"
            | "D\u{266F}m"
            | "D\u{266F}m/E\u{266D}m"
            | "e\u{266D}/d\u{266F}"
            | "e\u{266D}"
            | "d\u{266F}"
            | "d\u{266F}/e\u{266D}" => Ebmin,
            "D\u{266D}/C\u{266F}" | "D\u{266D}" | "C\u{266F}" | "C\u{266F}/D\u{266D}" => Dbmaj,
            "B\u{266D}m" | "b\u{266D}" => Bbmin,
            "A\u{266D}/G\u{266F}" | "A\u{266D}" | "G\u{266F}" | "G\u{266F}/A\u{266D}" => Abmaj,
            "Fm" | "f" => Fmin,
            "E\u{266D}/D\u{266F}" | "E\u{266D}" | "D\u{266F}" | "D\u{266F}/E\u{266D}" => Ebmaj,
            "Cm" | "c" => Cmin,
            "B\u{266D}" => Bbmaj,
            "Gm" | "g" => Gmin,
            "F" => Fmaj,
            "Dm" | "d" => Dmin,
            _ => {
                return None;
            }
        };
        Some(code)
    }

    #[must_use]
    pub const fn as_traditional_ascii_str(self) -> &'static str {
        #[allow(clippy::enum_glob_use)]
        use KeyCode::*;
        match self {
            Off => "",
            Cmaj => "C",
            Amin => "Am",
            Gmaj => "G",
            Emin => "Em",
            Dmaj => "D",
            Bmin => "Bm",
            Amaj => "A",
            Gbmin => "Gbm/F#m",
            Emaj => "E",
            Dbmin => "Dbm/C#m",
            Bmaj => "B",
            Abmin => "Abm/G#m",
            Gbmaj => "Gb/F#",
            Ebmin => "Ebm/D#m",
            Dbmaj => "Db/C#",
            Bbmin => "Bbm",
            Abmaj => "Ab/G#",
            Fmin => "Fm",
            Ebmaj => "Eb/D#",
            Cmin => "Cm",
            Bbmaj => "Bb",
            Gmin => "Gm",
            Fmaj => "F",
            Dmin => "Dm",
        }
    }

    #[must_use]
    pub fn try_from_traditional_ascii_str(s: &str) -> Option<Self> {
        #[allow(clippy::enum_glob_use)]
        use KeyCode::*;
        let code = match s {
            "" => Off,
            "C" => Cmaj,
            "Am" | "a" => Amin,
            "G" => Gmaj,
            "Em" | "e" => Emin,
            "D" => Dmaj,
            "Bm" | "b" => Bmin,
            "A" => Amaj,
            "Gbm/F#m" | "Gbm" | "F#m" | "F#m/Gbm" | "gb/f#" | "gb" | "f#" | "f#/gb" => Gbmin,
            "E" => Emaj,
            "Dbm/C#m" | "Dbm" | "C#m" | "C#m/Dbm" | "db/c#" | "db" | "c#" | "c#/db" => Dbmin,
            "B" => Bmaj,
            "Ab/G#m" | "Abm" | "G#m" | "G#m/Abm" | "ab/g#" | "ab" | "g#" | "g#/ab" => Abmin,
            "Gb/F#" | "Gb" | "F#" | "F#/Gb" => Gbmaj,
            "Ebm/D#m" | "Ebm" | "D#m" | "D#m/Ebm" | "eb/d#" | "eb" | "d#" | "d#/eb" => Ebmin,
            "Db/C#" | "Db" | "C#" | "C#/Db" => Dbmaj,
            "Bbm" | "bb" => Bbmin,
            "Ab/G#" | "Ab" | "G#" | "G#/Ab" => Abmaj,
            "Fm" | "f" => Fmin,
            "Eb/D#" | "Eb" | "D#" | "D#/Eb" => Ebmaj,
            "Cm" | "c" => Cmin,
            "B\u{266D}" => Bbmaj,
            "Gm" | "g" => Gmin,
            "F" => Fmaj,
            "Dm" | "d" => Dmin,
            _ => {
                return None;
            }
        };
        Some(code)
    }

    #[must_use]
    pub const fn as_openkey_str(self) -> &'static str {
        #[allow(clippy::enum_glob_use)]
        use KeyCode::*;
        match self {
            Off => "",
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

    #[must_use]
    pub fn try_from_openkey_str(s: &str) -> Option<Self> {
        #[allow(clippy::enum_glob_use)]
        use KeyCode::*;
        let code = match s {
            "" => Off,
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
            _ => {
                return None;
            }
        };
        Some(code)
    }

    #[must_use]
    pub const fn as_lancelot_str(self) -> &'static str {
        #[allow(clippy::enum_glob_use)]
        use KeyCode::*;
        match self {
            Off => "",
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

    #[must_use]
    pub fn try_from_camelot_str(s: &str) -> Option<Self> {
        #[allow(clippy::enum_glob_use)]
        use KeyCode::*;
        let code = match s {
            "" => Off,
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
            _ => {
                return None;
            }
        };
        Some(code)
    }

    #[must_use]
    pub const fn as_traxsource_str(self) -> &'static str {
        #[allow(clippy::enum_glob_use)]
        use KeyCode::*;
        match self {
            Off => "",
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

    #[must_use]
    pub fn try_from_traxsource_str(s: &str) -> Option<Self> {
        #[allow(clippy::enum_glob_use)]
        use KeyCode::*;
        let code = match s {
            "" => Off,
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
            _ => {
                return None;
            }
        };
        Some(code)
    }

    #[must_use]
    pub const fn as_beatport_str(self) -> &'static str {
        #[allow(clippy::enum_glob_use)]
        use KeyCode::*;
        match self {
            Off => "",
            Cmaj => "C maj",
            Amin => "A min",
            Gmaj => "G maj",
            Emin => "E min",
            Dmaj => "D maj",
            Bmin => "B min",
            Amaj => "Amaj",
            Gbmin => "G\u{266D}/F\u{266F} min",
            Emaj => "E maj",
            Dbmin => "D\u{266D}/C\u{266F} min",
            Bmaj => "B maj",
            Abmin => "A\u{266D}/G\u{266F} min",
            Gbmaj => "G\u{266D}/F\u{266F} maj",
            Ebmin => "E\u{266D}/D\u{266F} min",
            Dbmaj => "D\u{266D}/C\u{266F} maj",
            Bbmin => "B\u{266D}/A\u{266F} min",
            Abmaj => "A\u{266D}/G\u{266F} maj",
            Fmin => "F min",
            Ebmaj => "E\u{266D}/D\u{266F} maj",
            Cmin => "C min",
            Bbmaj => "B\u{266D}/A\u{266F} maj",
            Gmin => "G min",
            Fmaj => "F maj",
            Dmin => "D min",
        }
    }

    #[must_use]
    pub fn try_from_beatport_str(s: &str) -> Option<Self> {
        #[allow(clippy::enum_glob_use)]
        use KeyCode::*;
        let code = match s {
            "" => Off,
            "C maj" => Cmaj,
            "A min" => Amin,
            "G maj" => Gmaj,
            "E min" => Emin,
            "D maj" => Dmaj,
            "B min" => Bmin,
            "A maj" => Amaj,
            "G\u{266D} min" | "F\u{266F} min" | "G\u{266D}/F\u{266F} min" => Gbmin,
            "E maj" => Emaj,
            "D\u{266D} min" | "C\u{266F} min" | "D\u{266D}/C\u{266F} min" => Dbmin,
            "B maj" => Bmaj,
            "A\u{266D}\u{266F} min" | "G\u{266F} min" | "A\u{266D}/G\u{266F} min" => Abmin,
            "G\u{266D} maj" | "F\u{266F} maj" | "G\u{266D}/F\u{266F} maj" => Gbmaj,
            "E\u{266D} min" | "D\u{266F} min" | "E\u{266D}/D\u{266F} min" => Ebmin,
            "D\u{266D} maj" | "C\u{266F} maj" | "D\u{266D}/C\u{266F} maj" => Dbmaj,
            "B\u{266D} min" => Bbmin,
            "A\u{266D} maj" | "G\u{266F} maj" | "A\u{266D}/G\u{266F} maj" => Abmaj,
            "F min" => Fmin,
            "E\u{266D} maj" | "D\u{266F} maj" | "E\u{266D}/D\u{266F} maj" => Ebmaj,
            "C min" => Cmin,
            "B\u{266D} maj" => Bbmaj,
            "G min" => Gmin,
            "F maj" => Fmaj,
            "D min" => Dmin,
            _ => {
                return None;
            }
        };
        Some(code)
    }

    #[allow(clippy::doc_markdown)]
    /// See also `TKEY` in _ID3v2_: <https://id3.org/id3v2.4.0-frames>
    #[must_use]
    pub const fn as_serato_str(self) -> &'static str {
        #[allow(clippy::enum_glob_use)]
        use KeyCode::*;
        match self {
            Off => "o",
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

    #[allow(clippy::doc_markdown)]
    /// See also `TKEY` in _ID3v2_: <https://id3.org/id3v2.4.0-frames>
    #[must_use]
    pub fn try_from_serato_str(s: &str) -> Option<Self> {
        #[allow(clippy::enum_glob_use)]
        use KeyCode::*;
        let code = match s {
            "o" => Off,
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
            _ => {
                return None;
            }
        };
        Some(code)
    }
}

impl KeyCode {
    #[must_use]
    pub const fn to_value(self) -> KeyCodeValue {
        self as _
    }

    #[must_use]
    pub const fn try_from_value(val: KeyCodeValue) -> Option<Self> {
        Self::from_repr(val)
    }
}

impl TryFrom<KeyCodeValue> for KeyCode {
    type Error = ();

    fn try_from(from: KeyCodeValue) -> Result<Self, Self::Error> {
        Self::try_from_value(from).ok_or(())
    }
}

impl From<KeyCode> for KeyCodeValue {
    fn from(from: KeyCode) -> Self {
        from.to_value()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KeyMode {
    Major,
    Minor,
}

/// The ordering numbering of the key code follows the
/// Circle of fifth / Open Key notation in clock-wise orientation,
/// alternating between major and minor keys.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeySignature(KeyCode);

impl KeySignature {
    #[must_use]
    pub const fn new(code: KeyCode) -> Self {
        Self(code)
    }

    #[must_use]
    pub const fn code(self) -> KeyCode {
        let Self(code) = self;
        code
    }

    #[must_use]
    pub const fn mode(self) -> Option<KeyMode> {
        match self.code() {
            KeyCode::Off => None,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OpenKeySignature(KeySignature);

impl OpenKeySignature {
    pub const MIN_CODE: KeyCodeValue = 1;
    pub const MAX_CODE: KeyCodeValue = 12;

    #[must_use]
    #[allow(clippy::missing_panics_doc)] // Never panics
    #[allow(clippy::similar_names)] // False positive
    pub fn new(code: KeyCodeValue, mode: KeyMode) -> Self {
        #[allow(clippy::missing_panics_doc)] // Never panics
        let code = KeyCode::try_from_value(
            2 * code
                - match mode {
                    KeyMode::Major => 1,
                    KeyMode::Minor => 0,
                },
        )
        .expect("valid key code");
        Self(KeySignature::new(code))
    }

    #[must_use]
    pub const fn code(self) -> KeyCodeValue {
        1 + (self.0.code().to_value() - 1) / 2
    }

    #[must_use]
    pub const fn mode(self) -> Option<KeyMode> {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LancelotKeySignature(KeySignature);

impl LancelotKeySignature {
    pub const MIN_CODE: KeyCodeValue = 1;
    pub const MAX_CODE: KeyCodeValue = 12;

    #[must_use]
    #[allow(clippy::missing_panics_doc)] // Never panics
    #[allow(clippy::similar_names)] // False positive
    pub fn new(code: KeyCodeValue, mode: KeyMode) -> Self {
        let code = KeyCode::try_from_value(
            ((code * 2 + 9) % 24)
                + match mode {
                    KeyMode::Major => 0,
                    KeyMode::Minor => 1,
                },
        )
        .expect("valid key code");
        Self(KeySignature::new(code))
    }

    #[must_use]
    pub const fn code(self) -> KeyCodeValue {
        1 + ((self.0.code().to_value() + 13) / 2) % 12
    }

    #[must_use]
    pub const fn mode(self) -> Option<KeyMode> {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EngineKeySignature(KeySignature);

impl EngineKeySignature {
    pub const MIN_CODE: KeyCodeValue = 1;
    pub const MAX_CODE: KeyCodeValue = 24;

    #[must_use]
    #[allow(clippy::missing_panics_doc)] // Never panics
    pub fn from_code(code: KeyCodeValue) -> Self {
        let code = KeyCode::try_from_value(code % 24 + 1).expect("valid key code");
        Self(KeySignature::new(code))
    }

    #[must_use]
    pub const fn code(self) -> KeyCodeValue {
        match self.0.code().to_value() {
            1 => 24,
            code => code - 1,
        }
    }

    #[must_use]
    pub const fn mode(self) -> Option<KeyMode> {
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
