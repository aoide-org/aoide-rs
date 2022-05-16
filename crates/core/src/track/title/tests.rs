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

#[test]
fn title_names() {
    assert!(is_valid_title_name("A valid\ntitle\tname"));
    assert!(!is_valid_title_name(" Leading whitespace"));
    assert!(!is_valid_title_name("Trailing whitespace\n"));
    assert!(!is_valid_title_name(""));
    assert!(!is_valid_title_name(" "));
    assert!(!is_valid_title_name("\t"));
}

#[test]
fn validate_empty_titles() {
    let titles = [];
    assert!(Titles::validate(&titles.iter()).is_ok());
}

#[test]
fn validate_main_title() {
    let titles = [Title {
        name: "title1".into(),
        kind: TitleKind::Main,
    }];
    assert!(Titles::validate(&titles.iter()).is_ok());
}

#[test]
fn validate_single_main_title() {
    let titles = [Title {
        name: "title1".into(),
        kind: TitleKind::Main,
    }];
    assert!(Titles::validate(&titles.iter()).is_ok());
}

#[test]
fn validate_missing_main_title() {
    let titles = [Title {
        name: "title1".into(),
        kind: TitleKind::Sub,
    }];
    assert_eq!(
        1,
        Titles::validate(&titles.iter())
            .err()
            .unwrap()
            .into_iter()
            .count()
    );
}

#[test]
fn validate_ambiguous_main_title() {
    let titles = [
        Title {
            name: "title1".into(),
            kind: TitleKind::Main,
        },
        Title {
            name: "title2".into(),
            kind: TitleKind::Main,
        },
    ];
    assert_eq!(
        1,
        Titles::validate(&titles.iter())
            .err()
            .unwrap()
            .into_iter()
            .count()
    );
}

#[test]
fn validate_multiple_titles() {
    let titles = [
        Title {
            name: "title1".into(),
            kind: TitleKind::Main,
        },
        Title {
            name: "title2".into(),
            kind: TitleKind::Sub,
        },
        Title {
            name: "title3".into(),
            kind: TitleKind::Sub,
        },
        Title {
            name: "title4".into(),
            kind: TitleKind::Work,
        },
        Title {
            name: "title4".into(),
            kind: TitleKind::Movement,
        },
    ];
    assert!(Titles::validate(&titles.iter()).is_ok());
}
