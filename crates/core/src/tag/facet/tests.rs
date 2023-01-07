// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn clamp_from() {
    assert_eq!(
        Some(FACET_ID_ALPHABET),
        FacetId::clamp_from(FACET_ID_ALPHABET)
            .as_ref()
            .map(Borrow::borrow)
    );
    assert_eq!(
        Some(concat!(
            "+-./",
            "0123456789",
            "@[]_",
            "abcdefghijklmnopqrstuvwxyz~",
        )),
        FacetId::clamp_from(concat!(
            "\t !\"#$%&'()*+,-./0123456789:;<=>?",
            " @ ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_",
            " `abcdefghijklmn opqrstuvwxyz{|}~\n"
        ))
        .as_ref()
        .map(Borrow::borrow)
    );
}

#[test]
fn validate() {
    // FACET_ID_ALPHABET Does not start with a lowercase ASCII letter
    // but ends with '~' which is a valid first character.
    let reverse_alphabet: String = FACET_ID_ALPHABET.chars().rev().collect();
    assert!(FacetId::new(reverse_alphabet.into()).validate().is_ok());
    assert!(FacetId::new(FACET_ID_ALPHABET.into()).validate().is_err());
    assert!(FacetId::new("Facet".into()).validate().is_err());
    assert!(FacetId::new("a facet".into()).validate().is_err());
}

#[test]
fn default_is_invalid() {
    assert!(FacetId::default().validate().is_err());
}

#[test]
fn empty_is_invalid() {
    assert!(FacetId::new("".into()).validate().is_err());
}

#[test]
fn parse_empty() {
    assert!(FacetId::clamp_from("").is_none());
}
