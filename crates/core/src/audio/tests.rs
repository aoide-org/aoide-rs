// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn duration_to_string() {
    assert!(DurationMs(123.4)
        .to_string()
        .ends_with(DurationMs::unit_of_measure()));
}
