// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
fn resolve_url_from_url_encoded_roundtrip() -> Result<(), ResolveFromPathError> {
    let url_encoded_path = SourcePath::from(
        "https://www.example.com/Test&20path/file.mp3?param1=true&param2=test".to_owned(),
    );
    let url = UrlResolver.resolve_url_from_path(&url_encoded_path)?;
    assert_eq!(Url::parse(&url_encoded_path).unwrap(), url);
    assert_eq!(
        url_encoded_path,
        UrlResolver.resolve_path_from_url(&url).unwrap()
    );
    Ok(())
}

#[test]
fn resolve_url_from_empty_path() {
    let empty_path = SourcePath::default();
    assert!(empty_path.is_empty());
    assert!(UrlResolver.resolve_url_from_path(&empty_path).is_err());
    assert!(VirtualFilePathResolver::default()
        .resolve_url_from_path(&empty_path)
        .is_err());
}

#[test]
fn resolve_url_from_local_file_path_roundtrip() -> Result<(), ResolveFromPathError> {
    let file_url = Url::parse("file:///Test%20path/next%23*path/file.mp3").unwrap();

    let slash_path = SourcePath::from("/Test path/next#*path/file.mp3".to_owned());
    let url = VirtualFilePathResolver::default().resolve_url_from_path(&slash_path)?;
    assert_eq!(file_url, url);
    assert_eq!(
        slash_path,
        VirtualFilePathResolver::default()
            .resolve_path_from_url(&url)
            .unwrap()
    );

    let root_url = BaseUrl::parse_strict("file:///Test%20path/").unwrap();
    let resolver = VirtualFilePathResolver::with_root_url(root_url);
    let slash_path = SourcePath::from("next#*path/file.mp3".to_owned());
    let url = resolver.resolve_url_from_path(&slash_path)?;
    assert_eq!(file_url, url);
    assert_eq!(slash_path, resolver.resolve_path_from_url(&url).unwrap());
    let slash_path = SourcePath::from("next#*path/file.mp3".to_owned());
    let url = resolver.resolve_url_from_path(&slash_path)?;
    assert_eq!(file_url, url);
    assert_eq!(slash_path, resolver.resolve_path_from_url(&url).unwrap());

    Ok(())
}

#[test]
fn resolve_url_from_local_directory_path_roundtrip() -> Result<(), ResolveFromPathError> {
    let file_url = Url::parse("file:///Test%20path/next%23*path/").unwrap();

    let slash_path = SourcePath::from("/Test path/next#*path/".to_owned());
    let url = VirtualFilePathResolver::default().resolve_url_from_path(&slash_path)?;
    assert_eq!(file_url, url);
    assert_eq!(
        slash_path,
        VirtualFilePathResolver::default()
            .resolve_path_from_url(&url)
            .unwrap()
    );

    let root_url = BaseUrl::parse_strict("file:///Test%20path/").unwrap();
    let resolver = VirtualFilePathResolver::with_root_url(root_url);
    let slash_path = SourcePath::from("next#*path/".to_owned());
    let url = resolver.resolve_url_from_path(&slash_path)?;
    assert_eq!(file_url, url);
    assert_eq!(slash_path, resolver.resolve_path_from_url(&url).unwrap());
    let slash_path = SourcePath::from("next#*path/".to_owned());
    let url = resolver.resolve_url_from_path(&slash_path)?;
    assert_eq!(file_url, url);
    assert_eq!(slash_path, resolver.resolve_path_from_url(&url).unwrap());

    Ok(())
}

#[test]
fn resolve_url_from_empty_path_with_root_url() -> Result<(), ResolveFromPathError> {
    let root_url = Url::parse("file:///").unwrap();
    let resolver = VirtualFilePathResolver::with_root_url(root_url.clone().try_into().unwrap());

    assert_eq!(root_url, resolver.resolve_url_from_path("").unwrap());
    assert_eq!(root_url, resolver.resolve_url_from_path("/").unwrap());
    assert_eq!(root_url, resolver.resolve_url_from_path("//").unwrap());

    Ok(())
}

#[test]
fn resolve_url_from_relative_path_without_root_url_fails() -> Result<(), ResolveFromPathError> {
    let slash_path = SourcePath::from("Test path/file.mp3".to_owned());
    assert!(VirtualFilePathResolver::default()
        .resolve_url_from_path(&slash_path)
        .is_err());
    Ok(())
}
