// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

mod _core {
    pub use aoide_core::track::index::*;
}

///////////////////////////////////////////////////////////////////////
// Index
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum Index {
    Number(u16),
    NumberAndTotal(u16, u16),
}

impl Index {
    fn encode(from: _core::Index) -> Option<Self> {
        match (from.number, from.total) {
            (None, None) => None,
            (Some(number), None) => Some(Index::Number(number)),
            (None, Some(total)) => Some(Index::NumberAndTotal(0, total)),
            (Some(number), Some(total)) => Some(Index::NumberAndTotal(number, total)),
        }
    }

    fn decode(from: Option<Self>) -> _core::Index {
        if let Some(from) = from {
            use Index::*;
            match from {
                Number(number) => _core::Index {
                    number: Some(number),
                    ..Default::default()
                },
                NumberAndTotal(number, total) => _core::Index {
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
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
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
    pub(crate) fn is_default(&self) -> bool {
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
