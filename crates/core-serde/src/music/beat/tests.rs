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

use super::*;

#[test]
fn deserialize_top() {
    let top = 4;
    let json = format!("{}", top);
    let timing: TimeSignature = serde_json::from_str(&json).unwrap();
    assert_eq!(TimeSignature::Top(top), timing);
    assert_eq!(json, serde_json::to_string(&timing).unwrap());
}

#[test]
fn should_fail_to_deserialize_single_element_array_with_top() {
    let top = 4;
    let json = format!("[{}]", top);
    assert!(serde_json::from_str::<TimeSignature>(&json).is_err());
}

#[test]
fn deserialize_top_bottom() {
    let top = 3;
    let bottom = 4;
    let json = format!("[{},{}]", top, bottom);
    let timing: TimeSignature = serde_json::from_str(&json).unwrap();
    assert_eq!(TimeSignature::TopBottom(top, bottom), timing);
    assert_eq!(json, serde_json::to_string(&timing).unwrap());
}
