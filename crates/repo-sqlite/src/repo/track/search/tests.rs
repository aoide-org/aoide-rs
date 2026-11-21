// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use itertools::Itertools as _;
use nonicle::CanonicalizeInto as _;

use aoide_core::{
    Collection, FacetedTags, Track, TrackBody, TrackEntity, TrackHeader,
    audio::DurationMs,
    collection::{Entity as CollectionEntity, EntityHeader as CollectionHeader},
    media::{
        self,
        content::{AudioContentMetadata, ContentLink},
    },
    tag::{FacetKey, Label, PlainTag, TagsMap},
    track::tag::{FACET_ID_COMMENT, FACET_ID_GENRE},
    util::clock::{OffsetDateTimeMs, UtcDateTimeMs},
};
use aoide_core_api::{
    filter::{FilterModifier, StringPredicate},
    tag::search::{FacetsFilter, Filter as TagFilter},
    track::search::{Filter as TrackFilter, Scope, TitlePhraseFilter},
};
use aoide_repo::{
    CollectionId, RecordCollector, RepoResult, ReservableRecordCollector, TrackId,
    collection::EntityRepo as _,
    media::source::CollectionRepo as _,
    track::{CollectionRepo, EntityRepo as _, TagRepo as _},
};
use uuid::Uuid;

use crate::{
    repo::tests::vfs_media_source_config,
    tests::{TestResult, establish_connection},
};

struct DummyCollector<H, R> {
    _header: std::marker::PhantomData<H>,
    _record: std::marker::PhantomData<R>,
}

impl<H, R> DummyCollector<H, R> {
    const fn new() -> Self {
        Self {
            _header: std::marker::PhantomData,
            _record: std::marker::PhantomData,
        }
    }
}

impl<H, R> RecordCollector for DummyCollector<H, R> {
    type Header = H;
    type Record = R;

    fn collect(&mut self, _header: Self::Header, _record: Self::Record) {}
}

impl<H, R> ReservableRecordCollector for DummyCollector<H, R> {
    fn reserve(&mut self, _additional: usize) {}
}

fn create_single_track_collection_with_tags(
    db: &mut crate::Connection<'_>,
) -> TestResult<CollectionId> {
    let collection = Collection {
        title: "Collection".into(),
        notes: None,
        kind: None,
        color: None,
        media_source_config: vfs_media_source_config(),
    };
    let collected_at = OffsetDateTimeMs::now_local();
    let created_at = collected_at.to_utc();
    let collection_entity = CollectionEntity::new(CollectionHeader::initial_random(), collection);
    let collection_id = db.insert_collection_entity(created_at, &collection_entity)?;
    let media_source = media::Source {
        collected_at,
        content: media::Content {
            link: ContentLink {
                path: "/home/test/file.mp3".into(),
                rev: None,
            },
            r#type: "audio/mpeg".parse().unwrap(),
            metadata_flags: Default::default(),
            metadata: AudioContentMetadata {
                duration: Some(DurationMs::new(1.0)),
                ..Default::default()
            }
            .into(),
            digest: None,
        },
        artwork: Default::default(),
    };
    let media_source_id = db
        .insert_media_source(collection_id, created_at, &media_source)?
        .id;
    let mut track = Track::new_from_media_source(media_source);
    // Track title with Unicode characters.
    track.set_track_title("ÄäÖöÜüÉéÈè");
    // Album title with ASCII characters.
    track.set_album_title("AaOoUuEeEe");
    let plain_tags = (1..10)
        .flat_map(|i| {
            [
                PlainTag {
                    label: Some(Label::from_unchecked(format!("Tag\\{i}"))),
                    score: Default::default(),
                },
                PlainTag {
                    label: Some(Label::from_unchecked(format!("tag\\{i}"))),
                    score: Default::default(),
                },
            ]
        })
        .collect::<Vec<_>>();
    track.tags = [(FacetKey::unfaceted(), plain_tags)]
        .into_iter()
        .collect::<TagsMap<'_>>()
        .canonicalize_into();
    let entity_body = TrackBody {
        track,
        updated_at: created_at,
        last_synchronized_rev: None,
        content_url: None,
    };
    let track_entity = TrackEntity::new(TrackHeader::initial_random(), entity_body);
    db.insert_track_entity(media_source_id, &track_entity)?;
    Ok(collection_id)
}

#[test]
fn filter_plain_tags() -> TestResult<()> {
    let mut db = establish_connection()?;
    let mut db = crate::Connection::new(&mut db);
    let collection_id = create_single_track_collection_with_tags(&mut db)?;
    let filter = TrackFilter::Tag(TagFilter {
        modifier: None,
        facets: Some(FacetsFilter::AnyOf(vec![FacetKey::unfaceted()])),
        label: Some(StringPredicate::StartsWith("Tag\\".into())),
        score: None,
    });
    assert_eq!(
        1,
        db.search_tracks(
            collection_id,
            &Default::default(),
            Some(&filter),
            Default::default(),
            &mut DummyCollector::new(),
        )?
    );
    let filter = TrackFilter::Tag(TagFilter {
        modifier: None,
        facets: Some(FacetsFilter::NoneOf(vec![FACET_ID_COMMENT.into()])),
        label: Some(StringPredicate::StartsWith("tag\\".into())),
        score: None,
    });
    assert_eq!(
        1,
        db.search_tracks(
            collection_id,
            &Default::default(),
            Some(&filter),
            Default::default(),
            &mut DummyCollector::new(),
        )?
    );
    let filter = TrackFilter::Tag(TagFilter {
        modifier: None,
        facets: None,
        label: Some(StringPredicate::StartsNotWith("tag\\".into())),
        score: None,
    });
    assert_eq!(
        0,
        db.search_tracks(
            collection_id,
            &Default::default(),
            Some(&filter),
            Default::default(),
            &mut DummyCollector::new(),
        )?
    );
    for i in 1..10 {
        let filter = TrackFilter::Tag(TagFilter {
            modifier: None,
            facets: None,
            label: Some(StringPredicate::EndsWith(format!("\\{i}").into())),
            score: None,
        });
        assert_eq!(
            1,
            db.search_tracks(
                collection_id,
                &Default::default(),
                Some(&filter),
                Default::default(),
                &mut DummyCollector::new(),
            )?
        );
        let filter = TrackFilter::Tag(TagFilter {
            modifier: None,
            facets: None,
            label: Some(StringPredicate::EndsNotWith(format!("\\{i}").into())),
            score: None,
        });
        assert_eq!(
            1,
            db.search_tracks(
                collection_id,
                &Default::default(),
                Some(&filter),
                Default::default(),
                &mut DummyCollector::new(),
            )?
        );
    }
    Ok(())
}

#[test]
fn count_plain_tag_labels() -> TestResult<()> {
    let mut db = establish_connection()?;
    let mut db = crate::Connection::new(&mut db);
    let collection_id = create_single_track_collection_with_tags(&mut db)?;
    let expected_labels = (1..10)
        .flat_map(|i| [format!("Tag\\{i}"), format!("tag\\{i}")])
        .sorted()
        .collect::<Vec<_>>();

    let label_counts = db.count_track_tag_labels(Some(collection_id), None)?;
    assert_eq!(
        expected_labels,
        label_counts
            .iter()
            .map(|(label, _)| label.as_str())
            .collect::<Vec<_>>()
    );
    assert!(label_counts.iter().all(|(_, count)| *count == 1));

    // All collections
    assert_eq!(label_counts, db.count_track_tag_labels(None, None)?);

    // Only plain tags without a facet
    assert_eq!(
        Vec::<(Label<'_>, usize)>::new(),
        db.count_track_tag_labels(Some(collection_id), Some(FACET_ID_GENRE))?
    );

    Ok(())
}

#[test]
fn search_title_phrase() -> TestResult<()> {
    let mut db = establish_connection()?;
    let mut db = crate::Connection::new(&mut db);
    let collection_id = create_single_track_collection_with_tags(&mut db)?;
    // Case-insensitive ASCII search.
    assert_eq!(
        1,
        db.search_tracks(
            collection_id,
            &Default::default(),
            Some(&TrackFilter::TitlePhrase(TitlePhraseFilter {
                modifier: None,
                scope: Some(Scope::Album),
                kinds: Default::default(),
                name_terms: vec!["aA".into(), "EEee".into()],
            })),
            Default::default(),
            &mut DummyCollector::new(),
        )?
    );
    // Case-sensitive Unicode search.
    assert_eq!(
        1,
        db.search_tracks(
            collection_id,
            &Default::default(),
            Some(&TrackFilter::TitlePhrase(TitlePhraseFilter {
                modifier: None,
                scope: Some(Scope::Track),
                kinds: Default::default(),
                name_terms: vec!["Ää".into(), "ÉéÈè".into()],
            })),
            Default::default(),
            &mut DummyCollector::new(),
        )?
    );
    // Case-insensitive/folding Unicode search.
    assert_eq!(
        1,
        db.search_tracks(
            collection_id,
            &Default::default(),
            Some(&TrackFilter::TitlePhrase(TitlePhraseFilter {
                modifier: None,
                scope: Some(Scope::Track),
                kinds: Default::default(),
                name_terms: vec!["aA".into(), "EEee".into()],
            })),
            Default::default(),
            &mut DummyCollector::new(),
        )?
    );
    Ok(())
}

const GENRE_TAG_LABELS: &[&str] = &["Genre 1", "Genre 2"];

fn init_track_genre_tags(track: &mut Track, genre_tag_labels: &[&str]) {
    let mut tags = std::mem::take(&mut track.tags).untie();

    assert!(tags.is_empty());

    let genre_tags = genre_tag_labels
        .iter()
        .copied()
        .map(|label| {
            let tag_label = Label::clamp_from(label).unwrap();
            assert_eq!(tag_label.as_str(), label);
            PlainTag {
                label: Some(tag_label.into_owned()),
                ..Default::default()
            }
        })
        .collect::<Vec<_>>();
    tags.facets.push(FacetedTags {
        facet_id: FACET_ID_GENRE.clone(),
        tags: genre_tags,
    });
    let tags = tags.canonicalize_into();

    assert_eq!(tags.facets.is_empty(), genre_tag_labels.is_empty());
    assert_eq!(tags.total_count(), genre_tag_labels.len());

    track.tags = tags;
}

struct TracksWithGenreTagsFixture {
    collection_id: CollectionId,
}

impl TracksWithGenreTagsFixture {
    fn new(db: &mut crate::Connection<'_>) -> TestResult<Self> {
        let collection = Collection {
            title: "Collection".into(),
            notes: None,
            kind: None,
            color: None,
            media_source_config: vfs_media_source_config(),
        };
        let collection_entity =
            CollectionEntity::new(CollectionHeader::initial_random(), collection);
        let collection_id = crate::Connection::new(db)
            .insert_collection_entity(UtcDateTimeMs::now(), &collection_entity)?;

        let this = Self { collection_id };
        for genre_tag_labels in GENRE_TAG_LABELS
            .iter()
            .copied()
            .powerset()
            .collect::<Vec<Vec<_>>>()
        {
            this.create_media_source_and_track(db, &genre_tag_labels)?;
        }

        Ok(this)
    }

    fn create_media_source_and_track(
        &self,
        db: &mut crate::Connection<'_>,
        genre_tag_labels: &[&str],
    ) -> RepoResult<TrackId> {
        self.create_media_source_and_track_with(db, |track| {
            init_track_genre_tags(track, genre_tag_labels);
        })
    }

    fn create_media_source_and_track_with(
        &self,
        db: &mut crate::Connection<'_>,
        init_track: impl FnOnce(&mut Track),
    ) -> RepoResult<TrackId> {
        let collected_at = OffsetDateTimeMs::now_local();
        let created_at = collected_at.to_utc();
        let uuid = Uuid::now_v7();
        let media_source = media::Source {
            collected_at,
            content: media::Content {
                link: ContentLink {
                    path: format!("/home/test/file-{uuid}.mp3").into(),
                    rev: None,
                },
                r#type: "audio/mpeg".parse().unwrap(),
                metadata_flags: Default::default(),
                metadata: AudioContentMetadata {
                    duration: Some(DurationMs::new(150_000.0)),
                    ..Default::default()
                }
                .into(),
                digest: None,
            },
            artwork: Default::default(),
        };
        let media_source_id = db
            .insert_media_source(self.collection_id, UtcDateTimeMs::now(), &media_source)?
            .id;

        let mut track = Track::new_from_media_source(media_source);
        init_track(&mut track);

        let entity_body = TrackBody {
            track,
            updated_at: created_at,
            last_synchronized_rev: None,
            content_url: None,
        };
        let track_entity = TrackEntity::new(TrackHeader::initial_random(), entity_body);
        db.insert_track_entity(media_source_id, &track_entity)
    }
}

#[test]
fn find_tracks_with_missing_genre_tag() -> TestResult<()> {
    let mut db = establish_connection()?;
    let mut db = crate::Connection::new(&mut db);
    let fixture = TracksWithGenreTagsFixture::new(&mut db)?;

    let filter = TrackFilter::Tag(TagFilter {
        facets: Some(FacetsFilter::AnyOf(vec![FACET_ID_GENRE.into()])),
        modifier: Some(FilterModifier::Complement),
        ..Default::default()
    });
    let mut entities = Vec::new();
    db.search_tracks(
        fixture.collection_id,
        &Default::default(),
        Some(&filter),
        &[],
        &mut entities,
    )?;
    assert_eq!(entities.len(), 1);

    Ok(())
}

#[test]
fn find_no_tracks_with_empty_or_invalid_genre_tag_label() -> TestResult<()> {
    let mut db = establish_connection()?;
    let mut db = crate::Connection::new(&mut db);
    let fixture = TracksWithGenreTagsFixture::new(&mut db)?;

    for label in ["", " ", " Genre 1 "] {
        for label_predicate in [
            StringPredicate::Equals(label.into()),
            StringPredicate::Matches(label.into()),
        ] {
            let filter = TrackFilter::Tag(TagFilter {
                facets: Some(FacetsFilter::AnyOf(vec![FACET_ID_GENRE.into()])),
                label: Some(label_predicate),
                ..Default::default()
            });
            let mut entities = Vec::new();
            db.search_tracks(
                fixture.collection_id,
                &Default::default(),
                Some(&filter),
                &[],
                &mut entities,
            )?;
            assert_eq!(entities, Vec::new());
        }
    }

    Ok(())
}

/// Populate all children of a track that are stored in separate tables.
fn init_track_children(track: &mut Track, index: usize) {
    use aoide_core::{
        audio::PositionMs,
        track::{
            actor::{Actor, Kind as ActorKind, Role as ActorRole},
            cue::{Cue, InMarker},
        },
    };

    track.set_track_title(format!("Track title {index}"));
    track.set_album_title(format!("Album title {index}"));
    track.actors = vec![
        Actor {
            role: ActorRole::Artist,
            kind: ActorKind::Summary,
            name: format!("Track artist {index}"),
            role_notes: None,
        },
        Actor {
            role: ActorRole::Composer,
            kind: ActorKind::Individual,
            name: format!("Composer {index}"),
            role_notes: None,
        },
    ]
    .canonicalize_into();
    let mut album = std::mem::take(&mut track.album).untie();
    album.actors = vec![Actor {
        role: ActorRole::Artist,
        kind: ActorKind::Summary,
        name: format!("Album artist {index}"),
        role_notes: None,
    }]
    .canonicalize_into();
    track.album = nonicle::Canonical::tie(album);
    let cues = (0..=index % 3)
        .map(|slot| Cue {
            bank_index: 0,
            slot_index: Some(slot as _),
            in_marker: Some(InMarker {
                position: PositionMs::new((index * 1000 + slot) as f64),
            }),
            out_marker: None,
            kind: None,
            label: Some(format!("Cue {index}/{slot}")),
            color: None,
            flags: Default::default(),
        })
        .collect();
    track.cues = nonicle::Canonical::tie(cues);
    let genre = format!("Genre {index}");
    init_track_genre_tags(track, &[&genre]);
}

#[test]
fn search_tracks_preloads_children_of_each_track() -> TestResult<()> {
    let mut db = establish_connection()?;
    let mut db = crate::Connection::new(&mut db);
    let fixture = TracksWithGenreTagsFixture {
        collection_id: {
            let collection = Collection {
                title: "Collection".into(),
                notes: None,
                kind: None,
                color: None,
                media_source_config: vfs_media_source_config(),
            };
            let collection_entity =
                CollectionEntity::new(CollectionHeader::initial_random(), collection);
            db.insert_collection_entity(UtcDateTimeMs::now(), &collection_entity)?
        },
    };

    // Exceed a single chunk of preloaded tracks.
    let track_count = super::super::PRELOAD_TRACKS_CHUNK_SIZE + 2;
    let mut expected_tracks = Vec::with_capacity(track_count);
    for index in 0..track_count {
        let mut inserted_track = None;
        let track_id = fixture.create_media_source_and_track_with(&mut db, |track| {
            // Leave some tracks without any children.
            if index % 3 != 1 {
                init_track_children(track, index);
            }
            inserted_track = Some(track.clone());
        })?;
        let uid = db.load_track_entity(track_id)?.1.raw.hdr.uid;
        expected_tracks.push((uid, inserted_track.unwrap()));
    }

    let mut entities = Vec::new();
    let count = db.search_tracks(
        fixture.collection_id,
        &Default::default(),
        None,
        &[],
        &mut entities,
    )?;
    assert_eq!(count, track_count);
    assert_eq!(entities.len(), track_count);
    for ((_, entity), (expected_uid, expected_track)) in entities.iter().zip(&expected_tracks) {
        assert_eq!(entity.hdr.uid, *expected_uid);
        let track = &entity.body.track;
        assert_eq!(track.titles, expected_track.titles);
        assert_eq!(track.actors, expected_track.actors);
        assert_eq!(track.album, expected_track.album);
        assert_eq!(track.cues, expected_track.cues);
        assert_eq!(track.tags, expected_track.tags);
    }
    assert!(
        entities
            .iter()
            .any(|(_, entity)| entity.body.track.cues.len() == 3)
    );
    assert!(
        entities
            .iter()
            .any(|(_, entity)| entity.body.track.titles.is_empty())
    );

    Ok(())
}

#[test]
fn replace_track_updates_modified_children() -> TestResult<()> {
    use aoide_repo::track::{ReplaceMode, ReplaceOutcome, ReplaceParams};

    let mut db = establish_connection()?;
    let mut db = crate::Connection::new(&mut db);
    let fixture = TracksWithGenreTagsFixture {
        collection_id: {
            let collection = Collection {
                title: "Collection".into(),
                notes: None,
                kind: None,
                color: None,
                media_source_config: vfs_media_source_config(),
            };
            let collection_entity =
                CollectionEntity::new(CollectionHeader::initial_random(), collection);
            db.insert_collection_entity(UtcDateTimeMs::now(), &collection_entity)?
        },
    };
    let track_id = fixture
        .create_media_source_and_track_with(&mut db, |track| init_track_children(track, 2))?;
    let (_, entity) = db.load_track_entity(track_id)?;

    // Keep the titles and actors, modify the cues and tags.
    let mut track = entity.raw.body.track.clone();
    let mut modified = Track::new_from_media_source(track.media_source.clone());
    init_track_children(&mut modified, 1);
    track.cues = modified.cues;
    track.tags = modified.tags;
    assert_ne!(track, entity.body.track);

    let params = ReplaceParams {
        mode: ReplaceMode::UpdateOnly,
        preserve_collected_at: true,
        update_last_synchronized_rev: false,
    };
    let outcome = db.replace_track_by_media_source_content_path(
        fixture.collection_id,
        params,
        track.clone(),
    )?;
    assert!(matches!(outcome, ReplaceOutcome::Updated(..)));

    let (_, reloaded) = db.load_track_entity(track_id)?;
    let reloaded = &reloaded.body.track;
    assert_eq!(reloaded.titles, track.titles);
    assert_eq!(reloaded.actors, track.actors);
    assert_eq!(reloaded.album, track.album);
    assert_eq!(reloaded.cues, track.cues);
    assert_eq!(reloaded.tags, track.tags);
    assert_ne!(reloaded.cues, entity.body.track.cues);

    Ok(())
}

#[test]
fn load_recently_updated_track_headers_with_keyset_pagination() -> TestResult<()> {
    let mut db = establish_connection()?;
    let mut db = crate::Connection::new(&mut db);
    let fixture = TracksWithGenreTagsFixture {
        collection_id: {
            let collection = Collection {
                title: "Collection".into(),
                notes: None,
                kind: None,
                color: None,
                media_source_config: vfs_media_source_config(),
            };
            let collection_entity =
                CollectionEntity::new(CollectionHeader::initial_random(), collection);
            db.insert_collection_entity(UtcDateTimeMs::now(), &collection_entity)?
        },
    };
    let track_count = 10;
    for index in 0..track_count {
        fixture.create_media_source_and_track_with(&mut db, |track| {
            init_track_children(track, index);
        })?;
    }

    let all_headers =
        db.load_recently_updated_track_headers(fixture.collection_id, None, u64::MAX)?;
    assert_eq!(track_count, all_headers.len());
    for pair in all_headers.windows(2) {
        let (prev, _) = &pair[0];
        let (next, _) = &pair[1];
        assert!(
            prev.updated_at > next.updated_at
                || (prev.updated_at == next.updated_at && prev.id < next.id)
        );
    }

    // Paginate with a page size that doesn't divide the number of tracks.
    let mut paginated_headers = Vec::new();
    let mut after = None;
    loop {
        let page = db.load_recently_updated_track_headers(fixture.collection_id, after, 3)?;
        let Some((last, _)) = page.last() else {
            break;
        };
        after = Some((last.updated_at, last.id));
        paginated_headers.extend(page);
    }
    assert_eq!(all_headers, paginated_headers);

    // Load entities in the requested, reversed order.
    let ids = all_headers
        .iter()
        .rev()
        .map(|(record_header, _)| record_header.id)
        .collect::<Vec<_>>();
    let entities = db.load_track_entities(&ids)?;
    assert_eq!(ids.len(), entities.len());
    for (id, (record_header, entity)) in ids.iter().zip(&entities) {
        assert_eq!(*id, record_header.id);
        assert_eq!(db.load_track_entity(*id)?.1, *entity);
    }

    Ok(())
}
