// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn clamp_from() {
    assert_eq!(
        Some(FACET_ID_ALPHABET),
        FacetId::clamp_from(FACET_ID_ALPHABET)
            .as_ref()
            .map(FacetId::as_str),
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
        .map(FacetId::as_str)
    );
}

#[test]
fn validate() {
    let reverse_alphabet: String = FACET_ID_ALPHABET.chars().rev().collect();
    assert!(FacetId::new_unchecked(reverse_alphabet.into())
        .validate()
        .is_ok());
    assert!(FacetId::new_unchecked(FACET_ID_ALPHABET.into())
        .validate()
        .is_ok());
    assert!(FacetId::new_unchecked("Facet".into()).validate().is_err());
    assert!(FacetId::new_unchecked("a facet".into()).validate().is_err());
}

#[test]
fn default_is_invalid() {
    assert!(FacetId::default().validate().is_err());
}

#[test]
fn empty_is_invalid() {
    assert!(FacetId::new_unchecked("".into()).validate().is_err());
}

#[test]
fn parse_empty() {
    assert!(FacetId::clamp_from("").is_none());
}
