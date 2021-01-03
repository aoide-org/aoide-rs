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
fn deserialize_playlist() {
    let playlist: Playlist = serde_json::from_str(r#"{"nam":"test","typ":"type","lst":[{"itm":{"trk":{"uid":"MAdeyPtrDVSMnwpriPA5anaD66xw5iP1s"}},"add":1578221715728131},{"itm":"sep","add":1578221715728132}]}"#).unwrap();
    assert_eq!("test", playlist.name);
    assert_eq!(Some("type".into()), playlist.r#type);
    assert_eq!(2, playlist.entries.len());
}
