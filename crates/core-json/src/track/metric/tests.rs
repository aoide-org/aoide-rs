// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

///////////////////////////////////////////////////////////////////////

use super::*;

#[test]
fn is_default() {
    assert!(Metrics::default().is_default());
}

#[test]
fn into_default() {
    assert_eq!(
        _core::Metrics::default(),
        Metrics::default().try_into().unwrap()
    );
    assert_eq!(Metrics::default(), _core::Metrics::default().into());
}
