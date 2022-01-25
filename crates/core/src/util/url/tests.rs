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
