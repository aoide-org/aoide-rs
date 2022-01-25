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

#[test]
fn trim_owned_in_place_empty() {
    let mut s = String::new();
    trim_owned_in_place(&mut s);
    assert_eq!(String::new(), s);
    let mut s = "".into();
    trim_owned_in_place(&mut s);
    assert_eq!(String::new(), s);
}

#[test]
fn trim_from_empty() {
    let trimmed = trim_from("");
    assert_eq!(String::new(), trimmed);
    let trimmed = trim_from(String::new());
    assert_eq!(String::new(), trimmed);
}

#[test]
fn trim_owned_in_place_whitespace() {
    let mut s = " \n \t \r ".into();
    trim_owned_in_place(&mut s);
    assert_eq!(String::new(), s);
}

#[test]
fn trim_from_whitespace() {
    let trimmed = trim_from(" \n \t \r ");
    assert_eq!(String::new(), trimmed);
}

#[test]
fn trim_owned_in_place_start_end() {
    let mut s = " \n \tThis \n is\ta \r Text\r ".into();
    trim_owned_in_place(&mut s);
    assert_eq!("This \n is\ta \r Text", s.as_str());
}

#[test]
fn trim_from_start_end() {
    let trimmed = trim_from(" \n \tThis \n is\ta \r Text\r ");
    assert_eq!("This \n is\ta \r Text", trimmed);
}

#[test]
fn non_empty_from_empty() {
    assert_eq!(None, non_empty_from(""));
    assert_eq!(None, non_empty_from(String::new()));
}

#[test]
fn non_empty_from_non_empty() {
    assert_eq!(Some(" ".into()), non_empty_from(" "));
}
