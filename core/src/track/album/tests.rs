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

#[test]
fn validate_main_title() {
    let mut album = Album {
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
    let mut album = Album {
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
