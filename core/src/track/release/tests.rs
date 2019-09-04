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
fn min_max_release_date_year() {
    assert!(RELEASE_YEAR_MIN <= ReleaseDate::min().year());
    assert!(RELEASE_YEAR_MAX <= ReleaseDate::max().year());
}

#[test]
fn into_release_yyyymmdd() {
    assert_eq!(
        19_961_219,
        ReleaseDate::from("1996-12-19T02:00:57Z".parse::<ReleaseDateTime>().unwrap()).into()
    );
    assert_eq!(
        19_961_219,
        ReleaseDate::from(
            "1996-12-19T02:00:57-12:00"
                .parse::<ReleaseDateTime>()
                .unwrap()
        )
        .into()
    );
    assert_eq!(
        19_961_219,
        ReleaseDate::from(
            "1996-12-19T02:00:57+12:00"
                .parse::<ReleaseDateTime>()
                .unwrap()
        )
        .into()
    );
}

#[test]
fn from_to_string() {
    assert_eq!(
        "1996-12-19T02:00:57Z",
        "1996-12-19T02:00:57Z"
            .parse::<ReleaseDateTime>()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "1996-12-19T02:00:57Z",
        "1996-12-19T02:00:57+00:00"
            .parse::<ReleaseDateTime>()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "1996-12-19T02:00:57Z",
        "1996-12-19T02:00:57-00:00"
            .parse::<ReleaseDateTime>()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "1996-12-19T02:00:57-12:00",
        "1996-12-19T02:00:57-12:00"
            .parse::<ReleaseDateTime>()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "1996-12-19T02:00:57+12:00",
        "1996-12-19T02:00:57+12:00"
            .parse::<ReleaseDateTime>()
            .unwrap()
            .to_string()
    );
}

#[test]
fn validate_release_date() {
    assert!(ReleaseDate::new(19_960_000).validate().is_ok());
    assert!(ReleaseDate::new(19_960_101).validate().is_ok());
    assert!(ReleaseDate::new(19_961_231).validate().is_ok());
    assert!(ReleaseDate::new(19_960_230).validate().is_err()); // 1996-02-30
    assert!(ReleaseDate::new(19_960_001).validate().is_err()); // 1996-00-01
    assert!(ReleaseDate::new(19_960_00).validate().is_err());
    assert!(ReleaseDate::new(119_960_001).validate().is_err());
}
