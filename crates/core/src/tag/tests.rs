// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use nonicle::CanonicalizeInto as _;

use super::*;

#[test]
fn default_plain_tag_score() {
    assert_eq!(PlainTag::default_score(), PlainTag::default().score);
}

#[test]
fn default_plain_tag_is_valid() {
    assert!(PlainTag::default().validate().is_ok());
}

#[test]
fn default_facet_key_str_equals_default_facet_id_str() {
    assert_eq!(FacetKey::default().as_str(), FacetId::default().as_str());
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
#[allow(clippy::too_many_lines)] // TODO
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
            score: Score::new(0.75),
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
