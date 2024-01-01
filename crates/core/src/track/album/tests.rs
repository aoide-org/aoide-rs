// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::track::{actor, title};

#[test]
fn with_main_title() {
    let album = Album {
        titles: Canonical::tie(vec![Title {
            name: "main".to_string(),
            kind: title::Kind::Main,
        }]),
        ..Default::default()
    };
    assert!(album.validate().is_ok());
}

#[test]
fn without_main_title() {
    let album = Album {
        titles: Canonical::tie(vec![Title {
            name: "sub".to_string(),
            kind: title::Kind::Sub,
        }]),
        ..Default::default()
    };
    assert!(album.validate().is_err());
}

#[test]
fn with_main_artist() {
    let album = Album {
        titles: Canonical::tie(vec![Title {
            name: "main".to_string(),
            kind: title::Kind::Main,
        }]),
        actors: Canonical::tie(vec![Actor {
            name: "artist".to_string(),
            role: actor::Role::Artist,
            ..Default::default()
        }]),
        ..Default::default()
    };
    assert!(Actors::main_actor(album.actors.iter(), Default::default()).is_some());
    assert!(album.validate().is_ok());
}

#[test]
fn without_main_artist() {
    let album = Album {
        titles: Canonical::tie(vec![Title {
            name: "main".to_string(),
            kind: title::Kind::Main,
        }]),
        actors: Canonical::tie(vec![Actor {
            name: "composer".to_string(),
            role: actor::Role::Composer,
            ..Default::default()
        }]),
        ..Default::default()
    };
    // No main artist required
    assert!(Actors::main_actor(album.actors.iter(), Default::default()).is_none());
    assert!(album.validate().is_ok());
}
