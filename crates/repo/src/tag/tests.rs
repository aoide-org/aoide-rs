// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_api::tag::search::Filter;

#[test]
fn default_filter() {
    assert_eq!(Filter::any_score(), Filter::default().score);
    assert_eq!(Filter::any_term(), Filter::default().label);
}
