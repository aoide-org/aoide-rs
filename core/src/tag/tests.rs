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
    assert_eq!(Ok(Label::new("A Label")), "\tA Label  ".parse::<Label>());
}

#[test]
fn validate_label() {
    assert!(Label::new("A Term").validate().is_ok());
    assert!(Label::new("\tA Term  ").validate().is_err());
}

#[test]
fn parse_facet() {
    assert_eq!(Ok(Facet::new("a_facet")), "\tA Facet  ".parse::<Facet>());
}

#[test]
fn validate_facet() {
    assert!(Facet::new("a_facet").validate().is_ok());
    assert!(Facet::new("a facet").validate().is_err());
    assert!(Facet::new("\tA facet  ").validate().is_err());
}

#[test]
fn default_facet_is_invalid() {
    assert!(Facet::default().validate().is_err());
}

#[test]
fn empty_facet_is_invalid() {
    assert!(Facet::new("").validate().is_err());
    assert!("".parse::<Facet>().unwrap().validate().is_err());
}

#[test]
fn default_label_is_invalid() {
    assert!(Label::default().validate().is_err());
}

#[test]
fn empty_label_is_invalid() {
    assert!(Label::new("").validate().is_err());
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

#[test]
fn duplicate_labels() {
    let tags = [
        Tag {
            facet: None,
            label: Some(Label::new("label1")),
            ..Default::default()
        },
        Tag {
            facet: None,
            label: Some(Label::new("label2")),
            ..Default::default()
        },
    ];
    assert!(Tags::validate(tags.iter()).is_ok());

    let tags = [
        Tag {
            facet: None,
            label: Some(Label::new("label1")),
            ..Default::default()
        },
        Tag {
            facet: None,
            label: Some(Label::new("label2")),
            ..Default::default()
        },
        Tag {
            facet: None,
            label: Some(Label::new("label1")),
            ..Default::default()
        },
    ];
    assert_eq!(
        1,
        Tags::validate(tags.iter())
            .err()
            .unwrap()
            .into_iter()
            .count()
    );

    let tags = [
        Tag {
            facet: Some(Facet::new("facet1")),
            label: Some(Label::new("label1")),
            ..Default::default()
        },
        Tag {
            facet: None,
            label: Some(Label::new("label2")),
            ..Default::default()
        },
        Tag {
            facet: Some(Facet::new("facet2")),
            label: Some(Label::new("label2")),
            ..Default::default()
        },
        Tag {
            facet: None,
            label: Some(Label::new("label1")),
            ..Default::default()
        },
    ];
    assert!(Tags::validate(tags.iter()).is_ok());

    let tags = [
        Tag {
            facet: Some(Facet::new("facet1")),
            label: Some(Label::new("label1")),
            ..Default::default()
        },
        Tag {
            facet: None,
            label: Some(Label::new("label2")),
            ..Default::default()
        },
        Tag {
            facet: None,
            label: Some(Label::new("label2")),
            ..Default::default()
        },
        Tag {
            facet: Some(Facet::new("facet2")),
            label: Some(Label::new("label2")),
            ..Default::default()
        },
        Tag {
            facet: None,
            label: Some(Label::new("label1")),
            ..Default::default()
        },
    ];
    assert_eq!(
        1,
        Tags::validate(tags.iter())
            .err()
            .unwrap()
            .into_iter()
            .count()
    );

    let tags = [
        Tag {
            facet: Some(Facet::new("facet1")),
            label: Some(Label::new("label1")),
            ..Default::default()
        },
        Tag {
            facet: Some(Facet::new("facet2")),
            label: Some(Label::new("label2")),
            ..Default::default()
        },
        Tag {
            facet: None,
            label: Some(Label::new("label2")),
            ..Default::default()
        },
        Tag {
            facet: Some(Facet::new("facet2")),
            label: Some(Label::new("label2")),
            ..Default::default()
        },
        Tag {
            facet: None,
            label: Some(Label::new("label1")),
            ..Default::default()
        },
    ];
    assert_eq!(
        1,
        Tags::validate(tags.iter())
            .err()
            .unwrap()
            .into_iter()
            .count()
    );

    let tags = [
        Tag {
            facet: Some(Facet::new("facet1")),
            label: Some(Label::new("label1")),
            ..Default::default()
        },
        Tag {
            facet: Some(Facet::new("facet2")),
            label: Some(Label::new("label2")),
            ..Default::default()
        },
        Tag {
            facet: None,
            label: Some(Label::new("label2")),
            ..Default::default()
        },
        Tag {
            facet: None,
            label: Some(Label::new("label2")),
            ..Default::default()
        },
        Tag {
            facet: Some(Facet::new("facet2")),
            label: Some(Label::new("label2")),
            ..Default::default()
        },
        Tag {
            facet: None,
            label: Some(Label::new("label1")),
            ..Default::default()
        },
    ];
    assert_eq!(
        2,
        Tags::validate(tags.iter())
            .err()
            .unwrap()
            .into_iter()
            .count()
    );
}
