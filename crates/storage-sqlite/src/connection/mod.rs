// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    fmt,
    path::{Path, PathBuf},
    str::FromStr,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub mod pool;

pub const IN_MEMORY_STORAGE: &str = ":memory:";

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Storage {
    InMemory,
    File { path: PathBuf },
}

impl Storage {
    #[must_use]
    pub fn parent_dir(&self) -> Option<&Path> {
        match self {
            Self::InMemory => None,
            Self::File { path } => path.parent(),
        }
    }
}

impl AsRef<str> for Storage {
    fn as_ref(&self) -> &str {
        match self {
            Self::InMemory => IN_MEMORY_STORAGE,
            Self::File { path } => path.to_str().expect("valid UTF-8 path"),
        }
    }
}

impl fmt::Display for Storage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl FromStr for Storage {
    type Err = <PathBuf as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.to_lowercase().trim() == IN_MEMORY_STORAGE {
            return Ok(Self::InMemory);
        }
        let path = s.parse()?;
        Ok(Self::File { path })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Config {
    pub storage: Storage,

    pub pool: self::pool::Config,
}

impl Config {
    #[must_use]
    pub fn storage_dir(&self) -> Option<&Path> {
        self.storage.parent_dir()
    }
}
