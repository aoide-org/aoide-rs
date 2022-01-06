// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
    let primary_artist_name = "Madonna";
    let primary_producer_name = "Martin Solveig";
    let actors = vec![
        Actor {
            name: summary_artist_name.into(),
            ..Default::default()
        },
        Actor {
            name: primary_artist_name.into(),
            kind: ActorKind::Primary,
            ..Default::default()
        },
        Actor {
            name: "M.I.A.".into(),
            kind: ActorKind::Secondary,
            ..Default::default()
        },
        Actor {
            name: primary_producer_name.into(),
            role: ActorRole::Producer,
            kind: ActorKind::Primary,
            ..Default::default()
        },
        Actor {
            name: "Nicki Minaj".into(),
            kind: ActorKind::Secondary,
            ..Default::default()
        },
    ];

    assert!(Actors::validate(actors.iter()).is_ok());

    // Artist(s)
    assert_eq!(
        summary_artist_name,
        Actors::filter_kind_role(actors.iter(), ActorKind::Summary, ActorRole::Artist)
            .next()
            .unwrap()
            .name
    );
    assert_eq!(
        primary_artist_name,
        Actors::filter_kind_role(actors.iter(), ActorKind::Primary, ActorRole::Artist)
            .next()
            .unwrap()
            .name
    );
    // Not allowed to query for multiple secondary artists
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
        primary_producer_name,
        Actors::filter_kind_role(&actors, ActorKind::Primary, ActorRole::Producer)
            .next()
            .unwrap()
            .name
    );
    assert_eq!(
        0,
        Actors::filter_kind_role(&actors, ActorKind::Secondary, ActorRole::Producer).count()
    );
    assert_eq!(
        primary_producer_name,
        Actors::main_actor(actors.iter(), ActorRole::Producer)
            .unwrap()
            .name
    );

    // Conductor(s)
    for kind in &[ActorKind::Summary, ActorKind::Secondary, ActorKind::Primary] {
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
