// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn parse() {
    assert_eq!(
        Some(Label::from_unchecked("A Label")),
        Label::clamp_from("A Label")
    );
}

#[test]
fn clamp_from() {
    assert_eq!(
        Some(Label::from_unchecked("A Label")),
        Label::clamp_from("\tA Label  "),
    );
}

#[test]
fn validate() {
    assert!(Label::new("A Term".into()).validate().is_ok());
    assert!(Label::new("\tA Term  ".into()).validate().is_err());
}

#[test]
fn default_is_invalid() {
    assert!(Label::default().validate().is_err());
}

#[test]
fn parse_empty() {
    assert!(Label::clamp_from("").is_none());
}
