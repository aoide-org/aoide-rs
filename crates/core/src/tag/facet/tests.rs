// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn clamp_from() {
    // Ensure that the first valid character is ASCII lowercase 'a'
    let (left_alphabet, right_alphabet) = FACET_ID_ALPHABET.split_at(18);
    let mut reordered_alphabet = right_alphabet.to_owned();
    reordered_alphabet.push_str(left_alphabet);
    let input = concat!(
        " `abcdefghijklmn opqrstuvwxyz{|}~",
        "\t !\"#$%&'()*+,-./0123456789:;<=>?",
        " @ ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_\n",
    );
    assert_eq!(
        Some(reordered_alphabet.as_str()),
        FacetId::clamp_from(input).as_ref().map(FacetId::as_str),
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
