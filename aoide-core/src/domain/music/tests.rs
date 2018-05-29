// Aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom>
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
            priority: ActorPriority::Primary,
            ..Default::default()
        },
        Actor {
            name: "M.I.A.".into(),
            priority: ActorPriority::Secondary,
            ..Default::default()
        },
        Actor {
            name: primary_producer_name.into(),
            role: ActorRole::Producer,
            priority: ActorPriority::Primary,
            ..Default::default()
        },
        Actor {
            name: "Nicki Minaj".into(),
            priority: ActorPriority::Secondary,
            ..Default::default()
        },
    ];

    assert!(Actors::is_valid(&actors));

    // Artist(s)
    assert_eq!(
        summary_artist_name,
        Actors::actor(&actors, ActorRole::Artist, ActorPriority::Summary)
            .unwrap()
            .name
    );
    assert_eq!(
        primary_artist_name,
        Actors::actor(&actors, ActorRole::Artist, ActorPriority::Primary)
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
        Actors::actor(&actors, ActorRole::Producer, ActorPriority::Summary)
    );
    assert_eq!(
        primary_producer_name,
        Actors::actor(&actors, ActorRole::Producer, ActorPriority::Primary)
            .unwrap()
            .name
    );
    assert_eq!(
        None,
        Actors::actor(&actors, ActorRole::Producer, ActorPriority::Secondary)
    );
    assert_eq!(
        primary_producer_name,
        Actors::main_actor(&actors, ActorRole::Producer)
            .unwrap()
            .name
    );

    // Conductor(s)
    for prio in [
        ActorPriority::Summary,
        ActorPriority::Primary,
        ActorPriority::Secondary,
    ].iter()
    {
        assert_eq!(None, Actors::actor(&actors, ActorRole::Conductor, *prio));
    }
    assert_eq!(None, Actors::main_actor(&actors, ActorRole::Conductor));
}
