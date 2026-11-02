// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

#[test]
fn set_kind_title() {
    let mut titles = vec![Title {
        name: "main".into(),
        kind: Kind::Main,
    }];
    // Add
    assert!(Titles::set_kind_title(
        &mut titles,
        Kind::Sub,
        Some("sub".into())
    ));
    titles.canonicalize();
    assert_eq!(
        Some("sub"),
        Titles::kind_title(&titles, Kind::Sub).map(|title| title.name.as_str())
    );
    // Unmodified
    assert!(!Titles::set_kind_title(
        &mut titles,
        Kind::Sub,
        Some("sub".into())
    ));
    // Replace
    assert!(Titles::set_kind_title(
        &mut titles,
        Kind::Sub,
        Some("sub2".into())
    ));
    titles.canonicalize();
    assert_eq!(2, titles.len());
    assert_eq!(
        Some("sub2"),
        Titles::kind_title(&titles, Kind::Sub).map(|title| title.name.as_str())
    );
    // Remove
    assert!(Titles::set_kind_title(&mut titles, Kind::Sub, None));
    assert!(!Titles::set_kind_title(&mut titles, Kind::Sub, None));
    assert_eq!(
        Some("main"),
        Titles::main_title(&titles).map(|title| title.name.as_str())
    );
    assert_eq!(1, titles.len());
}
