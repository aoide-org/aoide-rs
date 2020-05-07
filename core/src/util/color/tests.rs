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
    assert_eq!(RgbColor::BLACK, "#000000".parse().unwrap());
    assert_eq!(RgbColor::RED, "#FF0000".parse().unwrap());
    assert_eq!(RgbColor::GREEN, "#00FF00".parse().unwrap());
    assert_eq!(RgbColor::BLUE, "#0000FF".parse().unwrap());
}

#[test]
fn format() {
    assert_eq!("#000000", RgbColor::BLACK.to_string());
    assert_eq!("#FF0000", RgbColor::RED.to_string());
    assert_eq!("#00FF00", RgbColor::GREEN.to_string());
    assert_eq!("#0000FF", RgbColor::BLUE.to_string());
}
