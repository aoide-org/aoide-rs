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
fn clamp_value() {
    assert_eq!(
        Some(FACET_ID_ALPHABET),
        FacetId::clamp_value(FACET_ID_ALPHABET)
            .as_ref()
            .map(Borrow::borrow)
    );
    assert_eq!(
        Some(concat!(
            "+-./",
            "0123456789",
            "@[]_",
            "abcdefghijklmnopqrstuvwxyz",
        )),
        FacetId::clamp_value(concat!(
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
    // but ends with one.
    let reverse_alphabet: String = FACET_ID_ALPHABET.chars().rev().collect();
    assert!(FacetId::new(reverse_alphabet).validate().is_ok());
    assert!(FacetId::new(FACET_ID_ALPHABET.to_owned())
        .validate()
        .is_err());
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
