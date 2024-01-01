// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

mod _core {
    pub(super) use aoide_core::track::index::*;
}

///////////////////////////////////////////////////////////////////////
// Index
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum Index {
    Number(u16),
    NumberAndTotal(u16, u16),
}

impl Index {
    const fn encode(from: _core::Index) -> Option<Self> {
        match (from.number, from.total) {
            (None, None) => None,
            (Some(number), None) => Some(Index::Number(number)),
            (None, Some(total)) => Some(Index::NumberAndTotal(0, total)),
            (Some(number), Some(total)) => Some(Index::NumberAndTotal(number, total)),
        }
    }

    fn decode(from: Option<Self>) -> _core::Index {
        if let Some(from) = from {
            use Index as From;
            match from {
                From::Number(number) => _core::Index {
                    number: Some(number),
                    ..Default::default()
                },
                From::NumberAndTotal(number, total) => _core::Index {
                    number: Some(number),
                    total: Some(total),
                },
            }
        } else {
            Default::default()
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Indexes
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Indexes {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<Index>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disc: Option<Index>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub movement: Option<Index>,
}

impl Indexes {
    pub(crate) const fn is_default(&self) -> bool {
        let Self {
            track,
            disc,
            movement,
        } = self;
        track.is_none() && disc.is_none() && movement.is_none()
    }
}

impl From<_core::Indexes> for Indexes {
    fn from(from: _core::Indexes) -> Self {
        Self {
            disc: Index::encode(from.disc),
            track: Index::encode(from.track),
            movement: Index::encode(from.movement),
        }
    }
}

impl From<Indexes> for _core::Indexes {
    fn from(from: Indexes) -> Self {
        Self {
            disc: Index::decode(from.disc),
            track: Index::decode(from.track),
            movement: Index::decode(from.movement),
        }
    }
}

#[cfg(test)]
mod tests;
