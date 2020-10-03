// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
    pub use aoide_core::track::album::*;
}

use crate::{actor::*, title::*};

///////////////////////////////////////////////////////////////////////
// Album
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Album {
    #[serde(rename = "tit", skip_serializing_if = "Vec::is_empty", default)]
    pub titles: Vec<Title>,

    #[serde(rename = "act", skip_serializing_if = "Vec::is_empty", default)]
    pub actors: Vec<Actor>,

    #[serde(rename = "cpl", skip_serializing_if = "Option::is_none")]
    pub compilation: Option<bool>,
}

impl From<_core::Album> for Album {
    fn from(from: _core::Album) -> Self {
        let _core::Album {
            titles,
            actors,
            compilation,
        } = from;
        Self {
            titles: titles.into_iter().map(Into::into).collect(),
            actors: actors.into_iter().map(Into::into).collect(),
            compilation,
        }
    }
}

impl From<Album> for _core::Album {
    fn from(from: Album) -> Self {
        let Album {
            titles,
            actors,
            compilation,
        } = from;
        Self {
            titles: titles.into_iter().map(Into::into).collect(),
            actors: actors.into_iter().map(Into::into).collect(),
            compilation,
        }
    }
}
