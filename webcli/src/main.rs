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

#![warn(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))]
#![deny(rust_2018_idioms)]
#![deny(rust_2021_compatibility)]
#![deny(missing_debug_implementations)]
#![deny(clippy::all)]
#![deny(clippy::explicit_deref_methods)]
#![deny(clippy::explicit_into_iter_loop)]
#![deny(clippy::explicit_iter_loop)]
#![deny(clippy::must_use_candidate)]
#![cfg_attr(not(test), deny(clippy::panic_in_result_fn))]
#![cfg_attr(not(debug_assertions), deny(clippy::used_underscore_binding))]

use std::{
    env,
    sync::Arc,
    time::{Duration, Instant},
};

use clap::{App, Arg, ArgMatches};
use model::{Effect, Task};
use tokio::signal;

use aoide_core::{
    collection::{Collection, MediaSourceConfig},
    entity::EntityUid,
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

const DEFAULT_SERVICE_URL: &str = "http://[::1]:8080";

const PROGRESS_POLLING_PERIOD: Duration = Duration::from_millis(1_000);

const COLLECTION_VFS_ROOT_URL_PARAM: &str = "vfs-root-url";

const MEDIA_ROOT_URL_PARAM: &str = "media-root-url";

const OUTPUT_FILE_PARAM: &str = "output-file";

#[derive(Debug)]
struct CliState {
    matches: ArgMatches,
    state: State,
}

impl CliState {
    pub fn new(matches: ArgMatches) -> Self {
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

    let default_service_url =
        env::var("API_URL").unwrap_or_else(|_| DEFAULT_SERVICE_URL.to_owned());

    let mut app = App::new("aoide-cli")
        .about("An experimental CLI for performing tasks on aoide")
        .version("0.0")
        .arg(
            Arg::new("collection-uid")
                .short('c')
                .long("collection-uid")
                .takes_value(true)
                .required(false)
        )
        .arg(
            Arg::new("service-url")
                .short('s')
                .long("service-url")
                .takes_value(true)
                .required(false)
                .default_value(DEFAULT_SERVICE_URL)
        )
        .subcommand(
            App::new("collections")
                .about("Tasks for managing collections")
                .subcommand(
                    App::new("create-mixxx")
                        .about("Creates a new mixxx.org collection for Mixxx")
                        .arg(
                            Arg::new("title")
                                .help("The title of the new collection")
                                .takes_value(true)
                                .required(true)
                        )
                        .arg(
                            Arg::new(COLLECTION_VFS_ROOT_URL_PARAM)
                                .help("The file URL of the common root directory that contains all media sources")
                                .takes_value(true)
                                .required(true)
                        ),
                )
        )
        .subcommand(
            App::new("media-sources")
                .about("Tasks for media sources")
                .subcommand(
                    App::new("purge-orphaned")
                        .about("Purges orphaned media sources that are not referenced by any track")
                        .arg(
                            Arg::new(MEDIA_ROOT_URL_PARAM)
                                .help("The common root URL or directory that should be considered")
                                .takes_value(true)
                                .required(false)
                        ),
                )
                .subcommand(
                    App::new("purge-untracked")
                        .about("Purges untracked media sources including their tracks")
                        .arg(
                            Arg::new(MEDIA_ROOT_URL_PARAM)
                                .help("The URL of the root directory containing tracked media files to be purged")
                                .takes_value(true)
                                .required(false)
                        ),
                )
        )
        .subcommand(
            App::new("tracks")
                .about("Tasks for tracks")
                .subcommand(
                    App::new("export-all-into-file")
                        .about("Exports all tracks of the collection into a JSON file")
                        .arg(
                            Arg::new(OUTPUT_FILE_PARAM)
                                .short('o')
                                .help("The output file path for writing JSON data")
                                .takes_value(true)
                                .required(true)
                        ),
                )
        )
        .subcommand(
            App::new("media-tracker")
                .about("Tasks for the media tracker")
                .subcommand(
                    App::new("progress").about("Query progress of a pending task"),
                )
                .subcommand(App::new("abort").about("Abort the current task"))
                .subcommand(
                    App::new("status")
                        .about("Queries the status of the media tracker")
                        .arg(
                            Arg::new(MEDIA_ROOT_URL_PARAM)
                                .help("The URL of the root directory containing tracked media files to be queried")
                                .takes_value(true)
                                .required(false)
                        ),
                )
                .subcommand(
                    App::new("scan-directories")
                        .about("Scans directories on the file system for added/modified/removed media sources")
                        .arg(
                            Arg::new(MEDIA_ROOT_URL_PARAM)
                                .help("The URL of the root directory containing tracked media files to be scanned")
                                .takes_value(true)
                                .required(false)
                        ),
                )
                .subcommand(
                    App::new("untrack-directories")
                        .about("Untracks directories on the file system")
                        .arg(
                            Arg::new(MEDIA_ROOT_URL_PARAM)
                                .help("The URL of the root directory containing tracked media files to be untracked")
                                .takes_value(true)
                                .required(true)
                        ),
                )
                .subcommand(
                    App::new("untrack-orphaned-directories")
                        .about("Untracks orphaned directories that have disappeared from the file system (deleted)")
                        .arg(
                            Arg::new(MEDIA_ROOT_URL_PARAM)
                                .help("The URL of the root directory containing tracked media files to be untracked")
                                .takes_value(true)
                                .required(false)
                        ),
                )
                .subcommand(
                    App::new("import-files")
                        .about("Imports media sources on the file system from scanned directories")
                        .arg(
                            Arg::new(MEDIA_ROOT_URL_PARAM)
                                .help("The URL of the root directory containing tracked media files to be imported")
                                .takes_value(true)
                                .required(false)
                        ),
                )
                .subcommand(
                    App::new("find-untracked-files")
                        .about("Scans directories on the file system for untracked entries")
                        .arg(
                            Arg::new(MEDIA_ROOT_URL_PARAM)
                                .help("The URL of the root directory containing tracked media files to be scanned")
                                .takes_value(true)
                                .required(false)
                        ),
                ),
        );
    let app_usage = app.render_usage();
    let matches = app.get_matches();

    let service_url = matches
        .value_of("service-url")
        .unwrap_or(&default_service_url)
        .parse()
        .expect("URL");
    let mut collection_uid = matches
        .value_of("collection-uid")
        .map(|s| s.parse::<EntityUid>().expect("Collection UID"));

    let shared_env = Arc::new(Environment::new(service_url));
    let (message_tx, message_rx) = message_channel();

    let mut last_media_sources_purge_orphaned_outcome = None;
    let mut last_media_sources_purge_untracked_outcome = None;
    let mut last_media_tracker_progress_fetched = None;
    let mut last_media_tracker_progress = None;
    let mut last_media_tracker_status = None;
    let mut last_media_tracker_scan_directories_outcome = None;
    let mut last_media_tracker_untrack_directories_outcome = None;
    let mut last_media_tracker_import_files_outcome = None;
    let mut last_media_tracker_find_untracked_files_outcome = None;
    let mut subcommand_submitted = false;
    let message_loop = tokio::spawn(message_loop(
        shared_env,
        (message_tx.clone(), message_rx),
        CliState::new(matches),
        Box::new(move |cli_state| {
            let CliState { matches, state } = cli_state;
            if !state.last_errors().is_empty() {
                for err in state.last_errors() {
                    log::error!("{}", err);
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
                    log::info!("Purging orphaned media sources succeeded: {:?}", outcome);
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
                    log::info!("Purging untracked media sources succeeded: {:?}", outcome);
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
                    log::info!("Media tracker progress: {:?}", progress);
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
                    log::info!("Media tracker status: {:?}", status);
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
                    log::info!("Scanning media directories succeeded: {:?}", outcome);
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
                    log::info!("Untracking media directories succeeded: {:?}", outcome);
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
                    log::info!("Finding untracked media files succeeded: {:?}", outcome);
                    if !outcome.value.source_paths.is_empty() {
                        log::info!(
                            "Found {} untracked entries on file system:\n{}",
                            outcome.value.source_paths.len(),
                            outcome
                                .value
                                .source_paths
                                .iter()
                                .map(|source_path| source_path.as_str())
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
            if let Some(("collections", collections_matches)) = matches.subcommand() {
                match collections_matches.subcommand() {
                    Some(("create-mixxx", create_matches)) => {
                        let title = create_matches.value_of("title").expect("title");
                        let vfs_root_url = create_matches
                            .value_of(COLLECTION_VFS_ROOT_URL_PARAM)
                            .map(|s| s.parse().expect(COLLECTION_VFS_ROOT_URL_PARAM))
                            .expect(COLLECTION_VFS_ROOT_URL_PARAM);
                        let new_collection = Collection {
                            title: title.to_owned(),
                            kind: Some("mixxx.org".to_owned()),
                            notes: None,
                            color: None,
                            media_source_config: MediaSourceConfig {
                                source_path: aoide_core::media::SourcePathConfig::VirtualFilePath {
                                    root_url: vfs_root_url,
                                },
                            },
                        };
                        subcommand_submitted = true;
                        let intent = collection::Intent::CreateEntity { new_collection };
                        return Some(intent.into());
                    }
                    Some((_subcommand, _)) => {
                        unreachable!("Unknown subcommand {}", _subcommand);
                    }
                    None => {
                        println!("{}", app_usage);
                    }
                }
            }
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

            // Select an active collection
            if let Some(filtered_entities) = state
                .active_collection
                .remote_view()
                .filtered_entities
                .last_snapshot()
            {
                if state.active_collection.active_entity_uid().is_none() {
                    if filtered_entities.value.is_empty() {
                        log::warn!("No collections available");
                        return None;
                    }
                    if collection_uid.is_none() && filtered_entities.value.len() == 1 {
                        collection_uid = filtered_entities.value.get(0).map(|e| e.hdr.uid.clone());
                        debug_assert!(collection_uid.is_some());
                        log::info!(
                            "Activating single collection: {}",
                            collection_uid.as_ref().unwrap()
                        );
                    }
                    if let Some(collection_uid) = &collection_uid {
                        if state
                            .active_collection
                            .remote_view()
                            .find_entity_by_uid(collection_uid)
                            .is_some()
                        {
                            let entity_uid = Some(collection_uid.to_owned());
                            let intent = collection::Intent::ActivateEntity { entity_uid };
                            return Some(intent.into());
                        } else {
                            log::warn!("Collection not available: {}", collection_uid);
                        }
                    }
                    println!("Filtered collections:");
                    for available_collection in &filtered_entities.value {
                        println!(
                            "{}: {} | {}",
                            available_collection.hdr.uid,
                            available_collection.body.title,
                            available_collection.body.notes.as_deref().unwrap_or(""),
                        );
                    }
                    return None;
                }
            } else if !state
                .active_collection
                .remote_view()
                .filtered_entities
                .is_pending()
            {
                // TODO: Provide kind as optional command line argument
                let filter_by_kind = None;
                let intent = collection::Intent::FetchFilteredEntities { filter_by_kind };
                return Some(intent.into());
            }

            if subcommand_submitted {
                return None;
            }

            // Commands that require an active collection
            if let Some(entity) = state.active_collection.active_entity() {
                log::info!("Active collection: {}", entity.hdr.uid);
                if state.is_pending() {
                    last_media_tracker_progress_fetched = Some(Instant::now());
                    return Some(media_tracker::Intent::FetchProgress.into());
                }
                match matches.subcommand() {
                    Some(("media-sources", matches)) => match matches.subcommand() {
                        Some(("purge-orphaned", matches)) => {
                            let collection_uid = entity.hdr.uid.clone();
                            let media_root_url = matches
                                .value_of(MEDIA_ROOT_URL_PARAM)
                                .map(|s| s.parse().expect("URL"));
                            subcommand_submitted = true;
                            let params = aoide_core_api::media::source::purge_orphaned::Params {
                                root_url: media_root_url,
                            };
                            let intent = media_source::Intent::PurgeOrphaned {
                                collection_uid,
                                params,
                            };
                            return Some(intent.into());
                        }
                        Some(("purge-untracked", matches)) => {
                            let collection_uid = entity.hdr.uid.clone();
                            let media_root_url = matches
                                .value_of(MEDIA_ROOT_URL_PARAM)
                                .map(|s| s.parse().expect("URL"));
                            subcommand_submitted = true;
                            let params = aoide_core_api::media::source::purge_untracked::Params {
                                root_url: media_root_url,
                            };
                            let intent = media_source::Intent::PurgeUntracked {
                                collection_uid,
                                params,
                            };
                            return Some(intent.into());
                        }
                        Some((_subcommand, _)) => {
                            unreachable!("Unknown subcommand {}", _subcommand);
                        }
                        None => {
                            println!("{}", app_usage);
                        }
                    },
                    Some(("media-tracker", matches)) => match matches.subcommand() {
                        Some(("query-status", matches)) => {
                            let collection_uid = entity.hdr.uid.clone();
                            let media_root_url = matches
                                .value_of(MEDIA_ROOT_URL_PARAM)
                                .map(|s| s.parse().expect("URL"))
                                .or_else(|| {
                                    entity
                                        .body
                                        .media_source_config
                                        .source_path
                                        .root_url()
                                        .cloned()
                                        .map(Into::into)
                                });
                            subcommand_submitted = true;
                            last_media_tracker_status = None;
                            let params = aoide_core_api::media::tracker::query_status::Params {
                                root_url: media_root_url,
                            };
                            let intent = media_tracker::Intent::FetchStatus {
                                collection_uid,
                                params,
                            };
                            return Some(intent.into());
                        }
                        Some(("scan-directories", matches)) => {
                            let collection_uid = entity.hdr.uid.clone();
                            let media_root_url = matches
                                .value_of(MEDIA_ROOT_URL_PARAM)
                                .map(|s| s.parse().expect("URL"))
                                .or_else(|| {
                                    entity
                                        .body
                                        .media_source_config
                                        .source_path
                                        .root_url()
                                        .cloned()
                                        .map(Into::into)
                                });
                            subcommand_submitted = true;
                            let params = aoide_core_api::media::tracker::scan_directories::Params {
                                root_url: media_root_url,
                                ..Default::default()
                            };
                            let intent = media_tracker::Intent::StartScanDirectories {
                                collection_uid,
                                params,
                            };
                            return Some(intent.into());
                        }
                        Some(("untrack-directories", matches)) => {
                            let collection_uid = entity.hdr.uid.clone();
                            let media_root_url = matches
                                .value_of(MEDIA_ROOT_URL_PARAM)
                                .map(|s| s.parse().expect("URL"))
                                .expect("required");
                            subcommand_submitted = true;
                            let params =
                                aoide_core_api::media::tracker::untrack_directories::Params {
                                    root_url: Some(media_root_url),
                                    status: None,
                                };
                            let intent = media_tracker::Intent::UntrackDirectories {
                                collection_uid,
                                params,
                            };
                            return Some(intent.into());
                        }
                        Some(("untrack-orphaned-directories", matches)) => {
                            let collection_uid = entity.hdr.uid.clone();
                            let media_root_url = matches
                                .value_of(MEDIA_ROOT_URL_PARAM)
                                .map(|s| s.parse().expect("URL"));
                            subcommand_submitted = true;
                            let params =
                                aoide_core_api::media::tracker::untrack_directories::Params {
                                    root_url: media_root_url,
                                    status: Some(DirTrackingStatus::Orphaned),
                                };
                            let intent = media_tracker::Intent::UntrackDirectories {
                                collection_uid,
                                params,
                            };
                            return Some(intent.into());
                        }
                        Some(("import-files", matches)) => {
                            let collection_uid = entity.hdr.uid.clone();
                            let media_root_url = matches
                                .value_of(MEDIA_ROOT_URL_PARAM)
                                .map(|s| s.parse().expect("URL"));
                            subcommand_submitted = true;
                            let params = aoide_core_api::media::tracker::import_files::Params {
                                root_url: media_root_url,
                                ..Default::default()
                            };
                            let intent = media_tracker::Intent::StartImportFiles {
                                collection_uid,
                                params,
                            };
                            return Some(intent.into());
                        }
                        Some(("find-untracked-files", find_untracked_files_matches)) => {
                            let collection_uid = entity.hdr.uid.clone();
                            let media_root_url = find_untracked_files_matches
                                .value_of(MEDIA_ROOT_URL_PARAM)
                                .map(|s| s.parse().expect("URL"))
                                .or_else(|| {
                                    entity
                                        .body
                                        .media_source_config
                                        .source_path
                                        .root_url()
                                        .cloned()
                                        .map(Into::into)
                                });
                            subcommand_submitted = true;
                            let params =
                                aoide_core_api::media::tracker::find_untracked_files::Params {
                                    root_url: media_root_url,
                                    ..Default::default()
                                };
                            let intent = media_tracker::Intent::StartFindUntrackedFiles {
                                collection_uid,
                                params,
                            };
                            return Some(intent.into());
                        }
                        Some((_subcommand, _)) => {
                            unreachable!("Unknown subcommand {}", _subcommand);
                        }
                        None => {
                            println!("{}", app_usage);
                        }
                    },
                    Some(("tracks", matches)) => match matches.subcommand() {
                        Some(("export-all-into-file", matches)) => {
                            let collection_uid = entity.hdr.uid.clone();
                            let output_file_path = matches
                                .value_of(OUTPUT_FILE_PARAM)
                                .expect(OUTPUT_FILE_PARAM)
                                .to_owned();
                            subcommand_submitted = true;
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
                                    resolve_url_from_path: true,
                                    ..Default::default()
                                },
                            };
                            let intent = Intent::ExportTracks {
                                collection_uid,
                                params,
                            };
                            return Some(intent);
                        }
                        Some((_subcommand, _)) => {
                            unreachable!("Unknown subcommand {}", _subcommand);
                        }
                        None => {
                            println!("{}", app_usage);
                        }
                    },
                    Some((_subcommand, _)) => {
                        unreachable!("Unknown subcommand {}", _subcommand);
                    }
                    None => {
                        println!("{}", app_usage);
                    }
                }
            }
            None
        }),
    ));

    // Handle Ctrl-C/SIGINT signals to abort processing
    tokio::spawn({
        let message_tx = message_tx.clone();
        async move {
            if let Err(err) = signal::ctrl_c().await {
                log::error!("Failed to receive Ctrl+C/SIGINT signal: {}", err);
            }
            log::info!("Terminating after receiving Ctrl+C/SIGINT...");
            send_message(&message_tx, Intent::Terminate);
        }
    });

    // Kick off the loop by sending a first message
    // before awaiting its termination
    send_message(&message_tx, Intent::RenderState);
    message_loop.await?;

    Ok(())
}
