// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

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
        kind: Kind::Main,
    }];
    assert!(Titles::validate(&titles.iter()).is_ok());
}

#[test]
fn validate_single_main_title() {
    let titles = [Title {
        name: "title1".into(),
        kind: Kind::Main,
    }];
    assert!(Titles::validate(&titles.iter()).is_ok());
}

#[test]
fn validate_missing_main_title() {
    let titles = [Title {
        name: "title1".into(),
        kind: Kind::Sub,
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
            kind: Kind::Main,
        },
        Title {
            name: "title2".into(),
            kind: Kind::Main,
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
            kind: Kind::Main,
        },
        Title {
            name: "title2".into(),
            kind: Kind::Sub,
        },
        Title {
            name: "title3".into(),
            kind: Kind::Sub,
        },
        Title {
            name: "title4".into(),
            kind: Kind::Work,
        },
        Title {
            name: "title4".into(),
            kind: Kind::Movement,
        },
    ];
    assert!(Titles::validate(&titles.iter()).is_ok());
}
