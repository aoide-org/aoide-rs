// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
fn parse() {
    assert_eq!(ColorRgb::BLACK, "#000000".parse().unwrap());
    assert_eq!(ColorRgb::RED, "#FF0000".parse().unwrap());
    assert_eq!(ColorRgb::GREEN, "#00FF00".parse().unwrap());
    assert_eq!(ColorRgb::BLUE, "#0000FF".parse().unwrap());
}

#[test]
fn format() {
    assert_eq!("#000000", ColorRgb::BLACK.to_string());
    assert_eq!("#FF0000", ColorRgb::RED.to_string());
    assert_eq!("#00FF00", ColorRgb::GREEN.to_string());
    assert_eq!("#0000FF", ColorRgb::BLUE.to_string());
}
