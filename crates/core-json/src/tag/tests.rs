// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

///////////////////////////////////////////////////////////////////////

use super::*;

#[test]
fn deserialize_plain_tag_label() {
    let label = _core::Label::from_unchecked("label");
    let json = format!("\"{label}\"");
    let tag: PlainTag = serde_json::from_str(&json).unwrap();
    assert_eq!(PlainTag::Label(label.into()), tag);
    assert_eq!(json, serde_json::to_string(&tag).unwrap());
}

#[test]
fn should_fail_to_deserialize_plain_tag_from_single_element_array_with_label() {
    let label = _core::Label::from_unchecked("label");
    let json = format!("[\"{label}\"]");
    assert!(serde_json::from_str::<PlainTag>(&json).is_err());
}

#[test]
fn deserialize_plain_tag_score_integer_one() {
    let score = _core::Score::new(1.0);
    let tag: PlainTag = serde_json::from_str("1").unwrap();
    assert_eq!(
        _core::PlainTag::from(PlainTag::Score(score.into())),
        _core::PlainTag::from(tag.clone())
    );
    assert_eq!("1", serde_json::to_string(&tag).unwrap());
}

#[test]
fn deserialize_plain_tag_score_integer_zero() {
    let score = _core::Score::new(0.0);
    let tag: PlainTag = serde_json::from_str("0").unwrap();
    assert_eq!(
        _core::PlainTag::from(PlainTag::Score(score.into())),
        _core::PlainTag::from(tag.clone())
    );
    assert_eq!("0", serde_json::to_string(&tag).unwrap());
}

#[test]
fn deserialize_plain_tag_label_score() {
    let label = _core::Label::from_unchecked("label");
    let score = _core::Score::new(0.5);
    let json = format!("[\"{label}\",{}]", score.value());
    let tag: PlainTag = serde_json::from_str(&json).unwrap();
    assert_eq!(PlainTag::LabelScore(label.into(), score.into()), tag);
    assert_eq!(json, serde_json::to_string(&tag).unwrap());
}

#[test]
fn deserialize_plain_tag_label_score_integer_zero() {
    let expected_tag = _core::PlainTag {
        label: Some(_core::Label::from_unchecked("label")),
        score: _core::Score::new(0.0),
    };
    // Ensure to parse score from literal 0, not 0.0!
    let json = format!("[\"{}\",0]", expected_tag.label.as_ref().unwrap());
    let parsed_tag: PlainTag = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&parsed_tag).unwrap());
    assert_eq!(expected_tag, parsed_tag.into());
}

#[test]
fn deserialize_plain_tag_label_score_integer_one() {
    let expected_tag = _core::PlainTag {
        label: Some(_core::Label::from_unchecked("label")),
        score: _core::Score::new(1.0),
    };
    // Ensure to parse score from literal 1, not 1.0!
    let json = format!("[\"{}\",1]", expected_tag.label.as_ref().unwrap());
    let parsed_tag: PlainTag = serde_json::from_str(&json).unwrap();
    assert_eq!(json, serde_json::to_string(&parsed_tag).unwrap());
    assert_eq!(expected_tag, parsed_tag.into());
}

#[test]
fn deserialize_tags_map() {
    let json = serde_json::json!({
        "": [
            "plain tag 1",
            ["plain tag 2", 0.5],
        ],
        "facet1": [
            "Label 1",
            ["Label 2", 0.1]
        ],
        "facet2": [
            0.8,
        ],
    })
    .to_string();
    let tags: TagsMap = serde_json::from_str(&json).unwrap();
    assert_eq!(3, tags.len());
}
