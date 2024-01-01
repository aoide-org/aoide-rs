// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn actors() {
    let summary_artist_name = "Madonna feat. M.I.A. and Nicki Minaj";
    let individual_artist_names = ["Madonna", "M.I.A.", "Nicki Minaj"];
    let individual_producer_name = "Martin Solveig";
    let actors = vec![
        Actor {
            name: summary_artist_name.into(),
            ..Default::default()
        },
        Actor {
            name: individual_artist_names[0].into(),
            kind: Kind::Individual,
            ..Default::default()
        },
        Actor {
            name: individual_artist_names[1].into(),
            kind: Kind::Individual,
            ..Default::default()
        },
        Actor {
            name: individual_producer_name.into(),
            role: Role::Producer,
            kind: Kind::Individual,
            ..Default::default()
        },
        Actor {
            name: individual_artist_names[2].into(),
            kind: Kind::Individual,
            ..Default::default()
        },
    ];

    assert!(Actors::validate(&actors.iter()).is_ok());

    // Artist(s)
    assert_eq!(
        &[summary_artist_name],
        Actors::filter_kind_role(actors.iter(), Kind::Summary, Role::Artist)
            .map(|actor| actor.name.as_str())
            .collect::<Vec<_>>()
            .as_slice()
    );
    assert_eq!(
        individual_artist_names,
        Actors::filter_kind_role(actors.iter(), Kind::Individual, Role::Artist)
            .map(|actor| actor.name.as_str())
            .collect::<Vec<_>>()
            .as_slice()
    );
    assert_eq!(
        summary_artist_name,
        Actors::main_actor(actors.iter(), Role::Artist)
            .unwrap()
            .name
    );

    // Producer(s)
    assert_eq!(
        0,
        Actors::filter_kind_role(&actors, Kind::Summary, Role::Producer).count()
    );
    assert_eq!(
        &[individual_producer_name],
        Actors::filter_kind_role(&actors, Kind::Individual, Role::Producer)
            .map(|actor| actor.name.as_str())
            .collect::<Vec<_>>()
            .as_slice()
    );
    assert_eq!(
        individual_producer_name,
        Actors::main_actor(actors.iter(), Role::Producer)
            .unwrap()
            .name
    );

    // Conductor(s)
    for kind in &[Kind::Summary, Kind::Individual, Kind::Individual] {
        assert_eq!(
            0,
            Actors::filter_kind_role(&actors, *kind, Role::Conductor).count()
        );
    }
    assert_eq!(None, Actors::main_actor(actors.iter(), Role::Conductor));
}

#[test]
fn validate_empty_actors() {
    let actors = [];
    assert!(Actors::validate(&actors.iter()).is_ok());
}

#[test]
fn actor_names() {
    assert!(is_valid_actor_name("A valid\nartist\tname"));
    assert!(!is_valid_actor_name(" Leading whitespace"));
    assert!(!is_valid_actor_name("Trailing whitespace\n"));
    assert!(!is_valid_actor_name(""));
    assert!(!is_valid_actor_name(" "));
    assert!(!is_valid_actor_name("\t"));
}

#[test]
fn summary_individual_actor_names() {
    assert!(is_valid_summary_individual_actor_name(
        "Artist 1 and artist 2",
        "Artist 1"
    ));
    assert!(is_valid_summary_individual_actor_name(
        "Artist 1 and artist 2",
        "artist 2"
    ));
    assert!(!is_valid_summary_individual_actor_name(
        "Artist 1 and artist 2",
        "artist 1"
    ));
    assert!(!is_valid_summary_individual_actor_name(
        "Artist 1 and artist 2",
        "Artist 2"
    ));
}

const ACTOR_NAME_SEPARATORS: &[&str] = &[
    " & ",
    " and ",
    " with ",
    ", ", // without leading whitespace
    " + ",
    " x ",
    " ft. ",
    " feat. ",
    " featuring ",
    " vs. ",
];

const PROTECTED_ACTOR_NAMES: &[&str] = &["simon & garfunkel", "tyler, the creator"];

#[test]
fn split_actor_names_summary() {
    let splitter = ActorNamesSummarySplitter::new(
        ACTOR_NAME_SEPARATORS.iter().copied(),
        PROTECTED_ACTOR_NAMES.iter().copied(),
    );
    assert_eq!(
        [
            "The Beatles",
            "Simon & Garfunkel",
            "Tyler, the Creator",
            "ABBA"
        ],
        splitter
            .split_all(" The Beatles, Simon & Garfunkel & Tyler, the Creator   ft. ABBA ")
            .collect::<Vec<_>>()
            .as_slice()
    );
}
