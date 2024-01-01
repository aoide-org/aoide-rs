// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn is_default() {
    assert!(Indexes::default().is_default());
}

#[test]
fn into_default() {
    assert_eq!(_core::Indexes::default(), Indexes::default().into());
    assert_eq!(Indexes::default(), _core::Indexes::default().into());
}
