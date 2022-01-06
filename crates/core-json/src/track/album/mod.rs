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

use aoide_core::util::canonical::{Canonical, CanonicalizeInto as _};

use crate::prelude::*;

use super::{actor::Actor, title::Title};

mod _core {
    pub use aoide_core::track::album::*;
}

///////////////////////////////////////////////////////////////////////
// Album
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize_repr, Deserialize_repr, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[repr(u8)]
pub enum AlbumKind {
    Unknown = 0,
    Album = 1,
    Single = 2,
    Compilation = 3,
}

impl AlbumKind {
    fn is_default(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}

impl From<_core::AlbumKind> for AlbumKind {
    fn from(from: _core::AlbumKind) -> Self {
        use _core::AlbumKind::*;
        match from {
            Unknown => Self::Unknown,
            Album => Self::Album,
            Single => Self::Single,
            Compilation => Self::Compilation,
        }
    }
}

impl From<AlbumKind> for _core::AlbumKind {
    fn from(from: AlbumKind) -> Self {
        use AlbumKind::*;
        match from {
            Unknown => Self::Unknown,
            Album => Self::Album,
            Single => Self::Single,
            Compilation => Self::Compilation,
        }
    }
}

impl Default for AlbumKind {
    fn default() -> Self {
        _core::AlbumKind::default().into()
    }
}

#[derive(Debug, Default, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Album {
    #[serde(
        rename = "type",
        skip_serializing_if = "AlbumKind::is_default",
        default
    )]
    pub kind: AlbumKind,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub titles: Vec<Title>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub actors: Vec<Actor>,
}

impl Album {
    pub(crate) fn is_default(&self) -> bool {
        let Self {
            kind,
            titles,
            actors,
        } = self;
        kind.is_default() && titles.is_empty() && actors.is_empty()
    }
}

impl From<_core::Album> for Album {
    fn from(from: _core::Album) -> Self {
        let _core::Album {
            kind,
            titles,
            actors,
        } = from;
        Self {
            kind: kind.into(),
            titles: titles.untie().into_iter().map(Into::into).collect(),
            actors: actors.untie().into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Album> for Canonical<_core::Album> {
    fn from(from: Album) -> Self {
        let Album {
            kind,
            titles,
            actors,
        } = from;
        Self::tie(_core::Album {
            kind: kind.into(),
            titles: Canonical::tie(
                titles
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<_>>()
                    .canonicalize_into(),
            ),
            actors: Canonical::tie(
                actors
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<_>>()
                    .canonicalize_into(),
            ),
        })
    }
}

#[cfg(test)]
mod tests;
