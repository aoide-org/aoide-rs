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

use test_log::test;

use crate::prelude::tests::*;

use super::*;

use aoide_core::{
    audio::DurationMs,
    collection::{Collection, Entity as CollectionEntity, MediaSourceConfig},
    entity::EntityHeaderTyped,
    media::{
        self,
        content::{AudioContentMetadata, ContentLink, ContentPath, ContentPathConfig},
    },
    track::{Entity as TrackEntity, EntityBody as TrackEntityBody, EntityUid as TrackUid, Track},
    util::{clock::DateTime, url::BaseUrl},
};

use aoide_repo::{
    collection::{EntityRepo as _, RecordId as CollectionId},
    media::source::{CollectionRepo as _, RecordId as MediaSourceId},
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
            media_source_config: MediaSourceConfig {
                content_path: ContentPathConfig::VirtualFilePath {
                    root_url: BaseUrl::parse_strict("file:///").unwrap(),
                },
            },
        };
        let db = establish_connection()?;
        let collection_entity =
            CollectionEntity::new(EntityHeaderTyped::initial_random(), collection);
        let collection_id = crate::Connection::new(&db)
            .insert_collection_entity(DateTime::now_utc(), &collection_entity)?;
        Ok(Self { db, collection_id })
    }

    fn create_media_sources_and_tracks(
        &self,
        count: usize,
    ) -> RepoResult<Vec<(MediaSourceId, TrackId, TrackUid)>> {
        let db = crate::Connection::new(&self.db);
        let mut created = Vec::with_capacity(count);
        for i in 0..count {
            let created_at = DateTime::now_local_or_utc();
            let media_source = media::Source {
                collected_at: created_at,
                content_link: ContentLink {
                    path: ContentPath::new(format!("/home/test/file{}.mp3", i)),
                    rev: None,
                },
                content_type: "audio/mpeg".parse().unwrap(),
                content_metadata_flags: Default::default(),
                content_metadata: AudioContentMetadata {
                    duration: Some(DurationMs::from_inner(i as f64)),
                    ..Default::default()
                }
                .into(),
                content_digest: None,
                artwork: Default::default(),
                advisory_rating: None,
            };
            let media_source_id = db
                .insert_media_source(self.collection_id, DateTime::now_utc(), &media_source)?
                .id;
            let track = Track::new_from_media_source(media_source);
            let entity_body = TrackEntityBody {
                track,
                updated_at: created_at,
                last_synchronized_rev: None,
            };
            let track_entity = TrackEntity::new(EntityHeaderTyped::initial_random(), entity_body);
            let track_id = db.insert_track_entity(media_source_id, &track_entity)?;
            created.push((media_source_id, track_id, track_entity.hdr.uid));
        }
        Ok(created)
    }

    fn create_playlists_with_track_entries(
        &self,
        track_count: usize,
    ) -> RepoResult<EntityWithEntries> {
        let db = crate::Connection::new(&self.db);
        let created_at = DateTime::now_local_or_utc();
        let playlist = Playlist {
            collected_at: created_at,
            title: "Playlist".into(),
            notes: None,
            kind: None,
            color: None,
            flags: Default::default(),
        };
        let playlist_entity = Entity::new(EntityHeaderTyped::initial_random(), playlist);
        let playlist_id =
            db.insert_playlist_entity(self.collection_id, DateTime::now_utc(), &playlist_entity)?;
        let media_sources_and_tracks = self.create_media_sources_and_tracks(track_count)?;
        let mut playlist_entries = Vec::with_capacity(track_count);
        for (i, (_, _, track_uid)) in media_sources_and_tracks.into_iter().enumerate() {
            let entry = Entry {
                added_at: DateTime::now_local_or_utc(),
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
        added_at: DateTime::now_local_or_utc(),
        title: None,
        notes: None,
        item: Item::Separator,
    }
}

fn new_separator_entry_with_title(title: String) -> Entry {
    Entry {
        added_at: DateTime::now_local_or_utc(),
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

#[test]
fn copy_all_entries() -> anyhow::Result<()> {
    let fixture = Fixture::new()?;
    let db = crate::Connection::new(&fixture.db);

    let track_count = 10;
    let source_entity_with_entries = fixture.create_playlists_with_track_entries(track_count)?;
    let (source_entity_header, source_playlist_with_entries) = source_entity_with_entries.into();
    let source_playlist_id = db.resolve_playlist_id(&source_entity_header.uid)?;
    let source_entries = source_playlist_with_entries.entries;
    assert_eq!(track_count, source_entries.len());

    let target_entity_with_entries = fixture.create_playlists_with_track_entries(0)?;
    let (target_entity_header, target_playlist_with_entries) = target_entity_with_entries.into();
    let target_playlist_id = db.resolve_playlist_id(&target_entity_header.uid)?;
    let target_entries = target_playlist_with_entries.entries;
    assert!(target_entries.is_empty());

    db.copy_all_playlist_entries(source_playlist_id, target_playlist_id)?;

    assert_eq!(
        source_entries,
        db.load_all_playlist_entries(target_playlist_id)?
    );

    Ok(())
}
