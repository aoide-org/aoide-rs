// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
            kind: ActorKind::Individual,
            ..Default::default()
        },
        Actor {
            name: individual_artist_names[1].into(),
            kind: ActorKind::Individual,
            ..Default::default()
        },
        Actor {
            name: individual_producer_name.into(),
            role: ActorRole::Producer,
            kind: ActorKind::Individual,
            ..Default::default()
        },
        Actor {
            name: individual_artist_names[2].into(),
            kind: ActorKind::Individual,
            ..Default::default()
        },
    ];

    assert!(Actors::validate(actors.iter()).is_ok());

    // Artist(s)
    assert_eq!(
        &[summary_artist_name],
        Actors::filter_kind_role(actors.iter(), ActorKind::Summary, ActorRole::Artist)
            .map(|actor| actor.name.as_str())
            .collect::<Vec<_>>()
            .as_slice()
    );
    assert_eq!(
        individual_artist_names,
        Actors::filter_kind_role(actors.iter(), ActorKind::Individual, ActorRole::Artist)
            .map(|actor| actor.name.as_str())
            .collect::<Vec<_>>()
            .as_slice()
    );
    assert_eq!(
        summary_artist_name,
        Actors::main_actor(actors.iter(), ActorRole::Artist)
            .unwrap()
            .name
    );

    // Producer(s)
    assert_eq!(
        0,
        Actors::filter_kind_role(&actors, ActorKind::Summary, ActorRole::Producer).count()
    );
    assert_eq!(
        &[individual_producer_name],
        Actors::filter_kind_role(&actors, ActorKind::Individual, ActorRole::Producer)
            .map(|actor| actor.name.as_str())
            .collect::<Vec<_>>()
            .as_slice()
    );
    assert_eq!(
        individual_producer_name,
        Actors::main_actor(actors.iter(), ActorRole::Producer)
            .unwrap()
            .name
    );

    // Conductor(s)
    for kind in &[
        ActorKind::Summary,
        ActorKind::Individual,
        ActorKind::Individual,
    ] {
        assert_eq!(
            0,
            Actors::filter_kind_role(&actors, *kind, ActorRole::Conductor).count()
        );
    }
    assert_eq!(
        None,
        Actors::main_actor(actors.iter(), ActorRole::Conductor)
    );
}

#[test]
fn validate_empty_actors() {
    let actors = [];
    assert!(Actors::validate(actors.iter()).is_ok());
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
