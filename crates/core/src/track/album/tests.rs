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

use crate::track::title::TitleKind;

use super::*;

#[test]
fn with_main_title() {
    let album = Album {
        titles: Canonical::tie(vec![Title {
            name: "main".to_string(),
            kind: TitleKind::Main,
        }]),
        ..Default::default()
    };
    assert!(album.validate().is_ok());
}

#[test]
fn without_main_title() {
    let album = Album {
        titles: Canonical::tie(vec![Title {
            name: "sub".to_string(),
            kind: TitleKind::Sub,
        }]),
        ..Default::default()
    };
    assert!(album.validate().is_err());
}

#[test]
fn with_main_artist() {
    let album = Album {
        titles: Canonical::tie(vec![Title {
            name: "main".to_string(),
            kind: TitleKind::Main,
        }]),
        actors: Canonical::tie(vec![Actor {
            name: "artist".to_string(),
            role: ActorRole::Artist,
            ..Default::default()
        }]),
        ..Default::default()
    };
    assert!(Actors::main_actor(album.actors.iter(), Default::default()).is_some());
    assert!(album.validate().is_ok());
}

#[test]
fn without_main_artist() {
    let album = Album {
        titles: Canonical::tie(vec![Title {
            name: "main".to_string(),
            kind: TitleKind::Main,
        }]),
        actors: Canonical::tie(vec![Actor {
            name: "composer".to_string(),
            role: ActorRole::Composer,
            ..Default::default()
        }]),
        ..Default::default()
    };
    // No main artist required
    assert!(Actors::main_actor(album.actors.iter(), Default::default()).is_none());
    assert!(album.validate().is_ok());
}
