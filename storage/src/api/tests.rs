// aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
fn default_tag_filter() {
    assert_eq!(TagFilter::any_score(), TagFilter::default().score_condition);
    assert_eq!(TagFilter::any_term(), TagFilter::default().term_condition);
    assert_eq!(TagFilter::any_facet(), TagFilter::default().facet);
    assert_ne!(TagFilter::no_facet(), TagFilter::default().facet);
}
