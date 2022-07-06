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

use aoide_core::util::canonical::CanonicalizeInto;

use super::*;

fn label_from_str(label: &str) -> Label {
    gigtags::label::Label::from_str(label)
}

fn facet_from_str(facet: &str) -> Facet {
    gigtags::facet::Facet::from_str(facet)
}

fn prop_name_from_str(name: &str) -> PropName {
    gigtags::props::Name::from_str(name)
}

fn score_prop_from_value(score_value: aoide_core::tag::ScoreValue) -> Property {
    Property {
        name: prop_name_from_str(SCORE_PROP_NAME),
        value: score_value.to_string().into(),
    }
}

fn plain_tag_with_label(label: impl Into<aoide_core::tag::Label>) -> PlainTag {
    PlainTag {
        label: Some(label.into()),
        ..Default::default()
    }
}

fn plain_tag_with_label_and_score(
    label: impl Into<aoide_core::tag::Label>,
    score: impl Into<aoide_core::tag::Score>,
) -> PlainTag {
    PlainTag {
        score: score.into(),
        ..plain_tag_with_label(label)
    }
}

#[test]
fn try_import_tags() {
    let label_value = "DJ";
    let date_like_facet = facet_from_str("~20220703");
    let score_value = 0.75;

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
        Some(FacetId::new(date_like_facet.as_ref().into())),
        facet_key.into_inner()
    );
    assert_eq!(plain_tag_with_label(tag.label().to_string()), plain_tag);

    // Label + Facet + Score
    let tag = Tag {
        props: vec![score_prop_from_value(score_value)],
        ..tag
    };
    let (facet_key, plain_tag) = try_import_tag(&tag).unwrap();
    assert_eq!(
        Some(FacetId::new(date_like_facet.as_ref().into())),
        facet_key.into_inner()
    );
    assert_eq!(
        plain_tag_with_label_and_score(tag.label().to_string(), score_value),
        plain_tag
    );

    // Facet + Score
    let tag = Tag {
        label: Default::default(),
        ..tag
    };
    let (facet_key, plain_tag) = try_import_tag(&tag).unwrap();
    assert_eq!(
        Some(FacetId::new(date_like_facet.as_ref().into())),
        facet_key.into_inner()
    );
    assert_eq!(
        PlainTag {
            score: score_value.into(),
            ..Default::default()
        },
        plain_tag
    );
}

#[test]
fn try_import_tag_should_skip_invalid_tags() {
    assert!(try_import_tag(&Default::default()).is_none());
    assert!(try_import_tag(&Tag {
        props: vec![score_prop_from_value(0.75)],
        ..Default::default()
    })
    .is_none());
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
        prop_name_from_str(&format!("{}{}", SCORE_PROP_NAME, SCORE_PROP_NAME));
    assert!(try_import_tag(&tag).is_none());
}

#[test]
fn reencode_roundtrip() {
    let encoded = "Some text\n facet~20220703#Tag2 ?name=value#TagWithUnsupportedProperties #Tag1";

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

    // Replace #Tag1 with #Tag3
    tags_map.replace_faceted_plain_tags(
        Default::default(),
        vec![plain_tag_with_label("Tag3".to_string())],
    );

    let mut reencoded = encoded.to_string();
    assert!(update_tags_in_encoded(&tags_map.into(), &mut reencoded).is_ok());
    assert_eq!(
        "Some text\n ?name=value#TagWithUnsupportedProperties #Tag3 facet~20220703#Tag2",
        reencoded
    );
}

#[test]
fn encode_decode_roundtrip_with_valid_tags() {
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
            score: Score::min(),
        },
    );
    // Only a facet, no label, max. score
    tags_map.insert(
        FacetKey::new(FacetId::clamp_from("facet_max_score")),
        PlainTag {
            label: None,
            score: Score::max(),
        },
    );
    // Only a label, no facet, default score
    tags_map.insert(
        FacetKey::new(None),
        plain_tag_with_label("label_default_score".to_string()),
    );
    // Only a label, no facet, min. score
    tags_map.insert(
        FacetKey::new(None),
        plain_tag_with_label_and_score("label_min_score".to_string(), Score::min()),
    );
    // Only a label, no facet, max. score
    tags_map.insert(
        FacetKey::new(None),
        plain_tag_with_label_and_score("label_max_score".to_string(), Score::max()),
    );
    // Both facet and label, default score
    tags_map.insert(
        FacetKey::new(FacetId::clamp_from("facet")),
        plain_tag_with_label("label_default_score".to_string()),
    );
    // Both facet and label, min. score
    tags_map.insert(
        FacetKey::new(FacetId::clamp_from("facet")),
        plain_tag_with_label_and_score("label_min_score".to_string(), Score::min()),
    );
    // Both facet and label, max. score
    tags_map.insert(
        FacetKey::new(FacetId::clamp_from("facet")),
        plain_tag_with_label_and_score("label_max_score".to_string(), Score::max()),
    );
    let expected_count = tags_map.total_count();

    let tags: Tags = tags_map.into();
    let tags = tags.canonicalize_into();
    assert!(tags.is_valid());
    assert_eq!(expected_count, tags.total_count());

    let mut encoded = String::new();
    assert!(update_tags_in_encoded(&tags, &mut encoded).is_ok());
    println!("encoded = {encoded}");

    let mut tags_map = TagsMap::default();
    let (decoded, decoded_count) = decode_tags_eagerly_into(&encoded, Some(&mut tags_map));
    assert_eq!(decoded_count, tags_map.total_count());
    assert_eq!(expected_count, decoded_count);
    assert!(decoded.undecoded_prefix.is_empty());

    let decoded_tags: Tags = tags_map.into();
    let decoded_tags = decoded_tags.canonicalize_into();
    assert_eq!(tags, decoded_tags);
}
