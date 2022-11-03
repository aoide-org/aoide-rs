// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn trim_owned_in_place_empty() {
    let mut s = String::new();
    trim_owned_in_place(&mut s);
    assert_eq!(String::new(), s);
    let mut s = String::new();
    trim_owned_in_place(&mut s);
    assert_eq!(String::new(), s);
}

#[test]
fn trim_from_empty() {
    let trimmed = trim_from("");
    assert_eq!(String::new(), trimmed);
    let trimmed = trim_from(String::new());
    assert_eq!(String::new(), trimmed);
}

#[test]
fn trim_owned_in_place_whitespace() {
    let mut s = " \n \t \r ".into();
    trim_owned_in_place(&mut s);
    assert_eq!(String::new(), s);
}

#[test]
fn trim_from_whitespace() {
    let trimmed = trim_from(" \n \t \r ");
    assert_eq!(String::new(), trimmed);
}

#[test]
fn trim_owned_in_place_start_end() {
    let mut s = " \n \tThis \n is\ta \r Text\r ".into();
    trim_owned_in_place(&mut s);
    assert_eq!("This \n is\ta \r Text", s.as_str());
}

#[test]
fn trim_from_start_end() {
    let trimmed = trim_from(" \n \tThis \n is\ta \r Text\r ");
    assert_eq!("This \n is\ta \r Text", trimmed);
}

#[test]
fn non_empty_from_empty() {
    assert_eq!(None, non_empty_from(""));
    assert_eq!(None, non_empty_from(String::new()));
}

#[test]
fn non_empty_from_non_empty() {
    assert_eq!(Some(" ".into()), non_empty_from(" "));
}
