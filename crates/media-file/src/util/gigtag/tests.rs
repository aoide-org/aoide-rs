// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use nonicle::CanonicalizeInto as _;

use super::*;

fn label_from_str(label: &str) -> Label {
    gigtag::Label::from_str(label)
}

fn facet_from_str(facet: &str) -> Facet {
    gigtag::Facet::from_str(facet)
}

fn prop_name_from_str(name: &str) -> PropName {
    gigtag::Name::from_str(name)
}

fn score_prop_from_value(score_value: aoide_core::tag::ScoreValue) -> Property {
    Property {
        name: prop_name_from_str(SCORE_PROP_NAME),
        value: score_value.to_string().into(),
    }
}

enum LabelOrValue<'a> {
    Label(aoide_core::tag::Label<'a>),
    Value(Cow<'a, str>),
}

impl<'a> From<aoide_core::tag::Label<'a>> for LabelOrValue<'a> {
    fn from(label: aoide_core::tag::Label<'a>) -> Self {
        Self::Label(label)
    }
}

impl<'a> From<Cow<'a, str>> for LabelOrValue<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self::Value(value)
    }
}

impl From<String> for LabelOrValue<'_> {
    fn from(value: String) -> Self {
        Self::Value(value.into())
    }
}

impl<'a> From<&'a str> for LabelOrValue<'a> {
    fn from(value: &'a str) -> Self {
        Self::Value(value.into())
    }
}

fn plain_tag_with_label<'a>(label: impl Into<LabelOrValue<'a>>) -> PlainTag<'a> {
    let label = match label.into() {
        LabelOrValue::Label(label) => label,
        LabelOrValue::Value(value) => aoide_core::tag::Label::new(value),
    };
    PlainTag {
        label: Some(label),
        ..Default::default()
    }
}

fn plain_tag_with_label_and_score<'a>(
    label: impl Into<LabelOrValue<'a>>,
    score: aoide_core::tag::Score,
) -> PlainTag<'a> {
    PlainTag {
        score,
        ..plain_tag_with_label(label)
    }
}

#[test]
fn try_import_tags() {
    let label_value = "DJ";
    let date_like_facet = facet_from_str("@20220703");
    let score = aoide_core::tag::Score::new_unchecked(0.75);

    // Label
    let tag = Tag {
        label: label_from_str(label_value),
        ..Default::default()
    };
    let (facet_key, plain_tag) = try_import_tag(&tag).unwrap();
    assert!(facet_key.into_inner().is_none());
    assert_eq!(plain_tag_with_label(tag.label().to_string()), plain_tag);

    // Label + Facet
    let tag = Tag {
        facet: date_like_facet.clone(),
        ..tag
    };
    let (facet_key, plain_tag) = try_import_tag(&tag).unwrap();
    assert_eq!(
        Some(FacetId::from_unchecked(date_like_facet.as_ref())),
        facet_key.into_inner()
    );
    assert_eq!(plain_tag_with_label(tag.label().to_string()), plain_tag);

    // Label + Facet + Score
    let tag = Tag {
        props: vec![score_prop_from_value(score.value())],
        ..tag
    };
    let (facet_key, plain_tag) = try_import_tag(&tag).unwrap();
    assert_eq!(
        Some(FacetId::from_unchecked(date_like_facet.as_ref())),
        facet_key.into_inner()
    );
    assert_eq!(
        plain_tag_with_label_and_score(tag.label().to_string(), score),
        plain_tag
    );

    // Facet + Score
    let tag = Tag {
        label: Default::default(),
        ..tag
    };
    let (facet_key, plain_tag) = try_import_tag(&tag).unwrap();
    assert_eq!(
        Some(FacetId::from_unchecked(date_like_facet.as_ref())),
        facet_key.into_inner()
    );
    assert_eq!(
        PlainTag {
            score,
            ..Default::default()
        },
        plain_tag
    );
}

#[test]
fn try_import_tag_should_skip_invalid_tags() {
    assert!(try_import_tag(&Default::default()).is_none());
    assert!(
        try_import_tag(&Tag {
            props: vec![score_prop_from_value(0.75)],
            ..Default::default()
        })
        .is_none()
    );
}

#[test]
fn try_import_tag_should_skip_tags_with_invalid_score_values() {
    let tag = Tag {
        label: label_from_str("Label"),
        props: vec![score_prop_from_value(0.75)],
        ..Default::default()
    };
    assert!(try_import_tag(&tag).is_some());
    let tag = Tag {
        props: vec![score_prop_from_value(2.0)],
        ..tag
    };
    assert!(try_import_tag(&tag).is_none());
    let tag = Tag {
        props: vec![score_prop_from_value(-0.5)],
        ..tag
    };
    assert!(try_import_tag(&tag).is_none());
}

#[test]
fn try_import_tag_should_skip_tags_with_too_many_props() {
    let mut tag = Tag {
        label: label_from_str("Label"),
        props: vec![score_prop_from_value(0.75)],
        ..Default::default()
    };
    // Verify that the tag is imported with a single, expected property
    assert!(try_import_tag(&tag).is_some());
    // Verify that the tag is not imported when duplicating this property
    tag.props.push(tag.props.first().unwrap().clone());
    assert!(try_import_tag(&tag).is_none());
}

#[test]
fn try_import_tag_should_skip_tags_with_unknown_props() {
    let props = vec![score_prop_from_value(0.75)];
    let mut tag = Tag {
        label: label_from_str("Label"),
        props,
        ..Default::default()
    };
    // Verify that the tag is imported with the expected property name
    assert!(try_import_tag(&tag).is_some());
    // Verify that the tag is not imported with an unknown property name
    tag.props.first_mut().unwrap().name =
        prop_name_from_str(&format!("{SCORE_PROP_NAME}{SCORE_PROP_NAME}"));
    assert!(try_import_tag(&tag).is_none());
}

#[test]
fn reencode_roundtrip() {
    let encoded = "Some text\n facet@20220703#Tag2 ?name=value#TagWithUnsupportedProperties #Tag1";

    let mut encoded_label = aoide_core::tag::Label::clamp_from(encoded.to_string()).unwrap();
    let mut tags_map = TagsMap::default();
    let (retain, num_imported) =
        import_and_extract_tags_from_label_eagerly_into(&mut encoded_label, Some(&mut tags_map));
    assert!(retain);
    assert_eq!(2, num_imported);
    assert_eq!(tags_map.total_count(), num_imported);
    assert_eq!(
        "Some text\n ?name=value#TagWithUnsupportedProperties",
        encoded_label.as_str()
    );

    // Replace plain tag #Tag1 with #Tag2
    tags_map.replace_faceted_plain_tags(
        Default::default(),
        vec![plain_tag_with_label("Tag2".to_string())],
    );

    // Add #Tag2 with a non-date-like facet
    tags_map.replace_faceted_plain_tags(
        FacetId::from_unchecked("facet"),
        vec![plain_tag_with_label("Tag2".to_string())],
    );

    let mut reencoded = Cow::Borrowed(encoded);
    let tags = tags_map.canonicalize_into();
    assert!(update_tags_in_encoded(tags.as_canonical_ref(), &mut reencoded).is_ok());
    // Encoding implicitly reorders the tags
    assert_eq!(
        "Some text\n #Tag2 ?name=value#TagWithUnsupportedProperties facet#Tag2 facet@20220703#Tag2",
        reencoded
    );
}

#[test]
fn encode_decode_roundtrip_with_valid_tags() {
    let half_score =
        Score::clamp_from(Score::MIN.value() + (Score::MAX.value() - Score::MIN.value()) / 2.0);
    let mut tags_map = TagsMap::default();
    // Only a facet, no label, default score
    tags_map.insert(
        FacetKey::new(FacetId::clamp_from("facet_default_score")),
        Default::default(),
    );
    // Only a facet, no label, min. score
    tags_map.insert(
        FacetKey::new(FacetId::clamp_from("facet_min_score")),
        PlainTag {
            label: None,
            score: Score::MIN,
        },
    );
    // Only a facet, no label, max. score
    tags_map.insert(
        FacetKey::new(FacetId::clamp_from("facet_max_score")),
        PlainTag {
            label: None,
            score: Score::MAX,
        },
    );
    // Only a facet, no label, half score
    tags_map.insert(
        FacetKey::new(FacetId::clamp_from("facet_half_score")),
        PlainTag {
            label: None,
            score: half_score,
        },
    );
    // Only a label, no facet, default score
    tags_map.insert(
        FacetKey::new(None),
        plain_tag_with_label("Label with default score".to_string()),
    );
    // Only a label, no facet, min. score
    tags_map.insert(
        FacetKey::new(None),
        plain_tag_with_label_and_score("Label with min. score".to_string(), Score::MIN),
    );
    // Only a label, no facet, max. score
    tags_map.insert(
        FacetKey::new(None),
        plain_tag_with_label_and_score("Label with max. score".to_string(), Score::MAX),
    );
    // Only a label, no facet, half score
    tags_map.insert(
        FacetKey::new(None),
        plain_tag_with_label_and_score("Label with half score".to_string(), half_score),
    );
    // Both facet and label, default score
    tags_map.insert(
        FacetKey::new(FacetId::clamp_from("facet")),
        plain_tag_with_label("Label with default score".to_string()),
    );
    // Both facet and label, min. score
    tags_map.insert(
        FacetKey::new(FacetId::clamp_from("facet")),
        plain_tag_with_label_and_score("Label with min. score".to_string(), Score::MIN),
    );
    // Both facet and label, max. score
    tags_map.insert(
        FacetKey::new(FacetId::clamp_from("facet")),
        plain_tag_with_label_and_score("Label with max. score".to_string(), Score::MAX),
    );
    // Both facet and label, half score
    tags_map.insert(
        FacetKey::new(FacetId::clamp_from("facet")),
        plain_tag_with_label_and_score("Label with half score".to_string(), half_score),
    );
    let expected_count = tags_map.total_count();

    let tags = tags_map.canonicalize_into();
    assert!(tags.is_valid());
    assert_eq!(expected_count, tags.total_count());

    let mut encoded = Cow::Owned(String::new());
    assert!(update_tags_in_encoded(tags.as_canonical_ref(), &mut encoded).is_ok());
    println!("encoded = {encoded}");

    let mut decoded_tags = TagsMap::default();
    let (decoded, decoded_count) = decode_tags_eagerly_into(&encoded, Some(&mut decoded_tags));
    assert_eq!(decoded_count, decoded_tags.total_count());
    assert_eq!(expected_count, decoded_count);
    assert!(decoded.undecoded_prefix.is_empty());

    let decoded_tags = decoded_tags.canonicalize_into();
    assert_eq!(tags, decoded_tags);
}

#[test]
fn encode_date_like_facet_should_omit_default_score() {
    let mut tags_map = TagsMap::default();
    tags_map.insert(
        FacetKey::new(FacetId::clamp_from("@20230330")),
        Default::default(),
    );
    let tags = tags_map.canonicalize_into();
    assert!(tags.is_valid());

    let mut encoded = Cow::Owned(String::new());
    assert!(update_tags_in_encoded(tags.as_canonical_ref(), &mut encoded).is_ok());
    println!("encoded = {encoded}");
    assert_eq!("@20230330", encoded);
}
