// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

///////////////////////////////////////////////////////////////////////

use super::*;

#[test]
fn is_default() {
    assert!(Role::default().is_default());
    assert!(Kind::default().is_default());
}

#[test]
fn into_default() {
    assert_eq!(_core::Role::default(), Role::default().into());
    assert_eq!(Role::default(), _core::Role::default().into());
    assert_eq!(_core::Kind::default(), Kind::default().into());
    assert_eq!(Kind::default(), _core::Kind::default().into());
}
