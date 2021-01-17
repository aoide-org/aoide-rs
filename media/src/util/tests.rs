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
