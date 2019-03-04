// aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

///////////////////////////////////////////////////////////////////////

#[test]
fn default_uid() {
    assert!(!EntityUid::default().is_valid());
    assert_eq!(
        EntityUid::default().as_ref().len(),
        mem::size_of::<EntityUid>()
    );
}

#[test]
fn generate_uid() {
    assert!(EntityUidGenerator::generate_uid().is_valid());
}

#[test]
fn revision_sequence() {
    let initial = EntityRevision::initial();
    assert!(initial.is_valid());
    assert!(initial.is_initial());

    let next = initial.next();
    assert!(next.is_valid());
    assert!(!next.is_initial());
    assert!(initial < next);
    assert!(initial.ordinal() < next.ordinal());
    assert!(initial.timestamp() <= next.timestamp());

    let nextnext = next.next();
    assert!(nextnext.is_valid());
    assert!(!nextnext.is_initial());
    assert!(next < nextnext);
    assert!(next.ordinal() < nextnext.ordinal());
    assert!(next.timestamp() <= nextnext.timestamp());
}

#[test]
fn header_without_uid() {
    let header = EntityHeader::initial_with_uid(EntityUid::default());
    assert!(!header.is_valid());
    assert!(header.revision().is_initial());
}

#[test]
fn header_with_uid() {
    let header = EntityHeader::initial_with_uid(EntityUidGenerator::generate_uid());
    assert!(header.is_valid());
    assert!(header.revision().is_initial());
}
