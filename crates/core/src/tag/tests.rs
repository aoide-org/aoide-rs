// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn default_plain_tag_score() {
    assert_eq!(PlainTag::DEFAULT_SCORE, PlainTag::default().score);
}

#[test]
fn default_plain_tag_is_valid() {
    assert!(PlainTag::default().validate().is_ok());
}

#[test]
fn default_facet_key_str_equals_default_facet_id_str() {
    assert_eq!(FacetKey::unfaceted().as_str(), FacetId::default().as_str());
}

#[test]
fn canonical_unique_labels_and_score() {
    let tags = Tags {
        plain: vec![
            PlainTag {
                label: Some(Label::from_unchecked("label1")),
                score: Score::new_unchecked(0.5),
            },
            PlainTag {
                label: Some(Label::from_unchecked("label2")),
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
                label: Some(Label::from_unchecked("label1")),
                score: Score::new_unchecked(0.5),
            },
            PlainTag {
                label: Some(Label::from_unchecked("label2")),
                ..Default::default()
            },
            PlainTag {
                label: Some(Label::from_unchecked("label1")),
                score: Score::new_unchecked(0.5),
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
                label: Some(Label::from_unchecked("label1")),
                score: Score::new_unchecked(0.7),
            },
            PlainTag {
                label: Some(Label::from_unchecked("label2")),
                ..Default::default()
            },
            PlainTag {
                label: Some(Label::from_unchecked("label1")),
                score: Score::new_unchecked(0.5),
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
                label: Some(Label::from_unchecked("label1")),
                ..Default::default()
            },
            PlainTag {
                label: Some(Label::from_unchecked("label2")),
                ..Default::default()
            },
        ],
        facets: vec![
            FacetedTags {
                facet_id: FacetId::from_unchecked("facet1"),
                tags: vec![PlainTag {
                    label: Some(Label::from_unchecked("label1")),
                    ..Default::default()
                }],
            },
            FacetedTags {
                facet_id: FacetId::from_unchecked("facet2"),
                tags: vec![PlainTag {
                    label: Some(Label::from_unchecked("label1")),
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
                label: Some(Label::from_unchecked("label2")),
                ..Default::default()
            },
            PlainTag {
                label: Some(Label::from_unchecked("label1")),
                ..Default::default()
            },
            PlainTag {
                label: Some(Label::from_unchecked("label2")),
                ..Default::default()
            },
        ],
        facets: vec![
            FacetedTags {
                facet_id: FacetId::from_unchecked("facet1"),
                tags: vec![PlainTag {
                    label: Some(Label::from_unchecked("label1")),
                    ..Default::default()
                }],
            },
            FacetedTags {
                facet_id: FacetId::from_unchecked("facet2"),
                tags: vec![PlainTag {
                    label: Some(Label::from_unchecked("label1")),
                    ..Default::default()
                }],
            },
            FacetedTags {
                facet_id: FacetId::from_unchecked("facet1"),
                tags: vec![PlainTag {
                    label: Some(Label::from_unchecked("label2")),
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
#[expect(clippy::too_many_lines)] // TODO
fn duplicate_facets_and_labels() {
    let mut tags = Tags {
        plain: vec![
            PlainTag {
                label: Some(Label::from_unchecked("label1")),
                ..Default::default()
            },
            PlainTag {
                label: Some(Label::from_unchecked("label2")),
                ..Default::default()
            },
        ],
        facets: vec![
            FacetedTags {
                facet_id: FacetId::from_unchecked("facet1"),
                tags: vec![PlainTag {
                    label: Some(Label::from_unchecked("label1")),
                    ..Default::default()
                }],
            },
            FacetedTags {
                facet_id: FacetId::from_unchecked("facet2"),
                tags: vec![
                    PlainTag {
                        label: Some(Label::from_unchecked("label2")),
                        score: Score::new_unchecked(0.5),
                    },
                    PlainTag {
                        label: Some(Label::from_unchecked("label2")),
                        score: Score::new_unchecked(1.0),
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
                label: Some(Label::from_unchecked("label2")),
                ..Default::default()
            },
            PlainTag {
                label: Some(Label::from_unchecked("label1")),
                ..Default::default()
            },
            PlainTag {
                label: Some(Label::from_unchecked("label2")),
                ..Default::default()
            },
        ],
        facets: vec![
            FacetedTags {
                facet_id: FacetId::from_unchecked("facet1"),
                tags: vec![
                    PlainTag {
                        label: Some(Label::from_unchecked("label1")),
                        ..Default::default()
                    },
                    PlainTag {
                        label: Some(Label::from_unchecked("label2")),
                        ..Default::default()
                    },
                    PlainTag {
                        label: Some(Label::from_unchecked("label1")),
                        ..Default::default()
                    },
                    PlainTag {
                        label: Some(Label::from_unchecked("label2")),
                        ..Default::default()
                    },
                ],
            },
            FacetedTags {
                facet_id: FacetId::from_unchecked("facet2"),
                tags: vec![
                    PlainTag {
                        label: Some(Label::from_unchecked("label2")),
                        score: Score::new_unchecked(0.5),
                    },
                    PlainTag {
                        label: Some(Label::from_unchecked("label2")),
                        score: Score::new_unchecked(0.75),
                    },
                    PlainTag {
                        label: Some(Label::from_unchecked("label2")),
                        score: Score::new_unchecked(0.25),
                    },
                ],
            },
        ],
    };
    tags.canonicalize();
    assert!(tags.validate().is_ok());
    assert_eq!(5, tags.total_count());
    assert!(tags.facets.contains(&FacetedTags {
        facet_id: FacetId::from_unchecked("facet2"),
        tags: vec![PlainTag {
            label: Some(Label::from_unchecked("label2")),
            score: Score::new_unchecked(0.75),
        },],
    }));
}

#[test]
fn canonicalize_should_remove_facets_without_tags() {
    let expected_tags = Tags {
        plain: vec![],
        facets: vec![
            FacetedTags {
                facet_id: FacetId::from_unchecked("facet1"),
                tags: vec![
                    PlainTag {
                        label: Some(Label::from_unchecked("label1")),
                        ..Default::default()
                    },
                    PlainTag {
                        label: Some(Label::from_unchecked("label2")),
                        ..Default::default()
                    },
                ],
            },
            FacetedTags {
                facet_id: FacetId::from_unchecked("facet3"),
                tags: vec![PlainTag {
                    label: Some(Label::from_unchecked("label1")),
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
            facet_id: FacetId::from_unchecked("facet2"),
            tags: vec![],
        },
    );
    actual_tags.facets.insert(
        2,
        FacetedTags {
            facet_id: FacetId::from_unchecked("facet3"),
            tags: vec![],
        },
    );
    actual_tags.facets.insert(
        3,
        FacetedTags {
            facet_id: FacetId::from_unchecked("facet3"),
            tags: vec![PlainTag {
                label: Some(Label::from_unchecked("label1")),
                ..Default::default()
            }],
        },
    );
    assert!(!actual_tags.is_canonical());

    actual_tags.canonicalize();
    assert!(actual_tags.is_canonical());
    assert_eq!(expected_tags, actual_tags);
}
