// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Performance tests of the track search with synthetic tracks.
//!
//! Searches are performed exactly like the app does, i.e. with the filter
//! parsed from the input and the default parameters and page size.
//!
//! Run in release mode:
//! `cargo test --release -p aoide-file-collection-app -- --ignored --nocapture bench_track_search`

use std::{
    num::NonZeroU32,
    path::PathBuf,
    time::{Duration, Instant},
};

use diesel::Connection as _;
use nonicle::Canonical;

use aoide::{
    Collection, CollectionEntity, CollectionHeader, CollectionUid, Track, TrackBody, TrackEntity,
    TrackHeader,
    api::Pagination,
    audio::DurationMs,
    collection::MediaSourceConfig,
    media::{
        self,
        content::{AudioContentMetadata, ContentLink, ContentPathConfig, VirtualFilePathConfig},
    },
    music::tempo::TempoBpm,
    prelude::*,
    repo::{
        collection::EntityRepo as _,
        media::source::CollectionRepo as _,
        track::{EntityRepo as _, RecordHeader},
    },
    repo_sqlite::DbConnection,
    storage_sqlite::connection::{
        Storage,
        pool::{
            ConnectionPool, InitializeConnection, create_connection_pool, get_pooled_connection,
        },
    },
    tag::{FacetKey, Label, PlainTag, TagsMap},
    track::{
        actor::{self, Actor},
        tag::{FACET_ID_COMMENT, FACET_ID_GENRE, FACET_ID_GROUPING, FACET_ID_MOOD},
    },
    util::{
        clock::{OffsetDateTimeMs, UtcDateTimeMs},
        url::BaseUrl,
    },
};

use super::{DEFAULT_PREFETCH_LIMIT, default_params, parse_filter_from_input};

const TRACK_COUNT: usize = 100_000;

const RUN_COUNT: u32 = 3;

/// Optional path of the database file with the synthetic tracks.
///
/// The database is created if the file does not exist yet and is kept
/// for subsequent runs. Otherwise a temporary database is created and
/// deleted afterwards.
const DATABASE_ENV_VAR: &str = "AOIDE_BENCH_TRACK_SEARCH_DATABASE";

/// Search inputs as entered by the user.
const SEARCH_INPUTS: &[&str] = &[
    // Common term
    "love",
    // Diacritics-insensitive term
    "beyonce",
    // Term without any matches
    "xyz",
    // Short term
    "lo",
    // Multiple terms
    "deep house",
    // Plain tag
    "#vinyl",
    // Term and tempo
    "love +128",
];

/// Words for titles, artists, albums and comments.
///
/// Mostly ASCII text with some non-ASCII characters.
const WORDS: &[&str] = &[
    "Love",
    "Night",
    "Dancing",
    "Summer",
    "Deep",
    "House",
    "Club",
    "Mix",
    "Remix",
    "Edit",
    "Original",
    "Feat.",
    "Tribe",
    "Sound",
    "Heart",
    "City",
    "Dream",
    "Fire",
    "Light",
    "Soul",
    "Funk",
    "Dub",
    "Disco",
    "Groove",
    "Motion",
    "Rhythm",
    "Space",
    "Time",
    "Water",
    "Gold",
    "Silver",
    "Midnight",
    "Sunrise",
    "Shadow",
    "Echo",
    "Wave",
    "Storm",
    "River",
    "Ocean",
    "Sky",
    "Beyoncé",
    "Señorita",
    "Café",
    "Über",
    "Mötley",
    "Crüe",
    "Sigur",
    "Rós",
    "Déjà",
    "Vu",
];

/// Syllables for generating additional words.
///
/// Avoid combinations that contain the search term without matches.
const SYLLABLES: &[&str] = &[
    "ka", "lo", "ve", "ri", "na", "so", "ul", "te", "mo", "da", "ni", "be", "yo", "ce", "ra", "pu",
    "shi", "ta", "mé", "ño", "zu", "fa", "gi", "ho",
];

const GENRES: &[&str] = &[
    "House",
    "Deep House",
    "Tech House",
    "Techno",
    "Minimal",
    "Disco",
    "Nu Disco",
    "Funk",
    "Soul",
    "Jazz",
    "Hip-Hop",
    "Drum & Bass",
    "Dubstep",
    "Ambient",
    "Electronica",
    "Pop",
    "Rock",
    "Reggae",
    "Latin",
    "Afro House",
];

const MOODS: &[&str] = &[
    "Happy",
    "Dark",
    "Melancholic",
    "Energetic",
    "Chill",
    "Uplifting",
];

const PLAIN_TAGS: &[&str] = &[
    "vinyl",
    "favorite",
    "warmup",
    "peaktime",
    "closing",
    "classic",
    "promo",
    "bootleg",
    "live",
    "remastered",
];

const PUBLISHERS: &[&str] = &[
    "Señorita Records",
    "Deep Groove Music",
    "Midnight Recordings",
    "Über Sound",
    "Ocean Waves",
    "Gold Standard",
];

/// Deterministic pseudo-random numbers (`SplitMix64`).
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    #[expect(clippy::cast_possible_truncation)]
    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }

    fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.below(items.len())]
    }
}

struct Vocabulary {
    words: Vec<String>,
    artists: Vec<String>,
    albums: Vec<String>,
}

impl Vocabulary {
    fn new(rng: &mut Rng) -> Self {
        let mut words: Vec<_> = WORDS.iter().map(ToString::to_string).collect();
        // Generated words outnumber the predefined words.
        for _ in 0..2_000 {
            let mut word = String::new();
            for _ in 0..2 + rng.below(2) {
                word.push_str(rng.pick(SYLLABLES));
            }
            let mut chars = word.chars();
            let capitalized = chars
                .next()
                .into_iter()
                .flat_map(char::to_uppercase)
                .chain(chars)
                .collect();
            words.push(capitalized);
        }
        let mut vocabulary = Self {
            words,
            artists: Vec::new(),
            albums: Vec::new(),
        };
        vocabulary.artists = (0..10_000).map(|_| vocabulary.phrase(rng, 1, 2)).collect();
        vocabulary.albums = (0..20_000).map(|_| vocabulary.phrase(rng, 1, 3)).collect();
        vocabulary
    }

    fn word(&self, rng: &mut Rng) -> &str {
        // Prefer the predefined words.
        if rng.chance(30) {
            &self.words[rng.below(WORDS.len())]
        } else {
            &self.words[rng.below(self.words.len())]
        }
    }

    fn phrase(&self, rng: &mut Rng, min_words: usize, max_words: usize) -> String {
        let word_count = min_words + rng.below(max_words - min_words + 1);
        (0..word_count)
            .map(|_| self.word(rng))
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn artist(&self, rng: &mut Rng) -> &str {
        &self.artists[rng.below(self.artists.len())]
    }

    fn album(&self, rng: &mut Rng) -> &str {
        &self.albums[rng.below(self.albums.len())]
    }
}

fn summary_artist(name: impl Into<String>) -> Actor {
    Actor {
        role: actor::Role::Artist,
        kind: actor::Kind::Summary,
        name: name.into(),
        role_notes: None,
    }
}

fn plain_tags<'a>(labels: impl IntoIterator<Item = &'a str>) -> Vec<PlainTag<'static>> {
    labels
        .into_iter()
        .map(|label| PlainTag {
            label: Some(Label::from_unchecked(label.to_owned())),
            score: Default::default(),
        })
        .collect()
}

#[expect(clippy::cast_precision_loss)]
fn new_synthetic_track(
    rng: &mut Rng,
    vocabulary: &Vocabulary,
    media_source: media::Source,
) -> Track {
    let mut track = Track::new_from_media_source(media_source);
    let mut title = vocabulary.phrase(rng, 1, 4);
    if rng.chance(20) {
        title = format!("{title} ({})", vocabulary.phrase(rng, 1, 2));
    }
    track.set_track_title(title);
    track.set_album_title(vocabulary.album(rng));
    let artist = vocabulary.artist(rng).to_owned();
    let mut actors = vec![summary_artist(&artist)];
    if rng.chance(30) {
        // Featured artist
        let featured = vocabulary.artist(rng);
        actors[0].name = format!("{artist} feat. {featured}");
        for name in [artist.as_str(), featured] {
            actors.push(Actor {
                kind: actor::Kind::Individual,
                ..summary_artist(name)
            });
        }
    }
    track.actors = actors.canonicalize_into();
    let mut album = std::mem::take(&mut track.album).untie();
    album.actors = vec![summary_artist(if rng.chance(80) {
        artist.as_str()
    } else {
        "Various Artists"
    })]
    .canonicalize_into();
    track.album = Canonical::tie(album);
    if rng.chance(70) {
        track.publisher = Some(rng.pick(PUBLISHERS).to_owned());
    }
    track.metrics.tempo_bpm = Some(TempoBpm::new(70.0 + rng.below(110) as f64));
    let mut tags = vec![(
        FacetKey::from(FACET_ID_GENRE),
        plain_tags((0..=rng.below(2)).map(|_| rng.pick(GENRES))),
    )];
    if rng.chance(50) {
        tags.push((FacetKey::from(FACET_ID_MOOD), plain_tags([rng.pick(MOODS)])));
    }
    if rng.chance(50) {
        let comment = vocabulary.phrase(rng, 3, 8);
        tags.push((
            FacetKey::from(FACET_ID_COMMENT),
            plain_tags([comment.as_str()]),
        ));
    }
    if rng.chance(20) {
        let grouping = vocabulary.phrase(rng, 1, 2);
        tags.push((
            FacetKey::from(FACET_ID_GROUPING),
            plain_tags([grouping.as_str()]),
        ));
    }
    tags.push((
        FacetKey::unfaceted(),
        plain_tags((0..rng.below(4)).map(|_| rng.pick(PLAIN_TAGS))),
    ));
    track.tags = tags
        .into_iter()
        .collect::<TagsMap<'_>>()
        .canonicalize_into();
    track
}

fn create_synthetic_collection(connection: &mut DbConnection) -> anyhow::Result<CollectionUid> {
    let mut rng = Rng(0x00a0_1de0);
    let vocabulary = Vocabulary::new(&mut rng);
    connection.transaction::<_, anyhow::Error, _>(|connection| {
        let mut repo = aoide::repo_sqlite::Connection::new(connection);
        let collection = Collection {
            title: "Synthetic".into(),
            notes: None,
            kind: None,
            color: None,
            media_source_config: MediaSourceConfig {
                content_path: ContentPathConfig::VirtualFilePath(VirtualFilePathConfig {
                    root_url: BaseUrl::parse_strict("file:///music/")?,
                    excluded_paths: Vec::new(),
                }),
            },
        };
        let collection_entity =
            CollectionEntity::new(CollectionHeader::initial_random(), collection);
        let now = UtcDateTimeMs::now();
        let collection_id = repo.insert_collection_entity(now, &collection_entity)?;
        for i in 0..TRACK_COUNT {
            // Spread the update time stamps for sorting.
            let updated_at = UtcDateTimeMs::from_unix_timestamp_millis(
                now.unix_timestamp_millis() - 60_000 * i64::try_from(rng.below(TRACK_COUNT))?,
            );
            let media_source = media::Source {
                collected_at: OffsetDateTimeMs::from_utc(updated_at),
                content: media::Content {
                    link: ContentLink {
                        path: format!("{i:06}.mp3").into(),
                        rev: None,
                    },
                    r#type: "audio/mpeg".parse()?,
                    metadata_flags: Default::default(),
                    metadata: AudioContentMetadata {
                        duration: Some(DurationMs::new(180_000.0)),
                        ..Default::default()
                    }
                    .into(),
                    digest: None,
                },
                artwork: Default::default(),
            };
            let media_source_id = repo
                .insert_media_source(collection_id, updated_at, &media_source)?
                .id;
            let track = new_synthetic_track(&mut rng, &vocabulary, media_source);
            let entity_body = TrackBody {
                track,
                updated_at,
                last_synchronized_rev: None,
                content_url: None,
            };
            let track_entity = TrackEntity::new(TrackHeader::initial_random(), entity_body);
            repo.insert_track_entity(media_source_id, &track_entity)?;
        }
        Ok(collection_entity.hdr.uid.clone())
    })
}

fn connection_pool(
    storage: &Storage,
    initialize_connection: InitializeConnection,
) -> anyhow::Result<ConnectionPool> {
    create_connection_pool(storage, NonZeroU32::MIN, initialize_connection).map_err(Into::into)
}

/// Provision the database like the app does.
fn provision_database(storage: &Storage) -> anyhow::Result<ConnectionPool> {
    let pool = connection_pool(storage, aoide::repo_sqlite::initialize_connection)?;
    let mut connection = get_pooled_connection(&pool)?;
    aoide::repo_sqlite::initialize_database(&mut connection)?;
    aoide::usecases_sqlite::database::migrate_schema(&mut connection)?;
    drop(connection);
    Ok(pool)
}

fn load_collection_uid(connection: &mut DbConnection) -> anyhow::Result<Option<CollectionUid>> {
    let mut collections = Vec::new();
    aoide::usecases_sqlite::collection::load_all(
        connection,
        None,
        None,
        aoide::api::collection::LoadScope::Entity,
        None,
        &mut collections,
    )?;
    Ok(collections
        .into_iter()
        .next()
        .map(|(_, collection)| collection.entity.hdr.uid.clone()))
}

/// Average execution time and number of results of the first page.
fn bench_search(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    input: &str,
) -> anyhow::Result<(Duration, usize)> {
    let params = aoide::api::track::search::Params {
        filter: parse_filter_from_input(input),
        ..default_params()
    };
    let pagination = Pagination {
        limit: Some(DEFAULT_PREFETCH_LIMIT.get().try_into()?),
        offset: None,
    };
    let mut search = || -> anyhow::Result<usize> {
        let mut collector = Vec::<(RecordHeader, TrackEntity)>::new();
        aoide::usecases_sqlite::track::search::search(
            connection,
            collection_uid,
            &params,
            &pagination,
            &mut collector,
        )
        .map_err(Into::into)
    };
    // Warm up the page cache.
    search()?;
    let started = Instant::now();
    let mut count = 0;
    for _ in 0..RUN_COUNT {
        count = search()?;
    }
    Ok((started.elapsed() / RUN_COUNT, count))
}

#[test]
#[ignore = "benchmark"]
fn bench_track_search() -> anyhow::Result<()> {
    let (path, is_temporary) = std::env::var_os(DATABASE_ENV_VAR).map_or_else(
        || {
            let file_name = format!("aoide_bench_track_search_{}.sqlite", std::process::id());
            (std::env::temp_dir().join(file_name), true)
        },
        |path| (PathBuf::from(path), false),
    );
    let result = run_bench_track_search(&path);
    if is_temporary {
        for suffix in ["", "-wal", "-shm"] {
            let mut file_path = path.clone().into_os_string();
            file_path.push(suffix);
            // The file might not exist.
            drop(std::fs::remove_file(file_path));
        }
    }
    result
}

fn run_bench_track_search(path: &std::path::Path) -> anyhow::Result<()> {
    let storage = Storage::File {
        path: path.to_owned(),
    };
    let pool = provision_database(&storage)?;
    let mut connection = get_pooled_connection(&pool)?;
    let collection_uid = if let Some(collection_uid) = load_collection_uid(&mut connection)? {
        println!("Reusing synthetic tracks in {}", path.display());
        collection_uid
    } else {
        let started = Instant::now();
        let collection_uid = create_synthetic_collection(&mut connection)?;
        println!(
            "Created {TRACK_COUNT} synthetic tracks in {} in {:.1?}",
            path.display(),
            started.elapsed()
        );
        aoide::storage_sqlite::analyze_and_optimize_database_stats(&mut connection)?;
        collection_uid
    };
    // A connection that has not been initialized uses the built-in LIKE.
    let builtin_like_pool = connection_pool(&storage, |_| Ok(()))?;
    let mut builtin_like_connection = get_pooled_connection(&builtin_like_pool)?;
    println!(
        "{:<12} {:>13} {:>13} {:>8}",
        "input", "custom LIKE", "built-in LIKE", "results"
    );
    for input in SEARCH_INPUTS {
        let (custom_elapsed, custom_count) = bench_search(&mut connection, &collection_uid, input)?;
        let (builtin_elapsed, builtin_count) =
            bench_search(&mut builtin_like_connection, &collection_uid, input)?;
        println!(
            "{input:<12} {custom_elapsed:>13.1?} {builtin_elapsed:>13.1?} {custom_count:>8} \
             (built-in: {builtin_count})"
        );
    }
    Ok(())
}
