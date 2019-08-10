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

use crate::metadata::{actor::*, title::*};

///////////////////////////////////////////////////////////////////////
// AlbumMetadata
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AlbumMetadata {
    #[serde(rename = "tit", skip_serializing_if = "Vec::is_empty", default)]
    #[validate(length(min = 1), custom = "Titles::validate_main_title")]
    pub titles: Vec<Title>,

    #[serde(rename = "act", skip_serializing_if = "Vec::is_empty", default)]
    #[validate(custom = "Actors::validate_main_actor")]
    pub actors: Vec<Actor>,

    #[serde(rename = "cpl", skip_serializing_if = "Option::is_none")]
    pub compilation: Option<bool>,
}

impl AlbumMetadata {
    pub fn main_title(&self) -> Option<&Title> {
        Titles::main_title(&self.titles)
    }

    pub fn main_actor(&self, role: ActorRole) -> Option<&Actor> {
        Actors::main_actor(&self.actors, role)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AlbumTracksCount {
    #[serde(rename = "tit", skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(rename = "art", skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,

    #[serde(rename = "rly", skip_serializing_if = "Option::is_none")]
    pub release_year: Option<i16>,

    #[serde(rename = "cnt")]
    pub count: usize,
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_main_title() {
        let mut album = AlbumMetadata {
            titles: vec![Title {
                name: "main".to_string(),
                level: TitleLevel::Main,
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(album.validate().is_ok());
        album.titles = vec![Title {
            name: "sub".to_string(),
            level: TitleLevel::Sub,
            ..Default::default()
        }];
        assert!(album.validate().is_err());
    }

    #[test]
    fn validate_main_actor() {
        let mut album = AlbumMetadata {
            titles: vec![Title {
                name: "main".to_string(),
                level: TitleLevel::Main,
                ..Default::default()
            }],
            actors: vec![Actor {
                name: "artist".to_string(),
                role: ActorRole::Artist,
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(album.validate().is_ok());
        album.actors = vec![Actor {
            name: "composer".to_string(),
            role: ActorRole::Composer,
            ..Default::default()
        }];
        assert!(album.validate().is_err());
    }
}
