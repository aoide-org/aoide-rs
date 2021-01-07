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

use crate::prelude::tests::*;

use aoide_core::{
    audio::{AudioContent, DurationMs},
    collection::{Collection, Entity as CollectionEntity},
    entity::EntityHeader,
    media,
    track::{Entity as TrackEntity, Track},
    util::clock::DateTime,
};

use aoide_repo::{
    collection::{EntityRepo as _, RecordId as CollectionId},
    media::source::{RecordId as MediaSourceId, Repo as _},
    track::RecordId as TrackId,
};

struct Fixture {
    db: SqliteConnection,
    collection_id: CollectionId,
}

impl Fixture {
    fn new() -> TestResult<Self> {
        let collection = Collection {
            title: "Collection".into(),
            notes: None,
            kind: None,
            color: None,
        };
        let db = establish_connection()?;
        let collection_entity = CollectionEntity::new(EntityHeader::initial_random(), collection);
        let collection_id = crate::Connection::new(&db)
            .insert_collection_entity(DateTime::now_utc(), &collection_entity)?;
        Ok(Self { db, collection_id })
    }

    fn create_media_sources_and_tracks(
        &self,
        count: usize,
    ) -> RepoResult<Vec<(MediaSourceId, TrackId, EntityUid)>> {
        let db = crate::Connection::new(&self.db);
        let mut created = Vec::with_capacity(count);
        for i in 0..count {
            let created_at = DateTime::now_local();
            let media_source = media::Source {
                collected_at: created_at,
                synchronized_at: Some(DateTime::now_utc()),
                uri: format!("file:///home/test/file{}.mp3", i),
                content_type: "audio/mpeg".to_string(),
                content_digest: None,
                content: AudioContent {
                    duration: Some(DurationMs(i as f64)),
                    ..Default::default()
                }
                .into(),
                artwork: Default::default(),
            };
            let media_source_id = db
                .insert_media_source(DateTime::now_utc(), self.collection_id, &media_source)?
                .id;
            let track = Track {
                media_source,
                tags: Default::default(),
                actors: Default::default(),
                titles: Default::default(),
                album: Default::default(),
                color: None,
                cues: Default::default(),
                indexes: Default::default(),
                metrics: Default::default(),
                play_counter: Default::default(),
                release: Default::default(),
            };
            let track_entity = TrackEntity::new(EntityHeader::initial_random(), track);
            let track_id = db.insert_track_entity(created_at, media_source_id, &track_entity)?;
            created.push((media_source_id, track_id, track_entity.hdr.uid));
        }
        Ok(created)
    }

    fn create_playlists_with_track_entries(
        &self,
        track_count: usize,
    ) -> RepoResult<EntityWithEntries> {
        let db = crate::Connection::new(&self.db);
        let created_at = DateTime::now_local();
        let playlist = Playlist {
            collected_at: created_at,
            title: "Playlist".into(),
            notes: None,
            kind: None,
            color: None,
            flags: Default::default(),
        };
        let playlist_entity = Entity::new(EntityHeader::initial_random(), playlist);
        let playlist_id = db.insert_collected_playlist_entity(
            self.collection_id,
            DateTime::now_utc(),
            &playlist_entity,
        )?;
        let media_sources_and_tracks = self.create_media_sources_and_tracks(track_count)?;
        let mut playlist_entries = Vec::with_capacity(track_count);
        for (i, (_, _, track_uid)) in media_sources_and_tracks.into_iter().enumerate() {
            let entry = Entry {
                added_at: DateTime::now_local(),
                title: Some(format!("Entry {}", i)),
                notes: None,
                item: Item::Track(track::Item { uid: track_uid }),
            };
            playlist_entries.push(entry);
        }
        db.append_playlist_entries(playlist_id, &playlist_entries)?;
        Ok((playlist_entity, playlist_entries).into())
    }
}

fn new_separator_entry() -> Entry {
    Entry {
        added_at: DateTime::now_local(),
        title: None,
        notes: None,
        item: Item::Separator,
    }
}

fn new_separator_entry_with_title(title: String) -> Entry {
    Entry {
        added_at: DateTime::now_local(),
        title: Some(title),
        notes: None,
        item: Item::Separator,
    }
}

#[test]
fn prepend_append_entries() -> anyhow::Result<()> {
    let fixture = Fixture::new()?;
    let db = crate::Connection::new(&fixture.db);

    let track_count = 100;
    let entity_with_entries = fixture.create_playlists_with_track_entries(track_count)?;
    let (entity_header, playlist_with_entries) = entity_with_entries.into();
    let track_entries = playlist_with_entries.entries;
    assert_eq!(track_count, track_entries.len());

    let playlist_id = db.resolve_playlist_id(&entity_header.uid)?;

    // Prepend entry
    let first_separator = new_separator_entry_with_title("First".to_string());
    db.prepend_playlist_entries(playlist_id, &[first_separator.clone()])?;
    let (_, playlist_with_entries) = db.load_playlist_entity_with_entries(playlist_id)?.into();
    assert_eq!(track_count + 1, playlist_with_entries.entries.len());
    assert_eq!(
        Some(&first_separator),
        playlist_with_entries.entries.first()
    );
    assert_eq!(&track_entries, &playlist_with_entries.entries[1..]);

    // Append entry
    let last_separator = new_separator_entry_with_title("Last".to_string());
    db.append_playlist_entries(playlist_id, &[last_separator.clone()])?;
    let (_, playlist_with_entries) = db.load_playlist_entity_with_entries(playlist_id)?.into();
    assert_eq!(track_count + 2, playlist_with_entries.entries.len());
    assert_eq!(
        Some(&first_separator),
        playlist_with_entries.entries.first()
    );
    assert_eq!(Some(&last_separator), playlist_with_entries.entries.last());
    assert_eq!(
        &track_entries,
        &playlist_with_entries.entries[1..playlist_with_entries.entries.len() - 1]
    );

    Ok(())
}

#[test]
fn should_not_modify_entries_when_moving_empty_ranges() -> anyhow::Result<()> {
    let fixture = Fixture::new()?;
    let db = crate::Connection::new(&fixture.db);

    let track_count = 10;
    let entity_with_entries = fixture.create_playlists_with_track_entries(track_count)?;
    let (entity_header, playlist_with_entries) = entity_with_entries.into();
    let track_entries = playlist_with_entries.entries;
    assert_eq!(track_count, track_entries.len());

    let playlist_id = db.resolve_playlist_id(&entity_header.uid)?;

    db.move_playlist_entries(playlist_id, &(0..0), 0)?;
    let (_, playlist_with_entries) = db.load_playlist_entity_with_entries(playlist_id)?.into();
    assert_eq!(track_entries, playlist_with_entries.entries);

    db.move_playlist_entries(playlist_id, &(0..0), 1)?;
    let (_, playlist_with_entries) = db.load_playlist_entity_with_entries(playlist_id)?.into();
    assert_eq!(track_entries, playlist_with_entries.entries);

    db.move_playlist_entries(playlist_id, &(0..0), -1)?;
    let (_, playlist_with_entries) = db.load_playlist_entity_with_entries(playlist_id)?.into();
    assert_eq!(track_entries, playlist_with_entries.entries);

    db.move_playlist_entries(playlist_id, &(0..0), track_count as isize + 1)?;
    let (_, playlist_with_entries) = db.load_playlist_entity_with_entries(playlist_id)?.into();
    assert_eq!(track_entries, playlist_with_entries.entries);

    db.move_playlist_entries(playlist_id, &(0..0), -(track_count as isize + 1))?;
    let (_, playlist_with_entries) = db.load_playlist_entity_with_entries(playlist_id)?.into();
    assert_eq!(track_entries, playlist_with_entries.entries);

    db.move_playlist_entries(playlist_id, &(100..100), 0)?;
    let (_, playlist_with_entries) = db.load_playlist_entity_with_entries(playlist_id)?.into();
    assert_eq!(track_entries, playlist_with_entries.entries);

    db.move_playlist_entries(playlist_id, &(100..100), 1)?;
    let (_, playlist_with_entries) = db.load_playlist_entity_with_entries(playlist_id)?.into();
    assert_eq!(track_entries, playlist_with_entries.entries);

    db.move_playlist_entries(playlist_id, &(100..100), track_count as isize + 1)?;
    let (_, playlist_with_entries) = db.load_playlist_entity_with_entries(playlist_id)?.into();
    assert_eq!(track_entries, playlist_with_entries.entries);

    db.move_playlist_entries(playlist_id, &(100..100), -1)?;
    let (_, playlist_with_entries) = db.load_playlist_entity_with_entries(playlist_id)?.into();
    assert_eq!(track_entries, playlist_with_entries.entries);

    db.move_playlist_entries(playlist_id, &(100..100), track_count as isize + 1)?;
    let (_, playlist_with_entries) = db.load_playlist_entity_with_entries(playlist_id)?.into();
    assert_eq!(track_entries, playlist_with_entries.entries);

    db.move_playlist_entries(playlist_id, &(100..100), -(track_count as isize + 1))?;
    let (_, playlist_with_entries) = db.load_playlist_entity_with_entries(playlist_id)?.into();
    assert_eq!(track_entries, playlist_with_entries.entries);

    Ok(())
}

#[test]
fn should_not_modify_entries_when_removing_empty_ranges() -> anyhow::Result<()> {
    let fixture = Fixture::new()?;
    let db = crate::Connection::new(&fixture.db);

    let track_count = 10;
    let entity_with_entries = fixture.create_playlists_with_track_entries(track_count)?;
    let (entity_header, playlist_with_entries) = entity_with_entries.into();
    let track_entries = playlist_with_entries.entries;
    assert_eq!(track_count, track_entries.len());

    let playlist_id = db.resolve_playlist_id(&entity_header.uid)?;

    db.remove_playlist_entries(playlist_id, &(0..0))?;
    let (_, playlist_with_entries) = db.load_playlist_entity_with_entries(playlist_id)?.into();
    assert_eq!(track_entries, playlist_with_entries.entries);

    // Non-overlapping range
    db.remove_playlist_entries(playlist_id, &(track_count..track_count + 1))?;
    let (_, playlist_with_entries) = db.load_playlist_entity_with_entries(playlist_id)?.into();
    assert_eq!(track_entries, playlist_with_entries.entries);

    Ok(())
}

#[test]
fn should_not_modify_entries_when_moving_by_zero_delta() -> anyhow::Result<()> {
    let fixture = Fixture::new()?;
    let db = crate::Connection::new(&fixture.db);

    let track_count = 10;
    let entity_with_entries = fixture.create_playlists_with_track_entries(track_count)?;
    let (entity_header, playlist_with_entries) = entity_with_entries.into();
    let track_entries = playlist_with_entries.entries;
    assert_eq!(track_count, track_entries.len());

    let playlist_id = db.resolve_playlist_id(&entity_header.uid)?;

    db.move_playlist_entries(playlist_id, &(0..1), 0)?;
    let (_, playlist_with_entries) = db.load_playlist_entity_with_entries(playlist_id)?.into();
    assert_eq!(track_entries, playlist_with_entries.entries);

    db.move_playlist_entries(playlist_id, &(0..track_count + 1), 0)?;
    let (_, playlist_with_entries) = db.load_playlist_entity_with_entries(playlist_id)?.into();
    assert_eq!(track_entries, playlist_with_entries.entries);

    db.move_playlist_entries(playlist_id, &(1..track_count + 1), 0)?;
    let (_, playlist_with_entries) = db.load_playlist_entity_with_entries(playlist_id)?.into();
    assert_eq!(track_entries, playlist_with_entries.entries);

    Ok(())
}

#[test]
fn move_entries_forward() -> anyhow::Result<()> {
    let fixture = Fixture::new()?;
    let db = crate::Connection::new(&fixture.db);

    let track_count = 10;
    let entity_with_entries = fixture.create_playlists_with_track_entries(track_count)?;
    let (entity_header, playlist_with_entries) = entity_with_entries.into();
    let track_entries = playlist_with_entries.entries;
    assert_eq!(track_count, track_entries.len());

    let playlist_id = db.resolve_playlist_id(&entity_header.uid)?;

    let moved_range = 1..track_count / 2 - 1;
    assert!(!moved_range.is_empty());
    db.insert_playlist_entries(playlist_id, moved_range.start, &[new_separator_entry()])?;
    db.insert_playlist_entries(playlist_id, moved_range.end - 1, &[new_separator_entry()])?;
    let (_, playlist_with_entries) = db.load_playlist_entity_with_entries(playlist_id)?.into();
    assert_eq!(track_count + 2, playlist_with_entries.entries.len());
    assert!(playlist_with_entries.entries[moved_range.start]
        .item
        .is_separator());
    assert!(playlist_with_entries.entries[moved_range.end - 1]
        .item
        .is_separator());

    let delta = (track_count / 2) as isize - 1;
    assert!(delta > 0);
    db.move_playlist_entries(playlist_id, &moved_range, delta)?;
    let (_, playlist_with_entries) = db.load_playlist_entity_with_entries(playlist_id)?.into();
    assert_eq!(track_count + 2, playlist_with_entries.entries.len());
    assert!(
        playlist_with_entries.entries[(moved_range.start as isize + delta) as usize]
            .item
            .is_separator()
    );
    assert!(
        playlist_with_entries.entries[(moved_range.end as isize + delta - 1) as usize]
            .item
            .is_separator()
    );

    Ok(())
}

#[test]
fn move_entries_forward_beyond_last_element() -> anyhow::Result<()> {
    let fixture = Fixture::new()?;
    let db = crate::Connection::new(&fixture.db);

    let track_count = 10;
    let entity_with_entries = fixture.create_playlists_with_track_entries(track_count)?;
    let (entity_header, playlist_with_entries) = entity_with_entries.into();
    let track_entries = playlist_with_entries.entries;
    assert_eq!(track_count, track_entries.len());

    let playlist_id = db.resolve_playlist_id(&entity_header.uid)?;

    let moved_range = track_count / 2..track_count - 1;
    assert!(!moved_range.is_empty());
    db.insert_playlist_entries(playlist_id, moved_range.start, &[new_separator_entry()])?;
    db.insert_playlist_entries(playlist_id, moved_range.end - 1, &[new_separator_entry()])?;
    let (_, playlist_with_entries) = db.load_playlist_entity_with_entries(playlist_id)?.into();
    assert_eq!(track_count + 2, playlist_with_entries.entries.len());
    assert!(playlist_with_entries.entries[moved_range.start]
        .item
        .is_separator());
    assert!(playlist_with_entries.entries[moved_range.end - 1]
        .item
        .is_separator());

    let delta = (track_count - moved_range.start) as isize + 1;
    assert!(delta > 0);
    db.move_playlist_entries(playlist_id, &moved_range, delta)?;
    let (_, playlist_with_entries) = db.load_playlist_entity_with_entries(playlist_id)?.into();
    assert_eq!(track_count + 2, playlist_with_entries.entries.len());
    assert!(
        playlist_with_entries.entries[playlist_with_entries.entries.len() - 1]
            .item
            .is_separator()
    );
    assert!(
        playlist_with_entries.entries[playlist_with_entries.entries.len() - moved_range.len()]
            .item
            .is_separator()
    );

    Ok(())
}

#[test]
fn move_entries_backward() -> anyhow::Result<()> {
    let fixture = Fixture::new()?;
    let db = crate::Connection::new(&fixture.db);

    let track_count = 10;
    let entity_with_entries = fixture.create_playlists_with_track_entries(track_count)?;
    let (entity_header, playlist_with_entries) = entity_with_entries.into();
    let track_entries = playlist_with_entries.entries;
    assert_eq!(track_count, track_entries.len());

    let playlist_id = db.resolve_playlist_id(&entity_header.uid)?;

    let moved_range = track_count / 2..track_count - 1;
    assert!(!moved_range.is_empty());
    db.insert_playlist_entries(playlist_id, moved_range.start, &[new_separator_entry()])?;
    db.insert_playlist_entries(playlist_id, moved_range.end - 1, &[new_separator_entry()])?;
    let (_, playlist_with_entries) = db.load_playlist_entity_with_entries(playlist_id)?.into();
    assert_eq!(track_count + 2, playlist_with_entries.entries.len());
    assert!(playlist_with_entries.entries[moved_range.start]
        .item
        .is_separator());
    assert!(playlist_with_entries.entries[moved_range.end - 1]
        .item
        .is_separator());

    assert!(moved_range.start > 0); // otherwise the range cannot be moved backwards
    let delta = -(moved_range.start as isize - 1);
    assert!(delta < 0);
    db.move_playlist_entries(playlist_id, &moved_range, delta)?;
    let (_, playlist_with_entries) = db.load_playlist_entity_with_entries(playlist_id)?.into();
    assert_eq!(track_count + 2, playlist_with_entries.entries.len());
    assert!(
        playlist_with_entries.entries[(moved_range.start as isize + delta) as usize]
            .item
            .is_separator()
    );
    assert!(
        playlist_with_entries.entries[(moved_range.end as isize + delta - 1) as usize]
            .item
            .is_separator()
    );

    Ok(())
}
