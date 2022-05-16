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

use crate::util::canonical::CanonicalizeInto as _;

use super::*;

#[test]
fn score_valid() {
    assert!(Score::min().validate().is_ok());
    assert!(Score::max().validate().is_ok());
    assert!(Score::new(Score::min().0 + Score::max().0)
        .validate()
        .is_ok());
    assert!(Score::new(Score::min().0 - Score::max().0)
        .validate()
        .is_err());
    assert!(Score::new(Score::max().0 + Score::max().0)
        .validate()
        .is_err());
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
        Some(Label::new("A Label".into())),
        Label::clamp_from("A Label")
    );
}

#[test]
fn clamp_label_value() {
    assert_eq!(
        Some("A Label"),
        Label::clamp_value("\tA Label  ")
            .as_ref()
            .map(Borrow::borrow)
    );
}

#[test]
fn clamp_facet_value() {
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
fn validate_label() {
    assert!(Label::new("A Term".into()).validate().is_ok());
    assert!(Label::new("\tA Term  ".into()).validate().is_err());
}

#[test]
fn validate_facet() {
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
fn default_facet_is_invalid() {
    assert!(FacetId::default().validate().is_err());
}

#[test]
fn empty_facet_is_invalid() {
    assert!(FacetId::new("".into()).validate().is_err());
}

#[test]
fn parse_empty_facet_id() {
    assert!(FacetId::clamp_from("").is_none());
}

#[test]
fn default_label_is_invalid() {
    assert!(Label::default().validate().is_err());
}

#[test]
fn parse_empty_label() {
    assert!(Label::clamp_from("").is_none());
}

#[test]
fn default_plain_tag_score() {
    assert_eq!(PlainTag::default_score(), PlainTag::default().score);
}

#[test]
fn default_plain_tag_is_valid() {
    assert!(PlainTag::default().validate().is_ok());
}

#[test]
fn canonical_unique_labels_and_score() {
    let tags = Tags {
        plain: vec![
            PlainTag {
                label: Some(Label::new("label1".into())),
                score: 0.5.into(),
            },
            PlainTag {
                label: Some(Label::new("label2".into())),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    assert!(tags.is_canonical());
    assert!(tags.validate().is_ok());
}

#[test]
fn duplicate_labels_same_score() {
    let tags = Tags {
        plain: vec![
            PlainTag {
                label: Some(Label::new("label1".into())),
                score: 0.5.into(),
            },
            PlainTag {
                label: Some(Label::new("label2".into())),
                ..Default::default()
            },
            PlainTag {
                label: Some(Label::new("label1".into())),
                score: 0.5.into(),
            },
        ],
        ..Default::default()
    };
    assert!(!tags.is_canonical());
    assert!(tags.canonicalize_into().validate().is_ok());
}

#[test]
fn duplicate_labels_differing_score() {
    let tags = Tags {
        plain: vec![
            PlainTag {
                label: Some(Label::new("label1".into())),
                score: 0.7.into(),
            },
            PlainTag {
                label: Some(Label::new("label2".into())),
                ..Default::default()
            },
            PlainTag {
                label: Some(Label::new("label1".into())),
                score: 0.5.into(),
            },
        ],
        ..Default::default()
    };
    assert!(!tags.is_canonical());
    let tags = tags.canonicalize_into();
    assert_eq!(2, tags.total_count());
}

#[test]
fn canonical_faceted_tags() {
    let tags = Tags {
        plain: vec![
            PlainTag {
                label: Some(Label::new("label1".into())),
                ..Default::default()
            },
            PlainTag {
                label: Some(Label::new("label2".into())),
                ..Default::default()
            },
        ],
        facets: vec![
            FacetedTags {
                facet_id: FacetId::new("facet1".into()),
                tags: vec![PlainTag {
                    label: Some(Label::new("label1".into())),
                    ..Default::default()
                }],
            },
            FacetedTags {
                facet_id: FacetId::new("facet2".into()),
                tags: vec![PlainTag {
                    label: Some(Label::new("label1".into())),
                    ..Default::default()
                }],
            },
        ],
    };
    assert!(tags.is_canonical());
    assert!(tags.validate().is_ok());
}

#[test]
fn duplicate_facets() {
    let mut tags = Tags {
        plain: vec![
            PlainTag {
                label: Some(Label::new("label2".into())),
                ..Default::default()
            },
            PlainTag {
                label: Some(Label::new("label1".into())),
                ..Default::default()
            },
            PlainTag {
                label: Some(Label::new("label2".into())),
                ..Default::default()
            },
        ],
        facets: vec![
            FacetedTags {
                facet_id: FacetId::new("facet1".into()),
                tags: vec![PlainTag {
                    label: Some(Label::new("label1".into())),
                    ..Default::default()
                }],
            },
            FacetedTags {
                facet_id: FacetId::new("facet2".into()),
                tags: vec![PlainTag {
                    label: Some(Label::new("label1".into())),
                    ..Default::default()
                }],
            },
            FacetedTags {
                facet_id: FacetId::new("facet1".into()),
                tags: vec![PlainTag {
                    label: Some(Label::new("label2".into())),
                    ..Default::default()
                }],
            },
        ],
    };
    tags.canonicalize();
    assert_eq!(5, tags.total_count());
    assert!(tags.validate().is_ok());
}

#[test]
#[allow(clippy::too_many_lines)]
fn duplicate_facets_and_labels() {
    let mut tags = Tags {
        plain: vec![
            PlainTag {
                label: Some(Label::new("label1".into())),
                ..Default::default()
            },
            PlainTag {
                label: Some(Label::new("label2".into())),
                ..Default::default()
            },
        ],
        facets: vec![
            FacetedTags {
                facet_id: FacetId::new("facet1".into()),
                tags: vec![PlainTag {
                    label: Some(Label::new("label1".into())),
                    ..Default::default()
                }],
            },
            FacetedTags {
                facet_id: FacetId::new("facet2".into()),
                tags: vec![
                    PlainTag {
                        label: Some(Label::new("label2".into())),
                        score: 0.5.into(),
                    },
                    PlainTag {
                        label: Some(Label::new("label2".into())),
                        score: 1.0.into(),
                    },
                ],
            },
        ],
    };
    tags.canonicalize();
    assert_eq!(4, tags.total_count());

    let mut tags = Tags {
        plain: vec![
            PlainTag {
                label: Some(Label::new("label2".into())),
                ..Default::default()
            },
            PlainTag {
                label: Some(Label::new("label1".into())),
                ..Default::default()
            },
            PlainTag {
                label: Some(Label::new("label2".into())),
                ..Default::default()
            },
        ],
        facets: vec![
            FacetedTags {
                facet_id: FacetId::new("facet1".into()),
                tags: vec![
                    PlainTag {
                        label: Some(Label::new("label1".into())),
                        ..Default::default()
                    },
                    PlainTag {
                        label: Some(Label::new("label2".into())),
                        ..Default::default()
                    },
                    PlainTag {
                        label: Some(Label::new("label1".into())),
                        ..Default::default()
                    },
                    PlainTag {
                        label: Some(Label::new("label2".into())),
                        ..Default::default()
                    },
                ],
            },
            FacetedTags {
                facet_id: FacetId::new("facet2".into()),
                tags: vec![
                    PlainTag {
                        label: Some(Label::new("label2".into())),
                        score: 0.5.into(),
                    },
                    PlainTag {
                        label: Some(Label::new("label2".into())),
                        score: 0.75.into(),
                    },
                    PlainTag {
                        label: Some(Label::new("label2".into())),
                        score: 0.25.into(),
                    },
                ],
            },
        ],
    };
    tags.canonicalize();
    assert!(tags.validate().is_ok());
    assert_eq!(5, tags.total_count());
    assert!(tags.facets.contains(&FacetedTags {
        facet_id: FacetId::new("facet2".into()),
        tags: vec![PlainTag {
            label: Some(Label::new("label2".into())),
            score: Score(0.75),
        },],
    }));
}

#[test]
fn canonicalize_should_remove_facets_without_tags() {
    let expected_tags = Tags {
        plain: vec![],
        facets: vec![
            FacetedTags {
                facet_id: FacetId::new("facet1".into()),
                tags: vec![
                    PlainTag {
                        label: Some(Label::new("label1".into())),
                        ..Default::default()
                    },
                    PlainTag {
                        label: Some(Label::new("label2".into())),
                        ..Default::default()
                    },
                ],
            },
            FacetedTags {
                facet_id: FacetId::new("facet3".into()),
                tags: vec![PlainTag {
                    label: Some(Label::new("label1".into())),
                    ..Default::default()
                }],
            },
        ],
    };
    assert!(expected_tags.is_canonical());

    let mut actual_tags = expected_tags.clone();
    actual_tags.facets.insert(
        1,
        FacetedTags {
            facet_id: FacetId::new("facet2".into()),
            tags: vec![],
        },
    );
    actual_tags.facets.insert(
        2,
        FacetedTags {
            facet_id: FacetId::new("facet3".into()),
            tags: vec![],
        },
    );
    actual_tags.facets.insert(
        3,
        FacetedTags {
            facet_id: FacetId::new("facet3".into()),
            tags: vec![PlainTag {
                label: Some(Label::new("label1".into())),
                ..Default::default()
            }],
        },
    );
    assert!(!actual_tags.is_canonical());

    actual_tags.canonicalize();
    assert!(actual_tags.is_canonical());
    assert_eq!(expected_tags, actual_tags);
}
