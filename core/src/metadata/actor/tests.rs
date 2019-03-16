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
            ..Default::default()
        },
        Actor {
            name: "Nicki Minaj".into(),
            precedence: ActorPrecedence::Secondary,
            ..Default::default()
        },
    ];

    assert!(Actors::is_valid(&actors));

    // Artist(s)
    assert_eq!(
        summary_artist_name,
        Actors::actor(&actors, ActorRole::Artist, ActorPrecedence::Summary)
            .unwrap()
            .name
    );
    assert_eq!(
        primary_artist_name,
        Actors::actor(&actors, ActorRole::Artist, ActorPrecedence::Primary)
            .unwrap()
            .name
    );
    // Not allowed to query for multiple secondary artists
    assert_eq!(
        summary_artist_name,
        Actors::main_actor(&actors, ActorRole::Artist).unwrap().name
    );

    // Producer(s)
    assert_eq!(
        None,
        Actors::actor(&actors, ActorRole::Producer, ActorPrecedence::Summary)
    );
    assert_eq!(
        primary_producer_name,
        Actors::actor(&actors, ActorRole::Producer, ActorPrecedence::Primary)
            .unwrap()
            .name
    );
    assert_eq!(
        None,
        Actors::actor(&actors, ActorRole::Producer, ActorPrecedence::Secondary)
    );
    assert_eq!(
        primary_producer_name,
        Actors::main_actor(&actors, ActorRole::Producer)
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
        assert_eq!(None, Actors::actor(&actors, ActorRole::Conductor, *prio));
    }
    assert_eq!(None, Actors::main_actor(&actors, ActorRole::Conductor));
}
