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
fn validate_empty_titles() {
    let titles = [];
    assert!(Titles::validate(&titles).is_ok());
}

#[test]
fn validate_main_title() {
    let titles = [Title {
        name: "title1".into(),
        level: TitleLevel::Main,
        language: None,
    }];
    assert!(Titles::validate(&titles).is_ok());

    let titles = [
        Title {
            name: "title1".into(),
            level: TitleLevel::Main,
            language: None,
        },
        Title {
            name: "title2".into(),
            level: TitleLevel::Main,
            language: None,
        },
    ];
    assert_eq!(
        1,
        Titles::validate(&titles).err().unwrap().into_iter().count()
    );

    let titles = [
        Title {
            name: "title1".into(),
            level: TitleLevel::Main,
            language: None,
        },
        Title {
            name: "title2".into(),
            level: TitleLevel::Main,
            language: Some("en".into()),
        },
    ];
    assert!(Titles::validate(&titles).is_ok());

    let titles = [Title {
        name: "title1".into(),
        level: TitleLevel::Main,
        language: Some("en".into()),
    }];
    assert!(Titles::validate(&titles).is_ok());
}
