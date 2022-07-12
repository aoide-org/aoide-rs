// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

///////////////////////////////////////////////////////////////////////

use super::*;

#[test]
fn is_default() {
    assert!(TitleKind::default().is_default());
}

#[test]
fn into_default() {
    assert_eq!(_core::TitleKind::default(), TitleKind::default().into());
    assert_eq!(TitleKind::default(), _core::TitleKind::default().into());
}
