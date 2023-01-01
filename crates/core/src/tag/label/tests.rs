// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn parse() {
    assert_eq!(
        Some(Label::new("A Label".into())),
        Label::clamp_from("A Label")
    );
}

#[test]
fn clamp_value() {
    assert_eq!(
        Some("A Label"),
        Label::clamp_value("\tA Label  ")
            .as_ref()
            .map(Borrow::borrow)
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
