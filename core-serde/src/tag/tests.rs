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
fn deserialize_tag_label_score() {
    let label = _core::Label::new("label");
    let score = _core::Score::new(0.5);
    let json = format!("[\"{}\",{}]", label, f64::from(score));
    let tag: PlainTag = serde_json::from_str(&json).unwrap();
    assert_eq!(PlainTag::LabelScore(label.into(), score.into()), tag);
    assert_eq!(json, serde_json::to_string(&tag).unwrap());
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