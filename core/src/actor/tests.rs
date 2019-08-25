// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
            precedence: ActorPrecedence::Primary,
            ..Default::default()
        },
        Actor {
            name: "M.I.A.".into(),
            precedence: ActorPrecedence::Secondary,
            ..Default::default()
        },
        Actor {
            name: primary_producer_name.into(),
            role: ActorRole::Producer,
            precedence: ActorPrecedence::Primary,
        },
        Actor {
            name: "Nicki Minaj".into(),
            precedence: ActorPrecedence::Secondary,
            ..Default::default()
        },
    ];

    assert!(Actors::validate(actors.iter()).is_ok());

    // Artist(s)
    assert_eq!(
        summary_artist_name,
        Actors::filter_role_precedence(actors.iter(), ActorRole::Artist, ActorPrecedence::Summary)
            .next()
            .unwrap()
            .name
    );
    assert_eq!(
        primary_artist_name,
        Actors::filter_role_precedence(actors.iter(), ActorRole::Artist, ActorPrecedence::Primary)
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
        Actors::filter_role_precedence(&actors, ActorRole::Producer, ActorPrecedence::Summary)
            .count()
    );
    assert_eq!(
        primary_producer_name,
        Actors::filter_role_precedence(&actors, ActorRole::Producer, ActorPrecedence::Primary)
            .next()
            .unwrap()
            .name
    );
    assert_eq!(
        0,
        Actors::filter_role_precedence(&actors, ActorRole::Producer, ActorPrecedence::Secondary)
            .count()
    );
    assert_eq!(
        primary_producer_name,
        Actors::main_actor(actors.iter(), ActorRole::Producer)
            .unwrap()
            .name
    );

    // Conductor(s)
    for prio in [
        ActorPrecedence::Summary,
        ActorPrecedence::Primary,
        ActorPrecedence::Secondary,
    ]
    .iter()
    {
        assert_eq!(
            0,
            Actors::filter_role_precedence(&actors, ActorRole::Conductor, *prio).count()
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
