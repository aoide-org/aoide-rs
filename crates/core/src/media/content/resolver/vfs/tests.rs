// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn resolve_url_from_empty_path() {
    let empty_path = ContentPath::default();
    assert!(empty_path.is_empty());
    assert!(VfsResolver::default()
        .resolve_url_from_content_path(&empty_path)
        .is_err());
}

#[test]
fn resolve_url_from_local_file_path_roundtrip() -> Result<(), ResolveFromPathError> {
    #[cfg(not(target_family = "windows"))]
    let file_url = Url::parse("file:///Test%20path/next%23*%3Fpath/file.mp3").unwrap();
    #[cfg(target_family = "windows")]
    // <https://docs.microsoft.com/en-us/windows/win32/fileio/naming-a-file#naming-conventions>
    let file_url = Url::parse("file:///C:/Test%20path/next%23path/file.mp3").unwrap();

    #[cfg(not(target_family = "windows"))]
    let slash_path = ContentPath::from("/Test path/next#*?path/file.mp3");
    #[cfg(target_family = "windows")]
    let slash_path = ContentPath::from("C:/Test path/next#path/file.mp3");

    let url = VfsResolver::default().resolve_url_from_content_path(&slash_path)?;
    assert_eq!(file_url, url);
    assert_eq!(
        slash_path,
        VfsResolver::default().resolve_path_from_url(&url).unwrap()
    );

    #[cfg(not(target_family = "windows"))]
    let root_url = BaseUrl::parse_strict("file:///Test%20path/").unwrap();
    #[cfg(target_family = "windows")]
    let root_url = BaseUrl::parse_strict("file:///C:/Test%20path/").unwrap();

    let resolver = VfsResolver::with_root_url(root_url);

    #[cfg(not(target_family = "windows"))]
    let slash_path = ContentPath::from("next#*?path/file.mp3");
    #[cfg(target_family = "windows")]
    let slash_path = ContentPath::from("next#path/file.mp3");

    let url = resolver.resolve_url_from_content_path(&slash_path)?;
    assert_eq!(file_url, url);
    assert_eq!(slash_path, resolver.resolve_path_from_url(&url).unwrap());

    #[cfg(not(target_family = "windows"))]
    let slash_path = ContentPath::from("next#*?path/file.mp3");
    #[cfg(target_family = "windows")]
    let slash_path = ContentPath::from("next#path/file.mp3");

    let url = resolver.resolve_url_from_content_path(&slash_path)?;
    assert_eq!(file_url, url);
    assert_eq!(slash_path, resolver.resolve_path_from_url(&url).unwrap());

    Ok(())
}

#[test]
fn resolve_url_from_local_directory_path_roundtrip() -> Result<(), ResolveFromPathError> {
    #[cfg(not(target_family = "windows"))]
    let file_url = Url::parse("file:///Test%20path/next%23*%3Fpath/").unwrap();
    #[cfg(target_family = "windows")]
    // <https://docs.microsoft.com/en-us/windows/win32/fileio/naming-a-file#naming-conventions>
    let file_url = Url::parse("file:///C:/Test%20path/next%23path/").unwrap();

    #[cfg(not(target_family = "windows"))]
    let slash_path = ContentPath::from("/Test path/next#*?path/");
    #[cfg(target_family = "windows")]
    let slash_path = ContentPath::from("C:/Test path/next#path/");

    let url = VfsResolver::default().resolve_url_from_content_path(&slash_path)?;
    assert_eq!(file_url, url);
    assert_eq!(
        slash_path,
        VfsResolver::default().resolve_path_from_url(&url).unwrap()
    );

    #[cfg(not(target_family = "windows"))]
    let root_url = BaseUrl::parse_strict("file:///Test%20path/").unwrap();
    #[cfg(target_family = "windows")]
    let root_url = BaseUrl::parse_strict("file:///C:/Test%20path/").unwrap();

    let resolver = VfsResolver::with_root_url(root_url);

    #[cfg(not(target_family = "windows"))]
    let slash_path = ContentPath::from("next#*?path/");
    #[cfg(target_family = "windows")]
    let slash_path = ContentPath::from("next#path/");

    let url = resolver.resolve_url_from_content_path(&slash_path)?;
    assert_eq!(file_url, url);
    assert_eq!(slash_path, resolver.resolve_path_from_url(&url).unwrap());

    #[cfg(not(target_family = "windows"))]
    let slash_path = ContentPath::from("next#*?path/");
    #[cfg(target_family = "windows")]
    let slash_path = ContentPath::from("next#path/");

    let url = resolver.resolve_url_from_content_path(&slash_path)?;
    assert_eq!(file_url, url);
    assert_eq!(slash_path, resolver.resolve_path_from_url(&url).unwrap());

    Ok(())
}

#[test]
fn resolve_url_from_empty_path_with_root_url() -> Result<(), ResolveFromPathError> {
    #[cfg(not(target_family = "windows"))]
    let root_url = Url::parse("file:///").unwrap();
    #[cfg(target_family = "windows")]
    let root_url = Url::parse("file:///C:/").unwrap();

    let resolver = VfsResolver::with_root_url(root_url.clone().try_into().unwrap());

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

#[test]
fn resolve_url_from_relative_path_without_root_url_fails() {
    let slash_path = ContentPath::from("Test path/file.mp3");
    assert!(VfsResolver::default()
        .resolve_url_from_content_path(&slash_path)
        .is_err());
}

#[test]
fn remap_content_path_to_file_path() {
    #[cfg(not(target_family = "windows"))]
    const ROOT_PATH: &str = "/root/path/";
    #[cfg(target_family = "windows")]
    const ROOT_PATH: &str = "/C:/root/path/";

    #[cfg(not(target_family = "windows"))]
    const OVERRIDE_ROOT_PATH: &str = "/override/";
    #[cfg(target_family = "windows")]
    const OVERRIDE_ROOT_PATH: &str = "/C:/override/";

    const SUB_PATH: &str = "sub/";

    const CONTENT_PATH: ContentPath<'_> = ContentPath::new(Cow::Borrowed("sub/file.mp3"));
    const SUB_OVERRIDE_CONTENT_PATH: ContentPath<'_> = ContentPath::new(Cow::Borrowed("file.mp3"));

    let canonical_root_url = &format!("file://{ROOT_PATH}").parse::<BaseUrl>().unwrap();
    let override_root_url = &format!("file://{OVERRIDE_ROOT_PATH}")
        .parse::<BaseUrl>()
        .unwrap();

    let root_sub_path = &format!("{ROOT_PATH}{SUB_PATH}");
    let root_sub_url = &format!("file://{root_sub_path}")
        .parse::<BaseUrl>()
        .unwrap();

    let vfs = RemappingVfsResolver::new(canonical_root_url.clone(), None, None).unwrap();
    assert!(vfs.root_url.is_none());
    assert!(vfs.root_path.is_empty());
    assert_eq!(
        format!("file://{ROOT_PATH}{CONTENT_PATH}")
            .parse::<Url>()
            .unwrap(),
        Url::from_file_path(vfs.build_file_path(&CONTENT_PATH)).unwrap()
    );

    let vfs_override = RemappingVfsResolver::new(
        canonical_root_url.clone(),
        None,
        Some(override_root_url.clone()),
    )
    .unwrap();
    assert_eq!(vfs_override.root_url.as_ref(), Some(canonical_root_url));
    assert!(vfs_override.root_path.is_empty());
    assert_eq!(
        format!("file://{OVERRIDE_ROOT_PATH}{CONTENT_PATH}")
            .parse::<Url>()
            .unwrap(),
        Url::from_file_path(vfs_override.build_file_path(&CONTENT_PATH)).unwrap()
    );

    let vfs_sub =
        RemappingVfsResolver::new(canonical_root_url.clone(), Some(root_sub_url), None).unwrap();
    assert!(vfs.root_url.is_none());
    assert_eq!(vfs_sub.root_path, ContentPath::new(SUB_PATH.into()));
    assert_eq!(
        format!("file://{ROOT_PATH}{CONTENT_PATH}")
            .parse::<Url>()
            .unwrap(),
        Url::from_file_path(vfs_sub.build_file_path(&CONTENT_PATH)).unwrap()
    );

    let vfs_sub_override = RemappingVfsResolver::new(
        canonical_root_url.clone(),
        Some(root_sub_url),
        Some(override_root_url.clone()),
    )
    .unwrap();
    assert_eq!(vfs_sub_override.root_url.as_ref(), Some(canonical_root_url));
    assert_eq!(
        vfs_sub_override.root_path,
        ContentPath::new(SUB_PATH.into())
    );
    assert_eq!(
        format!("file://{OVERRIDE_ROOT_PATH}{SUB_OVERRIDE_CONTENT_PATH}")
            .parse::<Url>()
            .unwrap(),
        Url::from_file_path(vfs_sub_override.build_file_path(&CONTENT_PATH)).unwrap()
    );
}
