// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::util::canonical::{Canonical, CanonicalizeInto as _};

use crate::prelude::*;

use super::{actor::Actor, title::Title};

mod _core {
    pub(super) use aoide_core::track::album::*;
}

///////////////////////////////////////////////////////////////////////
// Album
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[repr(u8)]
pub enum Kind {
    NoCompilation = 0,
    Compilation = 1,
    Album = 2,
    Single = 3,
}

impl From<_core::Kind> for Kind {
    fn from(from: _core::Kind) -> Self {
        use _core::Kind::*;
        match from {
            NoCompilation => Self::NoCompilation,
            Album => Self::Album,
            Single => Self::Single,
            Compilation => Self::Compilation,
        }
    }
}

impl From<Kind> for _core::Kind {
    fn from(from: Kind) -> Self {
        use Kind::*;
        match from {
            NoCompilation => Self::NoCompilation,
            Album => Self::Album,
            Single => Self::Single,
            Compilation => Self::Compilation,
        }
    }
}

impl Default for Kind {
    fn default() -> Self {
        _core::Kind::default().into()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Album {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<Kind>,

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
        kind.is_none() && titles.is_empty() && actors.is_empty()
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
            kind: kind.map(Into::into),
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
            kind: kind.map(Into::into),
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
