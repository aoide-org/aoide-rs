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
fn unreliable_is_empty_default() {
    assert!(ContentMetadataFlags::UNRELIABLE.is_empty());
    assert_eq!(
        ContentMetadataFlags::default(),
        ContentMetadataFlags::UNRELIABLE
    );
}

#[test]
fn given_unreliable_reset_stale() {
    for stale_flag in &[ContentMetadataFlags::empty(), ContentMetadataFlags::STALE] {
        let mut flags = ContentMetadataFlags::UNRELIABLE | *stale_flag;
        assert!(flags.reset_stale(ContentMetadataFlags::UNRELIABLE));
        assert_eq!(flags, ContentMetadataFlags::UNRELIABLE);

        let mut flags = ContentMetadataFlags::UNRELIABLE | *stale_flag;
        assert!(flags.reset_stale(ContentMetadataFlags::RELIABLE));
        assert_eq!(flags, ContentMetadataFlags::RELIABLE);

        let mut flags = ContentMetadataFlags::UNRELIABLE | *stale_flag;
        assert!(flags.reset_stale(ContentMetadataFlags::LOCKED));
        assert_eq!(flags, ContentMetadataFlags::LOCKED);

        let mut flags = ContentMetadataFlags::UNRELIABLE | *stale_flag;
        assert!(flags.reset_stale(ContentMetadataFlags::RELIABLE | ContentMetadataFlags::LOCKED));
        assert_eq!(
            flags,
            ContentMetadataFlags::RELIABLE | ContentMetadataFlags::LOCKED
        );
    }
}

#[test]
fn given_reliable_reset_stale() {
    for stale_flag in &[ContentMetadataFlags::empty(), ContentMetadataFlags::STALE] {
        let mut flags = ContentMetadataFlags::RELIABLE | *stale_flag;
        assert!(!flags.reset_stale(ContentMetadataFlags::UNRELIABLE));
        assert_eq!(
            flags,
            ContentMetadataFlags::RELIABLE | ContentMetadataFlags::STALE
        );

        let mut flags = ContentMetadataFlags::RELIABLE | *stale_flag;
        assert!(flags.reset_stale(ContentMetadataFlags::RELIABLE));
        assert_eq!(flags, ContentMetadataFlags::RELIABLE);

        let mut flags = ContentMetadataFlags::RELIABLE | *stale_flag;
        assert!(flags.reset_stale(ContentMetadataFlags::LOCKED));
        assert_eq!(flags, ContentMetadataFlags::LOCKED);

        let mut flags = ContentMetadataFlags::RELIABLE | *stale_flag;
        assert!(flags.reset_stale(ContentMetadataFlags::RELIABLE | ContentMetadataFlags::LOCKED));
        assert_eq!(
            flags,
            ContentMetadataFlags::RELIABLE | ContentMetadataFlags::LOCKED
        );
    }
}

#[test]
fn given_locked_reset_stale() {
    for reliable_flag in &[
        ContentMetadataFlags::empty(),
        ContentMetadataFlags::RELIABLE,
    ] {
        for stale_flag in &[ContentMetadataFlags::empty(), ContentMetadataFlags::STALE] {
            let mut flags = ContentMetadataFlags::LOCKED | *reliable_flag | *stale_flag;
            assert!(!flags.reset_stale(ContentMetadataFlags::UNRELIABLE));
            assert_eq!(
                flags,
                ContentMetadataFlags::LOCKED | *reliable_flag | ContentMetadataFlags::STALE
            );

            let mut flags = ContentMetadataFlags::LOCKED | *reliable_flag | *stale_flag;
            assert!(!flags.reset_stale(ContentMetadataFlags::RELIABLE));
            assert_eq!(
                flags,
                ContentMetadataFlags::LOCKED | *reliable_flag | ContentMetadataFlags::STALE
            );

            let mut flags = ContentMetadataFlags::LOCKED | *reliable_flag | *stale_flag;
            assert!(flags.reset_stale(ContentMetadataFlags::LOCKED));
            assert_eq!(flags, ContentMetadataFlags::LOCKED);

            let mut flags = ContentMetadataFlags::LOCKED | *reliable_flag | *stale_flag;
            assert!(
                flags.reset_stale(ContentMetadataFlags::RELIABLE | ContentMetadataFlags::LOCKED)
            );
            assert_eq!(
                flags,
                ContentMetadataFlags::RELIABLE | ContentMetadataFlags::LOCKED
            );
        }
    }
}
