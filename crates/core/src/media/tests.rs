// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

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
fn given_unreliable_update() {
    for stale_flag in &[ContentMetadataFlags::empty(), ContentMetadataFlags::STALE] {
        let mut flags = ContentMetadataFlags::UNRELIABLE | *stale_flag;
        assert!(flags.update(ContentMetadataFlags::UNRELIABLE));
        assert_eq!(flags, ContentMetadataFlags::UNRELIABLE);

        let mut flags = ContentMetadataFlags::UNRELIABLE | *stale_flag;
        assert!(flags.update(ContentMetadataFlags::RELIABLE));
        assert_eq!(flags, ContentMetadataFlags::RELIABLE);

        let mut flags = ContentMetadataFlags::UNRELIABLE | *stale_flag;
        assert!(flags.update(ContentMetadataFlags::LOCKED));
        assert_eq!(flags, ContentMetadataFlags::LOCKED);

        let mut flags = ContentMetadataFlags::UNRELIABLE | *stale_flag;
        assert!(flags.update(ContentMetadataFlags::RELIABLE | ContentMetadataFlags::LOCKED));
        assert_eq!(
            flags,
            ContentMetadataFlags::RELIABLE | ContentMetadataFlags::LOCKED
        );
    }
}

#[test]
fn given_reliable_update() {
    for stale_flag in &[ContentMetadataFlags::empty(), ContentMetadataFlags::STALE] {
        let mut flags = ContentMetadataFlags::RELIABLE | *stale_flag;
        assert!(!flags.update(ContentMetadataFlags::UNRELIABLE));
        assert_eq!(
            flags,
            ContentMetadataFlags::RELIABLE | ContentMetadataFlags::STALE
        );

        let mut flags = ContentMetadataFlags::RELIABLE | *stale_flag;
        assert!(flags.update(ContentMetadataFlags::RELIABLE));
        assert_eq!(flags, ContentMetadataFlags::RELIABLE);

        let mut flags = ContentMetadataFlags::RELIABLE | *stale_flag;
        assert!(flags.update(ContentMetadataFlags::LOCKED));
        assert_eq!(flags, ContentMetadataFlags::LOCKED);

        let mut flags = ContentMetadataFlags::RELIABLE | *stale_flag;
        assert!(flags.update(ContentMetadataFlags::RELIABLE | ContentMetadataFlags::LOCKED));
        assert_eq!(
            flags,
            ContentMetadataFlags::RELIABLE | ContentMetadataFlags::LOCKED
        );
    }
}

#[test]
fn given_locked_update() {
    for reliable_flag in &[
        ContentMetadataFlags::empty(),
        ContentMetadataFlags::RELIABLE,
    ] {
        for stale_flag in &[ContentMetadataFlags::empty(), ContentMetadataFlags::STALE] {
            let mut flags = ContentMetadataFlags::LOCKED | *reliable_flag | *stale_flag;
            assert!(!flags.update(ContentMetadataFlags::UNRELIABLE));
            assert_eq!(
                flags,
                // The stale flag is not set, but the current stale flag is preserved
                ContentMetadataFlags::LOCKED | *reliable_flag | *stale_flag
            );

            let mut flags = ContentMetadataFlags::LOCKED | *reliable_flag | *stale_flag;
            assert!(!flags.update(ContentMetadataFlags::RELIABLE));
            assert_eq!(
                flags,
                // The stale flag is not set, but the current stale flag is preserved
                ContentMetadataFlags::LOCKED | *reliable_flag | *stale_flag
            );

            let mut flags = ContentMetadataFlags::LOCKED | *reliable_flag | *stale_flag;
            assert!(flags.update(ContentMetadataFlags::LOCKED));
            assert_eq!(flags, ContentMetadataFlags::LOCKED);

            let mut flags = ContentMetadataFlags::LOCKED | *reliable_flag | *stale_flag;
            assert!(flags.update(ContentMetadataFlags::RELIABLE | ContentMetadataFlags::LOCKED));
            assert_eq!(
                flags,
                ContentMetadataFlags::RELIABLE | ContentMetadataFlags::LOCKED
            );
        }
    }
}
