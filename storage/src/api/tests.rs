// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
fn default_tag_filter() {
    assert_eq!(TagFilter::any_score(), TagFilter::default().score);
    assert_eq!(TagFilter::any_term(), TagFilter::default().label);
    assert_eq!(TagFilter::any_facet(), TagFilter::default().facets);
    assert_ne!(TagFilter::no_facet(), TagFilter::default().facets);
}

#[test]
fn deserialize_count_tracks_by_tag_params() {
    let params: CountTracksByTagParams = serde_json::from_str("{}").unwrap();
    assert!(params.facets.is_none());
    assert!(params.include_non_faceted_tags);

    let params: CountTracksByTagParams = serde_json::from_str("{\"facets\":null}").unwrap();
    assert!(params.facets.is_none());
    assert!(params.include_non_faceted_tags);

    let params: CountTracksByTagParams = serde_json::from_str("{\"facets\":[]}").unwrap();
    assert_eq!(Some(vec![]), params.facets);
    assert!(params.include_non_faceted_tags);

    let params: CountTracksByTagParams = serde_json::from_str(
        "{\"facets\":[\"facet1\",\"facet2\"],\"includeNonFacetedTags\":false}",
    )
    .unwrap();
    assert_eq!(
        Some(vec![
            Facet::new("facet1".into()),
            Facet::new("facet2".into())
        ]),
        params.facets
    );
    assert!(!params.include_non_faceted_tags);
}
