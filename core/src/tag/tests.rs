// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

///////////////////////////////////////////////////////////////////////

use super::*;

#[test]
fn default_facet_is_invalid() {
    assert!(Facet::default().validate().is_err());
}

#[test]
fn empty_facet_is_invalid() {
    assert!(Facet::from("").validate().is_err());
    assert!("".parse::<Facet>().unwrap().validate().is_err());
}

#[test]
fn default_label_is_invalid() {
    assert!(Label::default().validate().is_err());
}

#[test]
fn empty_label_is_invalid() {
    assert!(Label::from("").validate().is_err());
    assert!("".parse::<Label>().unwrap().validate().is_err());
}

#[test]
fn default_tag_score() {
    assert_eq!(Tag::default_score(), Tag::default().score);
}

#[test]
fn default_tag_is_invalid() {
    assert!(Tag::default().validate().is_err());
}
