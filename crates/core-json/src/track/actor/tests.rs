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
fn is_default() {
    assert!(ActorRole::default().is_default());
    assert!(ActorKind::default().is_default());
}

#[test]
fn into_default() {
    assert_eq!(_core::ActorRole::default(), ActorRole::default().into());
    assert_eq!(ActorRole::default(), _core::ActorRole::default().into());
    assert_eq!(_core::ActorKind::default(), ActorKind::default().into());
    assert_eq!(ActorKind::default(), _core::ActorKind::default().into());
}
