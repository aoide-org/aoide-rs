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
fn deserialize_tag_label() {
    let label = _core::Label::new("label");
    let json = format!("\"{}\"", label);
    let tag: PlainTag = serde_json::from_str(&json).unwrap();
    assert_eq!(PlainTag::Label(label.into()), tag);
    assert_eq!(json, serde_json::to_string(&tag).unwrap());
}

#[test]
fn should_fail_to_deserialize_single_element_array_with_label() {
    let label = _core::Label::new("label");
    let json = format!("[\"{}\"]", label);
    assert!(serde_json::from_str::<PlainTag>(&json).is_err());
}

#[test]
fn deserialize_tag_label_score() {
    let label = _core::Label::new("label");
    let score = _core::Score::new(0.5);
    let json = format!("[\"{}\",{}]", label, f64::from(score));
    let tag: PlainTag = serde_json::from_str(&json).unwrap();
    assert_eq!(PlainTag::LabelScore(label.into(), score.into()), tag);
    assert_eq!(json, serde_json::to_string(&tag).unwrap());
}

#[test]
fn deserialize_tag_label_score_zero() {
    let expected_tag = _core::Tag {
        label: Some(_core::Label::new("label")),
        score: _core::Score::new(0.0),
        ..Default::default()
    };
    // Ensure to parse score from literal 0, not 0.0!
    let json = format!("[\"{}\",0]", expected_tag.label.as_ref().unwrap());
    let parsed_tag: PlainTag = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&parsed_tag).unwrap());
    assert_eq!(expected_tag, parsed_tag.into());
}

#[test]
fn deserialize_tag_label_score_one() {
    let expected_tag = _core::Tag {
        label: Some(_core::Label::new("label")),
        score: _core::Score::new(1.0),
        ..Default::default()
    };
    // Ensure to parse score from literal 1, not 1.0!
    let json = format!("[\"{}\",1]", expected_tag.label.as_ref().unwrap());
    let parsed_tag: PlainTag = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&parsed_tag).unwrap());
    assert_eq!(expected_tag, parsed_tag.into());
}

#[test]
fn deserialize_tag_facet() {
    let facet = _core::Facet::new("facet");
    let json = format!("\"{}\"", facet);
    let tag: FacetedTag = serde_json::from_str(&json).unwrap();
    assert_eq!(FacetedTag::Facet(facet.into()), tag);
    assert_eq!(json, serde_json::to_string(&tag).unwrap());
}

#[test]
fn should_fail_to_deserialize_single_element_array_with_facet() {
    let facet = _core::Facet::new("facet");
    let json = format!("[\"{}\"]", facet);
    assert!(serde_json::from_str::<FacetedTag>(&json).is_err());
}

#[test]
fn deserialize_tag_facet_score() {
    let facet = _core::Facet::new("facet");
    let score = _core::Score::new(0.5);
    let json = format!("[\"{}\",{}]", facet, f64::from(score));
    let tag: FacetedTag = serde_json::from_str(&json).unwrap();
    assert_eq!(FacetedTag::FacetScore(facet.into(), score.into()), tag);
    assert_eq!(json, serde_json::to_string(&tag).unwrap());
}

#[test]
fn deserialize_tag_facet_label() {
    let facet = _core::Facet::new("facet");
    let label = _core::Label::new("label");
    let json = format!("[\"{}\",\"{}\"]", facet, label);
    let tag: FacetedTag = serde_json::from_str(&json).unwrap();
    assert_eq!(FacetedTag::FacetLabel(facet.into(), label.into()), tag);
    assert_eq!(json, serde_json::to_string(&tag).unwrap());
}

#[test]
fn deserialize_tag_facet_label_score() {
    let facet = _core::Facet::new("facet");
    let label = _core::Label::new("label");
    let score = _core::Score::new(0.5);
    let json = format!("[\"{}\",\"{}\",{}]", facet, label, f64::from(score));
    let tag: FacetedTag = serde_json::from_str(&json).unwrap();
    assert_eq!(
        FacetedTag::FacetLabelScore(facet.into(), label.into(), score.into()),
        tag
    );
    assert_eq!(json, serde_json::to_string(&tag).unwrap());
}

#[test]
fn deserialize_tag_facet_score_zero() {
    let expected_tag = _core::Tag {
        facet: Some(_core::Facet::new("facet")),
        score: _core::Score::new(0.0),
        ..Default::default()
    };
    // Ensure to parse score from literal 0, not 0.0!
    let json = format!("[\"{}\",0]", expected_tag.facet.as_ref().unwrap());
    let parsed_tag: FacetedTag = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&parsed_tag).unwrap());
    assert_eq!(expected_tag, parsed_tag.into());
}

#[test]
fn deserialize_tag_facet_score_one() {
    let expected_tag = _core::Tag {
        facet: Some(_core::Facet::new("facet")),
        score: _core::Score::new(1.0),
        ..Default::default()
    };
    // Ensure to parse score from literal 1, not 1.0!
    let json = format!("[\"{}\",1]", expected_tag.facet.as_ref().unwrap());
    let parsed_tag: FacetedTag = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&parsed_tag).unwrap());
    assert_eq!(expected_tag, parsed_tag.into());
}

#[test]
fn deserialize_tag_facet_label_score_zero() {
    let expected_tag = _core::Tag {
        facet: Some(_core::Facet::new("facet")),
        label: Some(_core::Label::new("label")),
        score: _core::Score::new(0.0),
    };
    // Ensure to parse score from literal 0, not 0.0!
    let json = format!(
        "[\"{}\",\"{}\",0]",
        expected_tag.facet.as_ref().unwrap(),
        expected_tag.label.as_ref().unwrap()
    );
    let parsed_tag: FacetedTag = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&parsed_tag).unwrap());
    assert_eq!(expected_tag, parsed_tag.into());
}

#[test]
fn deserialize_tag_facet_label_score_one() {
    let expected_tag = _core::Tag {
        facet: Some(_core::Facet::new("facet")),
        label: Some(_core::Label::new("label")),
        score: _core::Score::new(1.0),
    };
    // Ensure to parse score from literal 1, not 1.0!
    let json = format!(
        "[\"{}\",\"{}\",1]",
        expected_tag.facet.as_ref().unwrap(),
        expected_tag.label.as_ref().unwrap()
    );
    let parsed_tag: FacetedTag = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&parsed_tag).unwrap());
    assert_eq!(expected_tag, parsed_tag.into());
}

#[test]
fn should_fail_to_deserialize_single_element_array_with_score() {
    let score = _core::Score::new(0.5);
    let json = format!("[{}]", f64::from(score));
    assert!(serde_json::from_str::<PlainTag>(&json).is_err());
    assert!(serde_json::from_str::<FacetedTag>(&json).is_err());
}
