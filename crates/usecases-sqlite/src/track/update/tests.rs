// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    fs::File,
    path::Path,
    sync::atomic::AtomicBool,
    time::{Duration, SystemTime},
};

use anyhow::anyhow;
use diesel::Connection as _;
use nonicle::CanonicalizeInto as _;
use url::Url;

use aoide_core::{
    Collection, CollectionUid, PlainTag, TagFacetId, TagLabel, TagsMap, Track, TrackEntity,
    collection::MediaSourceConfig,
    media::content::{ContentPath, ContentPathConfig, VirtualFilePathConfig},
    tag::FacetKey,
    track::tag::FACET_ID_COMMENT,
    util::url::BaseUrl,
};
use aoide_core_api::media::SyncMode;
use aoide_media_file::io::{export::ExportTrackConfig, import::ImportTrackConfig};
use aoide_repo::track::ReplaceMode;
use aoide_repo_sqlite::{DbConnection, initialize_database, run_migrations};
use aoide_usecases::track::validate_input;

use super::{Conflict, ExportParams, Outcome, Params, update_one};

const ASSET_FILE_PATH: &str = "../media-file/tests/assets/empty.mp3";

const FILE_NAME: &str = "track.mp3";

struct Fixture {
    connection: DbConnection,
    collection_uid: CollectionUid,
    root_dir: tempfile::TempDir,
}

impl Fixture {
    fn new() -> anyhow::Result<Self> {
        let mut connection = DbConnection::establish(":memory:")?;
        initialize_database(&mut connection)?;
        run_migrations(&mut connection).map_err(|err| anyhow!(err))?;
        let root_dir = tempfile::tempdir()?;
        std::fs::copy(ASSET_FILE_PATH, root_dir.path().join(FILE_NAME))?;
        let root_url = Url::from_directory_path(root_dir.path())
            .map_err(|()| anyhow!("invalid root directory"))?;
        let collection = Collection {
            title: "Test Collection".into(),
            notes: None,
            kind: None,
            color: None,
            media_source_config: MediaSourceConfig {
                content_path: ContentPathConfig::VirtualFilePath(VirtualFilePathConfig {
                    root_url: BaseUrl::new(root_url),
                    excluded_paths: Vec::new(),
                }),
            },
        };
        let collection_uid = crate::collection::create(&mut connection, collection)?
            .hdr
            .uid
            .clone();
        Ok(Self {
            connection,
            collection_uid,
            root_dir,
        })
    }

    fn file_path(&self) -> std::path::PathBuf {
        self.root_dir.path().join(FILE_NAME)
    }

    fn import_track(&mut self) -> anyhow::Result<TrackEntity> {
        let params = aoide_usecases::track::import_and_replace::Params {
            sync_mode: SyncMode::Modified,
            import_config: ImportTrackConfig::default(),
            replace_mode: ReplaceMode::UpdateOrCreate,
        };
        let outcome = crate::track::import_and_replace::import_and_replace_many_by_local_file_path(
            &mut self.connection,
            &self.collection_uid,
            [ContentPath::new(FILE_NAME.into())],
            1,
            &params,
            &std::convert::identity,
            &AtomicBool::new(false),
        )?;
        let [entity] = <[_; 1]>::try_from(outcome.summary.created)
            .map_err(|_| anyhow!("track not created"))?;
        Ok(entity)
    }

    fn update_track(
        &mut self,
        entity: &TrackEntity,
        track: Track,
        write_file: bool,
    ) -> anyhow::Result<Outcome> {
        let (validated_input, _) = validate_input(track)?;
        let export = write_file.then(|| ExportParams {
            export_config: ExportTrackConfig {
                encode_gigtags: Some(FACET_ID_COMMENT.clone()),
                ..Default::default()
            },
            import_config: ImportTrackConfig::default(),
        });
        let params = Params { export };
        let outcome = self.connection.transaction(|connection| {
            update_one(
                connection,
                &self.collection_uid,
                &entity.hdr,
                validated_input,
                &params,
            )
        })?;
        Ok(outcome)
    }
}

fn custom_tags() -> TagsMap<'static> {
    let mut tags_map = TagsMap::default();
    tags_map.insert_one(
        FacetKey::unfaceted(),
        PlainTag {
            label: TagLabel::clamp_from("warmup"),
            ..Default::default()
        },
    );
    tags_map.insert_one(
        FacetKey::new(TagFacetId::clamp_from("venue")),
        PlainTag {
            label: TagLabel::clamp_from("club-x"),
            ..Default::default()
        },
    );
    tags_map
}

fn edit_track(track: &Track) -> Track {
    let mut track = track.clone();
    track.set_track_title("Edited title");
    track.set_track_artist("Edited artist".to_owned());
    let mut tags_map: TagsMap<'static> = std::mem::take(&mut track.tags).untie().into();
    tags_map.insert_one(
        FacetKey::from(FACET_ID_COMMENT.clone()),
        PlainTag {
            label: TagLabel::clamp_from("Some comment"),
            ..Default::default()
        },
    );
    tags_map.merge(custom_tags());
    track.tags = tags_map.canonicalize_into();
    track
}

fn touch_file(file_path: &Path) -> anyhow::Result<()> {
    let modified = File::open(file_path)?.metadata()?.modified()?;
    let file = File::options().write(true).open(file_path)?;
    file.set_modified(modified.max(SystemTime::now()) + Duration::from_secs(10))?;
    Ok(())
}

#[test]
fn update_and_write_file_with_custom_tags() -> anyhow::Result<()> {
    let mut fixture = Fixture::new()?;
    let imported_entity = fixture.import_track()?;
    let edited_track = edit_track(&imported_entity.body.track);

    let Outcome::Updated { entity, exported } =
        fixture.update_track(&imported_entity, edited_track.clone(), true)?
    else {
        panic!("rejected");
    };
    assert!(exported);
    assert!(entity.hdr.rev > imported_entity.hdr.rev);
    // The track is synchronized with the file after re-importing it.
    assert_eq!(Some(entity.hdr.rev), entity.body.last_synchronized_rev);
    assert_ne!(
        imported_entity.body.track.media_source.content.link.rev,
        entity.body.track.media_source.content.link.rev,
    );
    // All edited metadata survived the roundtrip through the file.
    assert_eq!(Some("Edited title"), entity.body.track.track_title());
    assert_eq!(Some("Edited artist"), entity.body.track.track_artist());
    assert_eq!(edited_track.tags, entity.body.track.tags);
    Ok(())
}

#[test]
fn update_without_writing_file() -> anyhow::Result<()> {
    let mut fixture = Fixture::new()?;
    let imported_entity = fixture.import_track()?;
    let edited_track = edit_track(&imported_entity.body.track);

    let Outcome::Updated { entity, exported } =
        fixture.update_track(&imported_entity, edited_track.clone(), false)?
    else {
        panic!("rejected");
    };
    assert!(!exported);
    assert_eq!(edited_track.titles, entity.body.track.titles);
    assert_eq!(edited_track.actors, entity.body.track.actors);
    assert_eq!(edited_track.tags, entity.body.track.tags);
    // The track is no longer synchronized with the file.
    assert_ne!(Some(entity.hdr.rev), entity.body.last_synchronized_rev);
    Ok(())
}

#[test]
fn reject_update_if_entity_modified() -> anyhow::Result<()> {
    let mut fixture = Fixture::new()?;
    let imported_entity = fixture.import_track()?;
    let edited_track = edit_track(&imported_entity.body.track);
    let Outcome::Updated { entity, .. } =
        fixture.update_track(&imported_entity, edited_track.clone(), false)?
    else {
        panic!("rejected");
    };

    // Save again with the stale entity header.
    let outcome = fixture.update_track(&imported_entity, edited_track, true)?;
    assert!(matches!(
        outcome,
        Outcome::Rejected(Conflict::EntityRevision { expected, actual })
            if expected == imported_entity.hdr.rev && actual == entity.hdr.rev
    ));
    Ok(())
}

#[test]
fn reject_update_if_file_modified() -> anyhow::Result<()> {
    let mut fixture = Fixture::new()?;
    let imported_entity = fixture.import_track()?;
    let edited_track = edit_track(&imported_entity.body.track);

    touch_file(&fixture.file_path())?;

    for write_file in [false, true] {
        let outcome = fixture.update_track(&imported_entity, edited_track.clone(), write_file)?;
        assert!(matches!(
            outcome,
            Outcome::Rejected(Conflict::ContentRevision { .. })
        ));
    }
    Ok(())
}
