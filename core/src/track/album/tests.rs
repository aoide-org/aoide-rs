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

use super::*;

#[test]
fn validate_main_title() {
    let mut album = Album {
        titles: Canonical::tie(vec![Title {
            name: "main".to_string(),
            kind: TitleKind::Main,
        }]),
        ..Default::default()
    };
    assert!(album.validate().is_ok());
    album.titles = Canonical::tie(vec![Title {
        name: "sub".to_string(),
        kind: TitleKind::Sub,
    }]);
    assert!(album.validate().is_err());
}

#[test]
fn validate_main_actor() {
    let mut album = Album {
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
    assert!(album.validate().is_ok());
    album.actors = Canonical::tie(vec![Actor {
        name: "composer".to_string(),
        role: ActorRole::Composer,
        ..Default::default()
    }]);
    assert!(album.validate().is_err());
}
