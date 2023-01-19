// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn resolve_url_from_url_encoded_roundtrip() -> Result<(), ResolveFromPathError> {
    let url_encoded_path =
        ContentPath::from("https://www.example.com/Test&20path/file.mp3?param1=true&param2=test");
    let url = UrlResolver.resolve_url_from_content_path(&url_encoded_path)?;
    assert_eq!(Url::parse(url_encoded_path.as_str()).unwrap(), url);
    assert_eq!(
        url_encoded_path,
        UrlResolver.resolve_path_from_url(&url).unwrap()
    );
    Ok(())
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn resolve_url_from_empty_path() {
    let empty_path = ContentPath::default();
    assert!(empty_path.is_empty());
    assert!(UrlResolver
        .resolve_url_from_content_path(&empty_path)
        .is_err());
    assert!(VirtualFilePathResolver::default()
        .resolve_url_from_content_path(&empty_path)
        .is_err());
}

#[cfg(any(target_family = "unix", target_family = "windows"))]
#[test]
fn resolve_url_from_local_file_path_roundtrip() -> Result<(), ResolveFromPathError> {
    #[cfg(target_family = "unix")]
    let file_url = Url::parse("file:///Test%20path/next%23*%3Fpath/file.mp3").unwrap();
    #[cfg(target_family = "windows")]
    // <https://docs.microsoft.com/en-us/windows/win32/fileio/naming-a-file#naming-conventions>
    let file_url = Url::parse("file:///C:/Test%20path/next%23path/file.mp3").unwrap();

    #[cfg(target_family = "unix")]
    let slash_path = ContentPath::from("/Test path/next#*?path/file.mp3");
    #[cfg(target_family = "windows")]
    let slash_path = ContentPath::from("C:/Test path/next#path/file.mp3");

    let url = VirtualFilePathResolver::default().resolve_url_from_content_path(&slash_path)?;
    assert_eq!(file_url, url);
    assert_eq!(
        slash_path,
        VirtualFilePathResolver::default()
            .resolve_path_from_url(&url)
            .unwrap()
    );

    #[cfg(target_family = "unix")]
    let root_url = BaseUrl::parse_strict("file:///Test%20path/").unwrap();
    #[cfg(target_family = "windows")]
    let root_url = BaseUrl::parse_strict("file:///C:/Test%20path/").unwrap();

    let resolver = VirtualFilePathResolver::with_root_url(root_url);

    #[cfg(target_family = "unix")]
    let slash_path = ContentPath::from("next#*?path/file.mp3");
    #[cfg(target_family = "windows")]
    let slash_path = ContentPath::from("next#path/file.mp3");

    let url = resolver.resolve_url_from_content_path(&slash_path)?;
    assert_eq!(file_url, url);
    assert_eq!(slash_path, resolver.resolve_path_from_url(&url).unwrap());

    #[cfg(target_family = "unix")]
    let slash_path = ContentPath::from("next#*?path/file.mp3");
    #[cfg(target_family = "windows")]
    let slash_path = ContentPath::from("next#path/file.mp3");

    let url = resolver.resolve_url_from_content_path(&slash_path)?;
    assert_eq!(file_url, url);
    assert_eq!(slash_path, resolver.resolve_path_from_url(&url).unwrap());

    Ok(())
}

#[cfg(any(target_family = "unix", target_family = "windows"))]
#[test]
fn resolve_url_from_local_directory_path_roundtrip() -> Result<(), ResolveFromPathError> {
    #[cfg(target_family = "unix")]
    let file_url = Url::parse("file:///Test%20path/next%23*%3Fpath/").unwrap();
    #[cfg(target_family = "windows")]
    // <https://docs.microsoft.com/en-us/windows/win32/fileio/naming-a-file#naming-conventions>
    let file_url = Url::parse("file:///C:/Test%20path/next%23path/").unwrap();

    #[cfg(target_family = "unix")]
    let slash_path = ContentPath::from("/Test path/next#*?path/");
    #[cfg(target_family = "windows")]
    let slash_path = ContentPath::from("C:/Test path/next#path/");

    let url = VirtualFilePathResolver::default().resolve_url_from_content_path(&slash_path)?;
    assert_eq!(file_url, url);
    assert_eq!(
        slash_path,
        VirtualFilePathResolver::default()
            .resolve_path_from_url(&url)
            .unwrap()
    );

    #[cfg(target_family = "unix")]
    let root_url = BaseUrl::parse_strict("file:///Test%20path/").unwrap();
    #[cfg(target_family = "windows")]
    let root_url = BaseUrl::parse_strict("file:///C:/Test%20path/").unwrap();

    let resolver = VirtualFilePathResolver::with_root_url(root_url);

    #[cfg(target_family = "unix")]
    let slash_path = ContentPath::from("next#*?path/");
    #[cfg(target_family = "windows")]
    let slash_path = ContentPath::from("next#path/");

    let url = resolver.resolve_url_from_content_path(&slash_path)?;
    assert_eq!(file_url, url);
    assert_eq!(slash_path, resolver.resolve_path_from_url(&url).unwrap());

    #[cfg(target_family = "unix")]
    let slash_path = ContentPath::from("next#*?path/");
    #[cfg(target_family = "windows")]
    let slash_path = ContentPath::from("next#path/");

    let url = resolver.resolve_url_from_content_path(&slash_path)?;
    assert_eq!(file_url, url);
    assert_eq!(slash_path, resolver.resolve_path_from_url(&url).unwrap());

    Ok(())
}

#[cfg(any(target_family = "unix", target_family = "windows"))]
#[test]
fn resolve_url_from_empty_path_with_root_url() -> Result<(), ResolveFromPathError> {
    #[cfg(target_family = "unix")]
    let root_url = Url::parse("file:///").unwrap();
    #[cfg(target_family = "windows")]
    let root_url = Url::parse("file:///C:/").unwrap();

    let resolver = VirtualFilePathResolver::with_root_url(root_url.clone().try_into().unwrap());

    assert_eq!(
        root_url,
        resolver.resolve_url_from_content_path(&ContentPath::from(""))?
    );
    assert_eq!(
        root_url,
        resolver.resolve_url_from_content_path(&ContentPath::from("/"))?
    );
    assert_eq!(
        root_url,
        resolver.resolve_url_from_content_path(&ContentPath::from("//"))?
    );

    Ok(())
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn resolve_url_from_relative_path_without_root_url_fails() {
    let slash_path = ContentPath::from("Test path/file.mp3");
    assert!(VirtualFilePathResolver::default()
        .resolve_url_from_content_path(&slash_path)
        .is_err());
}
