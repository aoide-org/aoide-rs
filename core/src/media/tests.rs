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

#[test]
fn valid_file_path_base_url() {
    let base_url = "file:///home/".parse().unwrap();
    assert!(is_valid_file_path_base_url(&base_url));
}

#[test]
fn invalid_file_path_base_url() {
    let base_url_with_unsupported_scheme = "nfs:///home/".parse().unwrap();
    assert!(!is_valid_file_path_base_url(
        &base_url_with_unsupported_scheme
    ));
}

#[test]
fn auto_complete_file_path_base_url_without_trailing_path_separator() {
    let base_url_without_trailing_path_separator = "file:///home".parse().unwrap();
    assert!(!is_valid_file_path_base_url(
        &base_url_without_trailing_path_separator
    ));
    let autocompleted_base_url =
        auto_complete_file_path_base_url(base_url_without_trailing_path_separator.clone()).unwrap();
    assert_eq!(
        &format!("{}/", base_url_without_trailing_path_separator),
        autocompleted_base_url.as_str(),
    );
    assert!(is_valid_file_path_base_url(&autocompleted_base_url));
}
