// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn resolve_url_from_url_encoded_roundtrip() -> Result<(), ResolveFromPathError> {
    let url_encoded_path =
        ContentPath::from("https://www.example.com/Test&20path/file.mp3?param1=true&param2=test");
    let url = UrlResolver.resolve_url_from_path(&url_encoded_path)?;
    assert_eq!(Url::parse(url_encoded_path.as_str()).unwrap(), url);
    assert_eq!(
        Some(url_encoded_path),
        UrlResolver.resolve_path_from_url(&url).unwrap()
    );
    Ok(())
}

#[test]
fn resolve_url_from_empty_path() {
    let empty_path = ContentPath::default();
    assert!(empty_path.is_empty());
    assert!(UrlResolver.resolve_url_from_path(&empty_path).is_err());
}
