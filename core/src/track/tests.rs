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

use super::*;

#[test]
fn deserialize_index_count() {
    let index_only = "1";
    let index_only: IndexCount = serde_json::from_str(&index_only).unwrap();
    assert_eq!(IndexCount::Index(1), index_only);

    let index_count = "[1,0]";
    let index_count: IndexCount = serde_json::from_str(&index_count).unwrap();
    assert_eq!(IndexCount::IndexAndCount(1, 0), index_count);

    let index_count = "[7,12]";
    let index_count: IndexCount = serde_json::from_str(&index_count).unwrap();
    assert_eq!(IndexCount::IndexAndCount(7, 12), index_count);

    let index_count = "[0,12]";
    let index_count: IndexCount = serde_json::from_str(&index_count).unwrap();
    assert_eq!(IndexCount::IndexAndCount(0, 12), index_count);
}
