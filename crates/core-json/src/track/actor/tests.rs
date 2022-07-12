// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

///////////////////////////////////////////////////////////////////////

use super::*;

#[test]
fn is_default() {
    assert!(ActorRole::default().is_default());
    assert!(ActorKind::default().is_default());
}

#[test]
fn into_default() {
    assert_eq!(_core::ActorRole::default(), ActorRole::default().into());
    assert_eq!(ActorRole::default(), _core::ActorRole::default().into());
    assert_eq!(_core::ActorKind::default(), ActorKind::default().into());
    assert_eq!(ActorKind::default(), _core::ActorKind::default().into());
}
