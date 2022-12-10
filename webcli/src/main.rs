// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::explicit_deref_methods)]
#![warn(clippy::explicit_into_iter_loop)]
#![warn(clippy::explicit_iter_loop)]
#![warn(clippy::must_use_candidate)]
#![warn(rustdoc::broken_intra_doc_links)]
#![cfg_attr(not(test), deny(clippy::panic_in_result_fn))]
#![cfg_attr(not(debug_assertions), deny(clippy::used_underscore_binding))]

use std::{
    env,
    ops::Not as _,
    sync::Arc,
    time::{Duration, Instant},
};

use clap::{Arg, ArgMatches, Command};
use model::{Effect, Task};
use tokio::signal;

use aoide_core::collection::{
    Collection, Entity as CollectionEntity, EntityUid as CollectionUid, MediaSourceConfig,
};

use aoide_core_api::{
    media::tracker::DirTrackingStatus,
    track::search::{SortField, SortOrder},
};

use aoide_client::{
    message::Message as ClientMessage,
    messaging::{message_channel, message_loop, send_message},
    models::{collection, media_source, media_tracker},
    state::{State as ClientState, StateUpdated as ClientStateUpdated},
};

mod model;
use crate::model::ExportTracksParams;

use self::model::{Environment, Intent, State};

const DEFAULT_LOG_FILTER: &str = "info";

const DEFAULT_WEBSRV_URL: &str = "http://[::1]:8080";

const WEBSRV_URL_ARG: &str = "websrv-url";

const CREATE_COLLECTION_TITLE_ARG: &str = "title";

const CREATE_COLLECTION_KIND_ARG: &str = "kind";

const CREATE_COLLECTION_VFS_ROOT_URL_ARG: &str = "vfs-root-url";

const ACTIVE_COLLECTION_TITLE_ARG: &str = "collection-title";

const MEDIA_ROOT_URL_ARG: &str = "media-root-url";

const OUTPUT_FILE_ARG: &str = "output-file";

const PROGRESS_POLLING_PERIOD: Duration = Duration::from_millis(1_000);

#[derive(Debug)]
struct CliState {
    matches: ArgMatches,
    state: State,
}

impl CliState {
    fn new(matches: ArgMatches) -> Self {
        Self {
            matches,
            state: Default::default(),
        }
    }
}

impl ClientState for CliState {
    type Intent = Intent;
    type Effect = Effect;
    type Task = Task;

    fn update(
        &mut self,
        message: ClientMessage<Self::Intent, Self::Effect>,
    ) -> ClientStateUpdated<Self::Effect, Self::Task> {
        match message {
            ClientMessage::Intent(intent) => intent.apply_on(&mut self.state),
            ClientMessage::Effect(effect) => effect.apply_on(&mut self.state),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(DEFAULT_LOG_FILTER))
        .init();

    let default_websrv_url =
        env::var("WEBSRV_URL").unwrap_or_else(|_| DEFAULT_WEBSRV_URL.to_owned());

    let active_collection_title_arg = Arg::new(ACTIVE_COLLECTION_TITLE_ARG)
        .long(ACTIVE_COLLECTION_TITLE_ARG)
        .num_args(1)
        .help("The `title` of the collection")
        .required(true);

    let mut app = Command::new("aoide-cli")
        .about("An experimental CLI for performing tasks on aoide")
        .version("0.0")
        .arg(
            Arg::new(WEBSRV_URL_ARG)
                .long(WEBSRV_URL_ARG)
                .num_args(1)
                .required(false)
                .default_value(DEFAULT_WEBSRV_URL)
        )
        .subcommand(
            Command::new("create-collection")
                .about("Creates a new collection")
                .arg(
                    Arg::new(CREATE_COLLECTION_TITLE_ARG)
                    .long(CREATE_COLLECTION_TITLE_ARG)
                    .help("The `title` of the new collection")
                    .num_args(1)
                    .required(true)                )
                .arg(
                    Arg::new(CREATE_COLLECTION_KIND_ARG)
                        .long(CREATE_COLLECTION_KIND_ARG)
                        .help("The `kind` of the new collection")
                        .num_args(1)
                        .required(false)
                )
                .arg(
                    Arg::new(CREATE_COLLECTION_VFS_ROOT_URL_ARG)
                        .long(CREATE_COLLECTION_VFS_ROOT_URL_ARG)
                        .help("The file URL of the common root directory that contains all media sources")
                        .num_args(1)
                        .required(true)
                ),
        )
        .subcommand({
            let media_root_url_arg = Arg::new(MEDIA_ROOT_URL_ARG)
                .help("The URL of the root directory with media source files")
                .num_args(1)
                .required(true);
            Command::new("media-sources")
                .about("Tasks for media sources")
                .subcommand(
                    Command::new("purge-orphaned")
                        .about("Purges orphaned media sources that are not referenced by any track")
                        .arg(
                            active_collection_title_arg.clone()
                        )
                        .arg(
                            media_root_url_arg.clone()
                        ),
                )
                .subcommand(
                    Command::new("purge-untracked")
                        .about("Purges untracked media sources including their tracks")
                        .arg(
                            active_collection_title_arg.clone()
                        )
                        .arg(
                            media_root_url_arg
                        ),
                )
        })
        .subcommand(
            Command::new("tracks")
                .about("Tasks for tracks")
                .subcommand(
                    Command::new("find-unsynchronized")
                        .about("Find all tracks with unsynchronized media sources")
                        .arg(
                            active_collection_title_arg
                                .clone()
                        )
                )
                .subcommand(
                    Command::new("export-all-into-file")
                        .about("Exports all tracks of the collection into a JSON file")
                        .arg(
                            active_collection_title_arg
                                .clone()
                        )
                        .arg(
                            Arg::new(OUTPUT_FILE_ARG)
                                .help("The output file path for writing JSON data")
                                .num_args(1)
                                .required(true)
                        ),
                )
        )
        .subcommand({
            let media_root_url_arg = Arg::new(MEDIA_ROOT_URL_ARG)
                .help("The URL of the root directory containing tracked media files")
                .num_args(1)
                .required(false);
            Command::new("media-tracker")
                .about("Tasks for the media tracker")
                .subcommand(
                    Command::new("progress").about("Query progress of a pending task"),
                )
                .subcommand(Command::new("abort").about("Abort the current task"))
                .subcommand(
                    Command::new("status")
                        .about("Queries the status of the media tracker")
                        .arg(
                            active_collection_title_arg.clone()
                        )
                        .arg(
                            media_root_url_arg.clone()
                        ),
                )
                .subcommand(
                    Command::new("scan-directories")
                        .about("Scans directories on the file system for added/modified/removed media sources")
                        .arg(
                            active_collection_title_arg.clone()
                        )
                        .arg(
                            media_root_url_arg.clone()
                        ),
                )
                .subcommand(
                    Command::new("untrack-directories")
                        .about("Untracks directories on the file system")
                        .arg(
                            active_collection_title_arg.clone()
                        )
                        .arg(
                            media_root_url_arg.clone()
                        ),
                )
                .subcommand(
                    Command::new("untrack-orphaned-directories")
                        .about("Untracks orphaned directories that have disappeared from the file system (deleted)")
                        .arg(
                            active_collection_title_arg.clone()
                        )
                        .arg(
                            media_root_url_arg.clone()
                        ),
                )
                .subcommand(
                    Command::new("import-files")
                        .about("Imports media sources on the file system from scanned directories")
                        .arg(
                            active_collection_title_arg.clone()
                        )
                        .arg(
                            media_root_url_arg.clone()
                        ),
                )
                .subcommand(
                    Command::new("find-untracked-files")
                        .about("Scans directories on the file system for untracked entries")
                        .arg(
                            active_collection_title_arg.clone()
                        )
                        .arg(
                            media_root_url_arg
                        ),
                )
        });
    let app_usage = app.render_usage();
    let matches = app.get_matches();

    let websrv_url = matches
        .get_one(WEBSRV_URL_ARG)
        .unwrap_or(&default_websrv_url)
        .parse()
        .expect(WEBSRV_URL_ARG);

    let shared_env = Arc::new(Environment::new(websrv_url));
    let (message_tx, message_rx) = message_channel();

    let mut collection_uid: Option<CollectionUid> = None;
    let mut subcommand_submitted = false;

    let mut last_media_sources_purge_orphaned_outcome = None;
    let mut last_media_sources_purge_untracked_outcome = None;
    let mut last_media_tracker_progress_fetched = None;
    let mut last_media_tracker_progress = None;
    let mut last_media_tracker_status = None;
    let mut last_media_tracker_scan_directories_outcome = None;
    let mut last_media_tracker_untrack_directories_outcome = None;
    let mut last_media_tracker_import_files_outcome = None;
    let mut last_media_tracker_find_untracked_files_outcome = None;

    let message_loop = tokio::spawn(message_loop(
        shared_env,
        (message_tx.clone(), message_rx),
        CliState::new(matches),
        Box::new(move |cli_state| {
            let CliState { matches, state } = cli_state;

            if !state.last_errors().is_empty() {
                for err in state.last_errors() {
                    log::error!("{err}");
                }
                // Terminate after errors occurred
                return Some(Intent::Terminate);
            }

            if last_media_sources_purge_orphaned_outcome.as_ref()
                != state
                    .media_sources
                    .remote_view()
                    .last_purge_orphaned_outcome
                    .last_snapshot()
            {
                last_media_sources_purge_orphaned_outcome = state
                    .media_sources
                    .remote_view()
                    .last_purge_orphaned_outcome
                    .last_snapshot()
                    .map(ToOwned::to_owned);
                if let Some(outcome) = &last_media_sources_purge_orphaned_outcome {
                    log::info!("Purging orphaned media sources succeeded: {outcome:?}");
                }
            }
            if last_media_sources_purge_untracked_outcome.as_ref()
                != state
                    .media_sources
                    .remote_view()
                    .last_purge_untracked_outcome
                    .last_snapshot()
            {
                last_media_sources_purge_untracked_outcome = state
                    .media_sources
                    .remote_view()
                    .last_purge_untracked_outcome
                    .last_snapshot()
                    .map(ToOwned::to_owned);
                if let Some(outcome) = &last_media_sources_purge_untracked_outcome {
                    log::info!("Purging untracked media sources succeeded: {outcome:?}");
                }
            }
            if last_media_tracker_progress.as_ref()
                != state.media_tracker.remote_view().progress.last_snapshot()
            {
                last_media_tracker_progress = state
                    .media_tracker
                    .remote_view()
                    .progress
                    .last_snapshot()
                    .map(ToOwned::to_owned);
                if let Some(progress) = &last_media_tracker_progress {
                    log::info!("Media tracker progress: {progress:?}");
                }
            }
            if last_media_tracker_status.as_ref()
                != state.media_tracker.remote_view().status.last_snapshot()
            {
                last_media_tracker_status = state
                    .media_tracker
                    .remote_view()
                    .status
                    .last_snapshot()
                    .map(ToOwned::to_owned);
                if let Some(status) = &last_media_tracker_status {
                    log::info!("Media tracker status: {status:?}");
                }
            }
            if last_media_tracker_scan_directories_outcome.as_ref()
                != state
                    .media_tracker
                    .remote_view()
                    .last_scan_directories_outcome
                    .last_snapshot()
            {
                last_media_tracker_scan_directories_outcome = state
                    .media_tracker
                    .remote_view()
                    .last_scan_directories_outcome
                    .last_snapshot()
                    .map(ToOwned::to_owned);
                if let Some(outcome) = &last_media_tracker_scan_directories_outcome {
                    log::info!("Scanning media directories succeeded: {outcome:?}");
                }
            }
            if last_media_tracker_untrack_directories_outcome.as_ref()
                != state
                    .media_tracker
                    .remote_view()
                    .last_untrack_directories_outcome
                    .last_snapshot()
            {
                last_media_tracker_untrack_directories_outcome = state
                    .media_tracker
                    .remote_view()
                    .last_untrack_directories_outcome
                    .last_snapshot()
                    .map(ToOwned::to_owned);
                if let Some(outcome) = &last_media_tracker_untrack_directories_outcome {
                    log::info!("Untracking media directories succeeded: {outcome:?}");
                }
            }
            if last_media_tracker_import_files_outcome.as_ref()
                != state
                    .media_tracker
                    .remote_view()
                    .last_import_files_outcome
                    .last_snapshot()
            {
                last_media_tracker_import_files_outcome = state
                    .media_tracker
                    .remote_view()
                    .last_import_files_outcome
                    .last_snapshot()
                    .map(ToOwned::to_owned);
                if let Some(outcome) = &last_media_tracker_import_files_outcome {
                    log::info!(
                        "Importing media files from '{}' ({}) succeeded with {:?}: {:?}",
                        outcome.value.root_path,
                        outcome.value.root_url,
                        outcome.value.completion,
                        outcome.value.summary
                    );
                    for imported_source_with_issues in &outcome.value.imported_sources_with_issues {
                        log::warn!(
                            "{}: {}",
                            imported_source_with_issues.path,
                            imported_source_with_issues.messages.join(" | ")
                        );
                    }
                }
            }
            if last_media_tracker_find_untracked_files_outcome.as_ref()
                != state
                    .media_tracker
                    .remote_view()
                    .last_find_untracked_files_outcome
                    .last_snapshot()
            {
                last_media_tracker_find_untracked_files_outcome = state
                    .media_tracker
                    .remote_view()
                    .last_find_untracked_files_outcome
                    .last_snapshot()
                    .map(ToOwned::to_owned);
                if let Some(outcome) = &last_media_tracker_find_untracked_files_outcome {
                    log::info!("Finding untracked media files succeeded: {outcome:?}");
                    if !outcome.value.content_paths.is_empty() {
                        log::info!(
                            "Found {num_untracked_entities} untracked entries on file system:\n{content_paths}",
                            num_untracked_entities = outcome.value.content_paths.len(),
                            content_paths = outcome
                                .value
                                .content_paths
                                .iter()
                                .map(|content_path| content_path.as_str())
                                .collect::<Vec<_>>()
                                .join("\n"),
                        );
                    }
                }
            }

            // Only submit a single subcommand
            if subcommand_submitted {
                let next_intent = if !state.is_terminating() && !state.is_pending() {
                    // Terminate when idle and no task is pending
                    Some(Intent::Terminate)
                } else {
                    // Periodically refetch and report progress while busy
                    if state.media_tracker.remote_view().is_pending() {
                        if let Some(last_fetched) = last_media_tracker_progress_fetched {
                            let now = Instant::now();
                            if now >= last_fetched {
                                let not_before = now + PROGRESS_POLLING_PERIOD;
                                last_media_tracker_progress_fetched = Some(not_before);
                                let intent = Intent::Deferred {
                                    not_before,
                                    intent: Box::new(media_tracker::Intent::FetchProgress.into()),
                                };
                                Some(intent)
                            } else {
                                None
                            }
                        } else {
                            last_media_tracker_progress_fetched = Some(Instant::now());
                            Some(media_tracker::Intent::FetchProgress.into())
                        }
                    } else {
                        None
                    }
                };
                return next_intent;
            }

            // Commands that don't require an active collection
            if let Some(("media-tracker", matches)) = matches.subcommand() {
                if matches!(matches.subcommand(), Some(("progress", _))) {
                    subcommand_submitted = true;
                    last_media_tracker_progress_fetched = Some(Instant::now());
                    let intent = media_tracker::Intent::FetchProgress;
                    return Some(intent.into());
                }
                if matches!(matches.subcommand(), Some(("abort", _))) {
                    subcommand_submitted = true;
                    let intent = Intent::AbortPendingRequest;
                    return Some(intent);
                }
            }

            if subcommand_submitted {
                return None;
            }

            match matches.subcommand() {
                Some(("create-collection", matches)) => {
                    let title = matches
                        .get_one::<String>(CREATE_COLLECTION_TITLE_ARG)
                        .expect(CREATE_COLLECTION_TITLE_ARG);
                    let kind = matches
                        .get_one::<String>(CREATE_COLLECTION_KIND_ARG)
                        .filter(|kind| kind.trim().is_empty().not());
                    let vfs_root_url = matches
                        .get_one::<String>(CREATE_COLLECTION_VFS_ROOT_URL_ARG)
                        .map(|s| s.parse().expect(CREATE_COLLECTION_VFS_ROOT_URL_ARG))
                        .expect(CREATE_COLLECTION_VFS_ROOT_URL_ARG);
                    let new_collection = Collection {
                        title: title.to_owned(),
                        kind: kind.map(ToOwned::to_owned),
                        notes: None,
                        color: None,
                        media_source_config: MediaSourceConfig {
                            content_path:
                                aoide_core::media::content::ContentPathConfig::VirtualFilePath {
                                    root_url: vfs_root_url,
                                },
                        },
                    };
                    subcommand_submitted = true;
                    let intent = collection::Intent::CreateEntity { new_collection };
                    return Some(intent.into());
                }
                Some(("media-sources", matches)) => match matches.subcommand() {
                    Some(("purge-orphaned", matches)) => {
                        require_active_collection(matches, state, &mut collection_uid).map(
                            |entity| {
                                let collection_uid = entity.hdr.uid.clone();
                                let media_root_url = matches
                                    .get_one::<String>(MEDIA_ROOT_URL_ARG)
                                    .map(|s| s.parse().expect("URL"));
                                let params =
                                    aoide_core_api::media::source::purge_orphaned::Params {
                                        root_url: media_root_url,
                                    };
                                subcommand_submitted = true;
                                let intent = media_source::Intent::PurgeOrphaned {
                                    collection_uid,
                                    params,
                                };
                                Some(intent.into())
                            },
                        )
                    }
                    Some(("purge-untracked", matches)) => {
                        require_active_collection(matches, state, &mut collection_uid).map(
                            |entity| {
                                let collection_uid = entity.hdr.uid.clone();
                                let media_root_url = matches
                                    .get_one::<String>(MEDIA_ROOT_URL_ARG)
                                    .map(|s| s.parse().expect("URL"));
                                let params =
                                    aoide_core_api::media::source::purge_untracked::Params {
                                        root_url: media_root_url,
                                    };
                                subcommand_submitted = true;
                                let intent = media_source::Intent::PurgeUntracked {
                                    collection_uid,
                                    params,
                                };
                                Some(intent.into())
                            },
                        )
                    }
                    Some((subcommand, _)) => {
                        unreachable!("Unknown subcommand {subcommand}");
                    }
                    None => Err(None),
                },
                Some(("media-tracker", matches)) => match matches.subcommand() {
                    Some(("query-status", matches)) => {
                        require_active_collection(matches, state, &mut collection_uid).map(
                            |entity| {
                                let collection_uid = entity.hdr.uid.clone();
                                let media_root_url = matches
                                    .get_one::<String>(MEDIA_ROOT_URL_ARG)
                                    .map(|s| s.parse().expect("URL"))
                                    .or_else(|| {
                                        entity
                                            .body
                                            .media_source_config
                                            .content_path
                                            .root_url()
                                            .cloned()
                                            .map(Into::into)
                                    });
                                last_media_tracker_status = None;
                                let params = aoide_core_api::media::tracker::query_status::Params {
                                    root_url: media_root_url,
                                };
                                subcommand_submitted = true;
                                let intent = media_tracker::Intent::FetchStatus {
                                    collection_uid,
                                    params,
                                };
                                Some(intent.into())
                            },
                        )
                    }
                    Some(("scan-directories", matches)) => {
                        require_active_collection(matches, state, &mut collection_uid).map(
                            |entity| {
                                let collection_uid = entity.hdr.uid.clone();
                                let media_root_url = matches
                                    .get_one::<String>(MEDIA_ROOT_URL_ARG)
                                    .map(|s| s.parse().expect("URL"))
                                    .or_else(|| {
                                        entity
                                            .body
                                            .media_source_config
                                            .content_path
                                            .root_url()
                                            .cloned()
                                            .map(Into::into)
                                    });
                                let params =
                                    aoide_core_api::media::tracker::scan_directories::Params {
                                        root_url: media_root_url,
                                        ..Default::default()
                                    };
                                subcommand_submitted = true;
                                let intent = media_tracker::Intent::StartScanDirectories {
                                    collection_uid,
                                    params,
                                };
                                Some(intent.into())
                            },
                        )
                    }
                    Some(("untrack-directories", matches)) => {
                        require_active_collection(matches, state, &mut collection_uid).map(
                            |entity| {
                                let collection_uid = entity.hdr.uid.clone();
                                let media_root_url = matches
                                    .get_one::<String>(MEDIA_ROOT_URL_ARG)
                                    .map(|s| s.parse().expect("URL"))
                                    .expect("required");
                                let params =
                                    aoide_core_api::media::tracker::untrack_directories::Params {
                                        root_url: Some(media_root_url),
                                        status: None,
                                    };
                                subcommand_submitted = true;
                                let intent = media_tracker::Intent::UntrackDirectories {
                                    collection_uid,
                                    params,
                                };
                                Some(intent.into())
                            },
                        )
                    }
                    Some(("untrack-orphaned-directories", matches)) => {
                        require_active_collection(matches, state, &mut collection_uid).map(
                            |entity| {
                                let collection_uid = entity.hdr.uid.clone();
                                let media_root_url = matches
                                    .get_one::<String>(MEDIA_ROOT_URL_ARG)
                                    .map(|s| s.parse().expect("URL"));
                                let params =
                                    aoide_core_api::media::tracker::untrack_directories::Params {
                                        root_url: media_root_url,
                                        status: Some(DirTrackingStatus::Orphaned),
                                    };
                                subcommand_submitted = true;
                                let intent = media_tracker::Intent::UntrackDirectories {
                                    collection_uid,
                                    params,
                                };
                                Some(intent.into())
                            },
                        )
                    }
                    Some(("import-files", matches)) => {
                        require_active_collection(matches, state, &mut collection_uid).map(
                            |entity| {
                                let collection_uid = entity.hdr.uid.clone();
                                let media_root_url = matches
                                    .get_one::<String>(MEDIA_ROOT_URL_ARG)
                                    .map(|s| s.parse().expect("URL"));
                                let params = aoide_core_api::media::tracker::import_files::Params {
                                    root_url: media_root_url,
                                    ..Default::default()
                                };
                                subcommand_submitted = true;
                                let intent = media_tracker::Intent::StartImportFiles {
                                    collection_uid,
                                    params,
                                };
                                Some(intent.into())
                            },
                        )
                    }
                    Some(("find-untracked-files", matches)) => {
                        require_active_collection(matches, state, &mut collection_uid).map(
                            |entity| {
                                let collection_uid = entity.hdr.uid.clone();
                                let media_root_url = matches
                                    .get_one::<String>(MEDIA_ROOT_URL_ARG)
                                    .map(|s| s.parse().expect("URL"))
                                    .or_else(|| {
                                        entity
                                            .body
                                            .media_source_config
                                            .content_path
                                            .root_url()
                                            .cloned()
                                            .map(Into::into)
                                    });
                                let params =
                                    aoide_core_api::media::tracker::find_untracked_files::Params {
                                        root_url: media_root_url,
                                        ..Default::default()
                                    };
                                subcommand_submitted = true;
                                let intent = media_tracker::Intent::StartFindUntrackedFiles {
                                    collection_uid,
                                    params,
                                };
                                Some(intent.into())
                            },
                        )
                    }
                    Some((subcommand, _)) => {
                        unreachable!("Unknown subcommand {subcommand}");
                    }
                    None => {
                        println!("{app_usage}");
                        Err(None)
                    }
                },
                Some(("tracks", matches)) => match matches.subcommand() {
                    Some(("find-unsynchronized", matches)) => {
                        require_active_collection(matches, state, &mut collection_uid).map(
                            |entity| {
                                let collection_uid = entity.hdr.uid.clone();
                                let params = aoide_core_api::track::find_unsynchronized::Params {
                                    resolve_url_from_content_path: Some(Default::default()),
                                    content_path_predicate: None,
                                };
                                subcommand_submitted = true;
                                let intent = Intent::FindUnsynchronizedTracks {
                                    collection_uid,
                                    params,
                                };
                                Some(intent)
                            },
                        )
                    }
                    Some(("export-all-into-file", matches)) => {
                        require_active_collection(matches, state, &mut collection_uid).map(
                            |entity| {
                                let collection_uid = entity.hdr.uid.clone();
                                let output_file_path = matches
                                    .get_one::<String>(OUTPUT_FILE_ARG)
                                    .expect(OUTPUT_FILE_ARG)
                                    .to_owned();
                                let params = ExportTracksParams {
                                    output_file_path: output_file_path.into(),
                                    track_search: aoide_core_api::track::search::Params {
                                        filter: None,
                                        ordering: vec![SortOrder {
                                            field: SortField::UpdatedAt,
                                            direction:
                                                aoide_core_api::sorting::SortDirection::Descending,
                                        }],
                                        // TODO: Configurable?
                                        resolve_url_from_content_path: Some(Default::default()),
                                    },
                                };
                                subcommand_submitted = true;
                                let intent = Intent::ExportTracks {
                                    collection_uid,
                                    params,
                                };
                                Some(intent)
                            },
                        )
                    }
                    Some((subcommand, _)) => {
                        unreachable!("Unknown subcommand {subcommand}");
                    }
                    None => Err(None),
                },
                Some((subcommand, _)) => {
                    unreachable!("Unknown subcommand {subcommand}");
                }
                None => Err(None),
            }
            .unwrap_or_else(|intent| {
                // No command submitted
                if intent.is_none() && !state.is_pending() {
                    println!("{app_usage}");
                }
                intent
            })
        }),
    ));

    // Handle Ctrl-C/SIGINT signals to abort processing
    tokio::spawn({
        let message_tx = message_tx.clone();
        async move {
            if let Err(err) = signal::ctrl_c().await {
                log::error!("Failed to receive Ctrl-C/SIGINT signal: {err}");
            }
            log::info!("Terminating after receiving Ctrl-C/SIGINT...");
            send_message(&message_tx, Intent::Terminate);
        }
    });

    // Kick off the loop by sending a first message
    // before awaiting its termination
    send_message(&message_tx, Intent::RenderState);
    message_loop.await?;

    Ok(())
}

fn require_active_collection<'s>(
    matches: &ArgMatches,
    state: &'s State,
    collection_uid: &mut Option<CollectionUid>,
) -> Result<&'s CollectionEntity, Option<Intent>> {
    if let Some(entity) = state.active_collection.active_entity() {
        debug_assert!(!state.is_pending());
        log::info!(
            "Active collection: '{}' ({})",
            entity.body.title,
            entity.hdr.uid
        );
        return Ok(entity);
    }
    let collection_title =
        if let Some(collection_title) = matches.get_one::<String>(ACTIVE_COLLECTION_TITLE_ARG) {
            collection_title
        } else {
            return Err(None);
        };
    if let Some(filtered_entities) = state
        .active_collection
        .remote_view()
        .filtered_entities
        .last_snapshot()
    {
        // Activate an existing collection
        if state.active_collection.active_entity_uid().is_none() {
            if filtered_entities.value.is_empty() {
                log::warn!("No collections available");
            } else if let Some(entity) = state
                .active_collection
                .remote_view()
                .find_entity_by_title(collection_title)
            {
                log::info!(
                    "Activating collection '{}' ({})",
                    entity.body.title,
                    entity.hdr.uid,
                );
                let entity_uid = Some(entity.hdr.uid.to_owned());
                *collection_uid = entity_uid.clone();
                let intent = collection::Intent::ActivateEntity { entity_uid };
                return Err(Some(intent.into()));
            } else {
                log::warn!("No collection with title '{collection_title}' found");
            }
        }
    } else if !state
        .active_collection
        .remote_view()
        .filtered_entities
        .is_pending()
    {
        let filter_by_kind = None;
        let intent = collection::Intent::FetchFilteredEntities { filter_by_kind };
        return Err(Some(intent.into()));
    }
    Err(None)
}
