// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn default_filter() {
    assert_eq!(Filter::any_score(), Filter::default().score);
    assert_eq!(Filter::any_term(), Filter::default().label);
    assert_eq!(Filter::any_facet(), Filter::default().facets);
    assert_ne!(Filter::no_facet(), Filter::default().facets);
}
