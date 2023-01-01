// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn parse_base_url() {
    assert!(BaseUrl::from_str("file:///home/path-with-trailing-slash/").is_ok());
    assert!(BaseUrl::from_str("file://host/this-is-parsed-as-an-absolute-file-path/").is_ok());
    assert!(BaseUrl::from_str("file:/this-is-parsed-as-an-absolute-file-path/").is_ok());
    assert!(
        BaseUrl::from_str("ntp:///home/path-without-trailing-slash-should-be-autocompleted")
            .is_ok()
    );
    assert!(BaseUrl::from_str("/url-without-scheme/").is_err());
}

#[test]
fn autocompletion() {
    let valid_url_without_trailing_slash = "ntp:///home/path-without-trailing-slash";
    // Autocompleted when parsing from string
    assert!(BaseUrl::from_str(valid_url_without_trailing_slash).is_ok());
    // Autocompleted on demand
    assert!(BaseUrl::try_autocomplete_from(
        Url::from_str(valid_url_without_trailing_slash).unwrap()
    )
    .is_ok());
    // Not implicitly autcompleted from intermediate URL
    assert!(BaseUrl::try_from(Url::from_str(valid_url_without_trailing_slash).unwrap()).is_err());
}
