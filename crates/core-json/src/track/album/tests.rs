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

use aoide_core::util::canonical::Canonical;

use super::*;

#[test]
fn is_default() {
    assert!(AlbumKind::default().is_default());
    assert!(Album::default().is_default());
}

#[test]
fn into_default() {
    assert_eq!(_core::AlbumKind::default(), AlbumKind::default().into());
    assert_eq!(AlbumKind::default(), _core::AlbumKind::default().into());
    assert_eq!(
        Canonical::tie(_core::Album::default()),
        Album::default().into()
    );
    assert_eq!(Album::default(), _core::Album::default().into());
}