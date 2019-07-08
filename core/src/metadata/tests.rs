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

use super::*;

///////////////////////////////////////////////////////////////////////

#[test]
fn score_valid() {
    assert!(Score::min().validate().is_ok());
    assert!(Score::max().validate().is_ok());
    assert!(Score::new(Score::min().0 + Score::max().0)
        .validate()
        .is_ok());
    assert!(!Score::new(Score::min().0 - Score::max().0)
        .validate()
        .is_ok());
    assert!(!Score::new(Score::max().0 + Score::max().0)
        .validate()
        .is_ok());
}

#[test]
fn score_display() {
    assert_eq!("0.0%", format!("{}", Score::min()));
    assert_eq!("100.0%", format!("{}", Score::max()));
    assert_eq!("90.1%", format!("{}", Score(0.901_234_5)));
    assert_eq!("90.2%", format!("{}", Score(0.901_5)));
}

#[test]
fn parse_label() {
    assert_eq!(
        Ok(Label::new("A Label".into())),
        "\tA Label  ".parse::<Label>()
    );
}

#[test]
fn validate_label() {
    assert!(Label::new("A Term".into()).validate().is_ok());
    assert!(Label::new("\tA Term  ".into()).validate().is_err());
}

#[test]
fn parse_facet() {
    assert_eq!(
        Ok(Facet::new("a_facet".into())),
        "\tA Facet  ".parse::<Facet>()
    );
}

#[test]
fn validate_facet() {
    assert!(Facet::new("a_facet".into()).validate().is_ok());
    assert!(Facet::new("a facet".into()).validate().is_err());
    assert!(Facet::new("\tA facet  ".into()).validate().is_err());
}
