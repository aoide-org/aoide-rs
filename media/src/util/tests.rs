// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
fn parse_replay_gain_valid() {
    assert_eq!(
        LoudnessLufs(-8.49428),
        parse_replay_gain("-9.50572 dB").unwrap()
    );
    assert_eq!(
        LoudnessLufs(-8.49428),
        parse_replay_gain(" -9.50572db ").unwrap()
    );
    assert_eq!(
        LoudnessLufs(-18.178062),
        parse_replay_gain("0.178062 DB").unwrap()
    );
    assert_eq!(
        LoudnessLufs(-18.178062),
        parse_replay_gain("  +0.178062   dB ").unwrap()
    );
}

#[test]
fn parse_replay_gain_invalid() {
    assert!(parse_replay_gain("-9.50572").is_none());
    assert!(parse_replay_gain("- 9.50572 dB").is_none());
    assert!(parse_replay_gain("+ 0.178062 dB").is_none());
    assert!(parse_replay_gain("+0.178062").is_none());
}

#[test]
fn parse_year_tag_valid() {
    // All test inputs surrounded by whitespaces!
    assert_eq!(
        Some(DateYYYYMMDD::new(19780000).into()),
        parse_year_tag(" 1978 ")
    );
    assert_eq!(
        Some(DateYYYYMMDD::new(20041200).into()),
        parse_year_tag(" 200412 ")
    );
    assert_eq!(
        Some(DateYYYYMMDD::new(20010900).into()),
        parse_year_tag(" 2001 \t - 9 ")
    );
    assert_eq!(
        Some(DateYYYYMMDD::new(19990702).into()),
        parse_year_tag(" 1999 - 7 - 2 ")
    );
    assert_eq!(
        Some(DateYYYYMMDD::new(19991231).into()),
        parse_year_tag(" 1999 - 12 - \t 31 ")
    );
    assert_eq!(
        Some(DateYYYYMMDD::new(20200229).into()),
        parse_year_tag(" \t20200229 ")
    );
    assert_eq!(
        "2009-09-18T07:00:00Z",
        parse_year_tag(" 2009-09-18T07:00:00Z ")
            .unwrap()
            .to_string()
    );
    // No time zone offset
    assert_eq!(
        "2009-09-18T07:00:00Z",
        parse_year_tag(" 2009-09-18T07:00:00 ").unwrap().to_string()
    );
    // Date/time separated by whitespace(s) without time zone offset
    assert_eq!(
        "2009-09-18T07:12:34Z",
        parse_year_tag("\t 2009-09-18 \t 07:12:34 ")
            .unwrap()
            .to_string()
    );
}

#[test]
fn trim_readable_should_ignore_whitespace_and_control_characters() {
    assert!(trim_readable(" \t \n ").is_empty());
    let input = String::from_utf8(vec![0x11, 0x00, 0x0A, 0x0D, 0x20, 0x09]).unwrap();
    assert!(trim_readable(&input).is_empty());
}
