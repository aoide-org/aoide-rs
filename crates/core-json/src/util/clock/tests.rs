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

use super::*;

use serde_json::json;

use aoide_core::util::clock::{DateYYYYMMDD, YEAR_MAX, YEAR_MIN};

mod _core {
    pub(super) use aoide_core::util::clock::DateOrDateTime;
}

#[test]
fn deserialize_min() {
    assert_eq!(
        _core::DateOrDateTime::Date(DateYYYYMMDD::min()),
        serde_json::from_value::<DateOrDateTime>(json!(DateYYYYMMDD::min().to_inner()))
            .unwrap()
            .into()
    );
}

#[test]
fn deserialize_max() {
    assert_eq!(
        _core::DateOrDateTime::Date(DateYYYYMMDD::max()),
        serde_json::from_value::<DateOrDateTime>(json!(DateYYYYMMDD::max().to_inner()))
            .unwrap()
            .into()
    );
}

#[test]
fn deserialize_year() {
    assert_eq!(
        _core::DateOrDateTime::Date(DateYYYYMMDD::from_year(YEAR_MIN)),
        serde_json::from_value::<DateOrDateTime>(json!(YEAR_MIN))
            .unwrap()
            .into()
    );
    assert_eq!(
        _core::DateOrDateTime::Date(DateYYYYMMDD::from_year(YEAR_MAX)),
        serde_json::from_value::<DateOrDateTime>(json!(YEAR_MAX))
            .unwrap()
            .into()
    );
    assert_eq!(
        _core::DateOrDateTime::Date(DateYYYYMMDD::new(19_960_000)),
        serde_json::from_value::<DateOrDateTime>(json!(1996))
            .unwrap()
            .into()
    );
    assert_eq!(
        _core::DateOrDateTime::Date(DateYYYYMMDD::new(19_960_000)),
        serde_json::from_value::<DateOrDateTime>(json!(19_960_000))
            .unwrap()
            .into()
    );
    assert_eq!(
        _core::DateOrDateTime::Date(DateYYYYMMDD::new(19_960_900)),
        serde_json::from_value::<DateOrDateTime>(json!(19_960_900))
            .unwrap()
            .into()
    );
    assert!(serde_json::from_value::<DateOrDateTime>(json!(19_960_001)).is_err());
    assert!(serde_json::from_value::<DateOrDateTime>(json!(199_600)).is_err());
    assert!(serde_json::from_value::<DateOrDateTime>(json!(0)).is_err());
    assert!(serde_json::from_value::<DateOrDateTime>(json!(-1996)).is_err());
}

#[test]
fn serialize_min() {
    assert!(DateYYYYMMDD::min().is_year());
    assert_eq!(
        serde_json::to_string(&DateOrDateTime::Date(DateYYYYMMDD::min().into())).unwrap(),
        serde_json::to_string(&json!(YEAR_MIN)).unwrap()
    );
}

#[test]
fn serialize_max() {
    assert!(!DateYYYYMMDD::max().is_year());
    assert_eq!(
        serde_json::to_string(&DateOrDateTime::Date(DateYYYYMMDD::max().into())).unwrap(),
        serde_json::to_string(&json!(DateYYYYMMDD::max().to_inner())).unwrap()
    );
}

#[test]
fn serialize_year() {
    assert_eq!(
        serde_json::to_string(&DateOrDateTime::Date(DateYYYYMMDD::new(19_960_000).into())).unwrap(),
        serde_json::to_string(&json!(1996)).unwrap()
    );
    assert_eq!(
        serde_json::to_string(&DateOrDateTime::Date(DateYYYYMMDD::new(19_961_000).into())).unwrap(),
        serde_json::to_string(&json!(19_961_000)).unwrap()
    );
    assert_eq!(
        serde_json::to_string(&DateOrDateTime::Date(DateYYYYMMDD::new(19_961_031).into())).unwrap(),
        serde_json::to_string(&json!(19_961_031)).unwrap()
    );
}

#[test]
fn deserialize_year_month() {
    assert_eq!(
        _core::DateOrDateTime::Date(DateYYYYMMDD::new(19_960_100)),
        serde_json::from_value::<DateOrDateTime>(json!(19_960_100))
            .unwrap()
            .into()
    );
    assert_eq!(
        _core::DateOrDateTime::Date(DateYYYYMMDD::new(19_961_200)),
        serde_json::from_value::<DateOrDateTime>(json!(19_961_200))
            .unwrap()
            .into()
    );
    assert!(serde_json::from_value::<DateOrDateTime>(json!(19_961_300)).is_err());
    assert!(serde_json::from_value::<DateOrDateTime>(json!(199_601)).is_err());
    assert!(serde_json::from_value::<DateOrDateTime>(json!(-199_601)).is_err());
}

#[test]
fn deserialize_year_month_day() {
    assert_eq!(
        _core::DateOrDateTime::Date(DateYYYYMMDD::new(19_960_101)),
        serde_json::from_value::<DateOrDateTime>(json!(19_960_101))
            .unwrap()
            .into()
    );
    assert_eq!(
        _core::DateOrDateTime::Date(DateYYYYMMDD::new(19_961_231)),
        serde_json::from_value::<DateOrDateTime>(json!(19_961_231))
            .unwrap()
            .into()
    );
    assert!(serde_json::from_value::<DateOrDateTime>(json!(19_961_232)).is_err());
    assert!(serde_json::from_value::<DateOrDateTime>(json!(19_961_301)).is_err());
    assert!(serde_json::from_value::<DateOrDateTime>(json!(19_960_631)).is_err());
    assert!(serde_json::from_value::<DateOrDateTime>(json!(19_960_001)).is_err());
}
