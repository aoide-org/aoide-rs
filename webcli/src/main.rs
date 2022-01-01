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

use aoide_client::{
    models::{
        active_collection, media_sources, media_tracker,
        webcli::{self, Environment, Intent, State},
    },
    prelude::{message_channel, mutable::message_loop, send_message},
};

use aoide_core::{
    collection::{Collection, MediaSourceConfig},
    entity::EntityUid,
};

use aoide_core_api::media::tracker::DirTrackingStatus;
use clap::{App, Arg};
use std::{
    env,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::signal;

const DEFAULT_LOG_FILTER: &str = "info";

const DEFAULT_SERVICE_URL: &str = "http://[::1]:8080";

const PROGRESS_POLLING_PERIOD: Duration = Duration::from_millis(1_000);

const COLLECTION_VFS_ROOT_URL_PARAM: &str = "vfs-root-url";

const MEDIA_ROOT_URL_PARAM: &str = "media-root-url";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(DEFAULT_LOG_FILTER))
        .init();

    let default_service_url =
        env::var("API_URL").unwrap_or_else(|_| DEFAULT_SERVICE_URL.to_owned());

    let matches = App::new("aoide-cli")
        .about("An experimental CLI for performing tasks on aoide")
        .version("0.0")
        .arg(
            Arg::with_name("collection-uid")
                .short("c")
                .long("collection-uid")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("service-url")
                .short("s")
                .long("service-url")
                .takes_value(true)
                .required(false)
                .default_value(DEFAULT_SERVICE_URL),
        )
        .subcommand(
            App::new("collections")
                .about("Tasks for managing collections")
                .subcommand(
                    App::new("create-mixxx")
                        .about("Creates a new mixxx.org collection for Mixxx")
                        .arg(
                            Arg::with_name("title")
                                .help("The title of the new collection")
                                .required(true),
                        )
                        .arg(
                            Arg::with_name(COLLECTION_VFS_ROOT_URL_PARAM)
                                .help("The file URL of the common root directory that contains all media sources")
                                .required(true),
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
                            Arg::with_name(MEDIA_ROOT_URL_PARAM)
                                .help("The common root URL or directory that should be considered")
                                .required(false),
                        ),
                )
                .subcommand(
                    App::new("purge-untracked")
                        .about("Purges untracked media sources including their tracks")
                        .arg(
                            Arg::with_name(MEDIA_ROOT_URL_PARAM)
                                .help("The URL of the root directory containing tracked media files to be purged")
                                .required(false),
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
                            Arg::with_name(MEDIA_ROOT_URL_PARAM)
                                .help("The URL of the root directory containing tracked media files to be queried")
                                .required(false),
                        ),
                )
                .subcommand(
                    App::new("scan-directories")
                        .about("Scans directories on the file system for added/modified/removed media sources")
                        .arg(
                            Arg::with_name(MEDIA_ROOT_URL_PARAM)
                                .help("The URL of the root directory containing tracked media files to be scanned")
                                .required(false),
                        ),
                )
                .subcommand(
                    App::new("untrack-directories")
                        .about("Untracks directories on the file system")
                        .arg(
                            Arg::with_name(MEDIA_ROOT_URL_PARAM)
                                .help("The URL of the root directory containing tracked media files to be untracked")
                                .required(true),
                        ),
                )
                .subcommand(
                    App::new("untrack-orphaned-directories")
                        .about("Untracks orphaned directories that have disappeared from the file system (deleted)")
                        .arg(
                            Arg::with_name(MEDIA_ROOT_URL_PARAM)
                                .help("The URL of the root directory containing tracked media files to be untracked")
                                .required(false),
                        ),
                )
                .subcommand(
                    App::new("import-files")
                        .about("Imports media sources on the file system from scanned directories")
                        .arg(
                            Arg::with_name(MEDIA_ROOT_URL_PARAM)
                                .help("The URL of the root directory containing tracked media files to be imported")
                                .required(false),
                        ),
                )
                .subcommand(
                    App::new("find-untracked-files")
                        .about("Scans directories on the file system for untracked entries")
                        .arg(
                            Arg::with_name(MEDIA_ROOT_URL_PARAM)
                                .help("The URL of the root directory containing tracked media files to be scanned")
                                .required(false),
                        ),
                ),
        )
        .get_matches();

    let api_url = matches
        .value_of("api-url")
        .unwrap_or(&default_service_url)
        .parse()
        .expect("URL");
    let mut collection_uid = matches
        .value_of("collection-uid")
        .map(|s| s.parse::<EntityUid>().expect("Collection UID"));

    let shared_env = Arc::new(Environment::new(api_url));
    let (message_tx, message_rx) = message_channel();

    let mut last_media_tracker_progress_fetched = None;
    let mut last_media_tracker_status = None;
    let mut last_media_tracker_progress = None;
    let mut last_media_tracker_scan_directories_outcome = None;
    let mut last_media_tracker_import_files_outcome = None;
    let mut last_media_tracker_untrack_directories_outcome = None;
    let mut last_media_tracker_find_untracked_files_outcome = None;
    let mut subcommand_submitted = false;
    let message_loop = tokio::spawn(message_loop(
        shared_env,
        (message_tx.clone(), message_rx),
        Default::default(),
        Box::new(move |state: &State| {
            if !state.last_errors().is_empty() {
                for err in state.last_errors() {
                    log::error!("{}", err);
                }
                // Terminate after errors occurred
                return Some(Intent::Terminate);
            }
            if last_media_tracker_progress.as_ref()
                != state.media_tracker.remote_view().progress.get()
            {
                last_media_tracker_progress = state
                    .media_tracker
                    .remote_view()
                    .progress
                    .get()
                    .map(ToOwned::to_owned);
                if let Some(progress) = &last_media_tracker_progress {
                    log::info!("Media tracker progress: {:?}", progress);
                }
            }
            if last_media_tracker_status.as_ref() != state.media_tracker.remote_view().status.get()
            {
                last_media_tracker_status = state
                    .media_tracker
                    .remote_view()
                    .status
                    .get()
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
                    .get_ready()
            {
                last_media_tracker_scan_directories_outcome = state
                    .media_tracker
                    .remote_view()
                    .last_scan_directories_outcome
                    .get_ready()
                    .map(ToOwned::to_owned);
                if let Some(outcome) = &last_media_tracker_scan_directories_outcome {
                    log::info!("Scan finished: {:?}", outcome);
                }
            }
            if last_media_tracker_import_files_outcome.as_ref()
                != state
                    .media_tracker
                    .remote_view()
                    .last_import_files_outcome
                    .get_ready()
            {
                last_media_tracker_import_files_outcome = state
                    .media_tracker
                    .remote_view()
                    .last_import_files_outcome
                    .get_ready()
                    .map(ToOwned::to_owned);
                if let Some(outcome) = &last_media_tracker_import_files_outcome {
                    log::info!("Import finished: {:?}", outcome);
                }
            }
            if last_media_tracker_untrack_directories_outcome.as_ref()
                != state
                    .media_tracker
                    .remote_view()
                    .last_untrack_directories_outcome
                    .get_ready()
            {
                last_media_tracker_untrack_directories_outcome = state
                    .media_tracker
                    .remote_view()
                    .last_untrack_directories_outcome
                    .get_ready()
                    .map(ToOwned::to_owned);
                if let Some(outcome) = &last_media_tracker_untrack_directories_outcome {
                    log::info!("Untrack finished: {:?}", outcome);
                }
            }
            if last_media_tracker_find_untracked_files_outcome.as_ref()
                != state
                    .media_tracker
                    .remote_view()
                    .last_find_untracked_files_outcome
                    .get_ready()
            {
                last_media_tracker_find_untracked_files_outcome = state
                    .media_tracker
                    .remote_view()
                    .last_find_untracked_files_outcome
                    .get_ready()
                    .map(ToOwned::to_owned);
                if let Some(outcome) = &last_media_tracker_find_untracked_files_outcome {
                    log::info!("Finding untracked files finished: {:?}", outcome);
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
                    Some(webcli::Intent::Terminate)
                } else {
                    // Periodically refetch and report progress while busy
                    if state.media_tracker.remote_view().is_pending() {
                        if let Some(last_fetched) = last_media_tracker_progress_fetched {
                            let now = Instant::now();
                            if now >= last_fetched {
                                let not_before = now + PROGRESS_POLLING_PERIOD;
                                last_media_tracker_progress_fetched = Some(not_before);
                                let intent = Intent::TimedIntent {
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
            if let ("collections", Some(collections_matches)) = matches.subcommand() {
                match collections_matches.subcommand() {
                    ("create-mixxx", Some(create_matches)) => {
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
                        return Some(
                            active_collection::Intent::CreateNewCollection(new_collection).into(),
                        );
                    }
                    (subcommand, _) => {
                        debug_assert!(subcommand.is_empty());
                        println!("{}", matches.usage());
                    }
                }
            }
            if let ("media", Some(matches)) = matches.subcommand() {
                if matches!(matches.subcommand(), ("progress", _)) {
                    subcommand_submitted = true;
                    last_media_tracker_progress_fetched = Some(Instant::now());
                    return Some(media_tracker::Intent::FetchProgress.into());
                }
                if matches!(matches.subcommand(), ("abort", _)) {
                    subcommand_submitted = true;
                    return Some(webcli::Intent::AbortPendingRequest);
                }
            }

            // Select an active collection
            if let Some(available_collections) = state
                .active_collection
                .remote_view()
                .available_collections
                .get_ready()
            {
                if state.active_collection.active_collection_uid().is_none() {
                    if available_collections.value.is_empty() {
                        log::warn!("No collections available");
                        return None;
                    }
                    if collection_uid.is_none() && available_collections.value.len() == 1 {
                        collection_uid = available_collections
                            .value
                            .get(0)
                            .map(|e| e.hdr.uid.clone());
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
                            .find_available_collection_by_uid(collection_uid)
                            .is_some()
                        {
                            return Some(
                                active_collection::Intent::ActivateCollection(Some(
                                    collection_uid.to_owned(),
                                ))
                                .into(),
                            );
                        } else {
                            log::warn!("Collection not available: {}", collection_uid);
                        }
                    }
                    println!("Available collections:");
                    for available_collection in available_collections.value.iter() {
                        println!(
                            "{}: {} | {}",
                            available_collection.hdr.uid,
                            available_collection.body.title,
                            available_collection.body.notes.as_deref().unwrap_or(""),
                        );
                    }
                    return None;
                }
            } else if state
                .active_collection
                .remote_view()
                .available_collections
                .is_unknown()
            {
                return Some(active_collection::Intent::FetchAvailableCollections.into());
            }

            if subcommand_submitted {
                return None;
            }

            // Commands that require an active collection
            if let Some(collection) = state.active_collection.active_collection() {
                log::info!("Active collection: {}", collection.hdr.uid);
                if state.is_pending() {
                    last_media_tracker_progress_fetched = Some(Instant::now());
                    return Some(media_tracker::Intent::FetchProgress.into());
                }
                match matches.subcommand() {
                    ("media-sources", Some(matches)) => match matches.subcommand() {
                        ("purge-orphaned", matches) => {
                            let collection_uid = collection.hdr.uid.clone();
                            let media_root_url = matches
                                .and_then(|m| m.value_of(MEDIA_ROOT_URL_PARAM))
                                .map(|s| s.parse().expect("URL"));
                            subcommand_submitted = true;
                            return Some(
                                media_sources::Intent::PurgeOrphaned {
                                    collection_uid,
                                    params: aoide_core_api::media::source::purge_orphaned::Params {
                                        root_url: media_root_url,
                                    },
                                }
                                .into(),
                            );
                        }
                        ("purge-untracked", matches) => {
                            let collection_uid = collection.hdr.uid.clone();
                            let media_root_url = matches
                                .and_then(|m| m.value_of(MEDIA_ROOT_URL_PARAM))
                                .map(|s| s.parse().expect("URL"));
                            subcommand_submitted = true;
                            return Some(
                                media_sources::Intent::PurgeUntracked {
                                    collection_uid,
                                    params:
                                        aoide_core_api::media::source::purge_untracked::Params {
                                            root_url: media_root_url,
                                        },
                                }
                                .into(),
                            );
                        }
                        (subcommand, _) => {
                            debug_assert!(subcommand.is_empty());
                            println!("{}", matches.usage());
                        }
                    },
                    ("media-tracker", Some(matches)) => match matches.subcommand() {
                        ("query-status", matches) => {
                            let collection_uid = collection.hdr.uid.clone();
                            let media_root_url = matches
                                .and_then(|m| m.value_of(MEDIA_ROOT_URL_PARAM))
                                .map(|s| s.parse().expect("URL"))
                                .or_else(|| {
                                    collection
                                        .body
                                        .media_source_config
                                        .source_path
                                        .root_url()
                                        .cloned()
                                        .map(Into::into)
                                });
                            subcommand_submitted = true;
                            last_media_tracker_status = None;
                            return Some(
                                media_tracker::Intent::FetchStatus {
                                    collection_uid,
                                    root_url: media_root_url,
                                }
                                .into(),
                            );
                        }
                        ("scan-directories", matches) => {
                            let collection_uid = collection.hdr.uid.clone();
                            let media_root_url = matches
                                .and_then(|m| m.value_of(MEDIA_ROOT_URL_PARAM))
                                .map(|s| s.parse().expect("URL"))
                                .or_else(|| {
                                    collection
                                        .body
                                        .media_source_config
                                        .source_path
                                        .root_url()
                                        .cloned()
                                        .map(Into::into)
                                });
                            subcommand_submitted = true;
                            return Some(
                                media_tracker::Intent::StartScanDirectories {
                                    collection_uid,
                                    params:
                                        aoide_core_api::media::tracker::scan_directories::Params {
                                            root_url: media_root_url,
                                            ..Default::default()
                                        },
                                }
                                .into(),
                            );
                        }
                        ("untrack-directories", matches) => {
                            let collection_uid = collection.hdr.uid.clone();
                            let media_root_url = matches
                                .and_then(|m| m.value_of(MEDIA_ROOT_URL_PARAM))
                                .map(|s| s.parse().expect("URL"))
                                .expect("required");
                            subcommand_submitted = true;
                            return Some(
                                media_tracker::Intent::UntrackDirectories {
                                    collection_uid,
                                    params: aoide_core_api::media::tracker::untrack_directories::Params {
                                        root_url: Some(media_root_url),
                                        status: None,
                                    }
                                }
                                .into(),
                            );
                        }
                        ("untrack-orphaned-directories", matches) => {
                            let collection_uid = collection.hdr.uid.clone();
                            let media_root_url = matches
                                .and_then(|m| m.value_of(MEDIA_ROOT_URL_PARAM))
                                .map(|s| s.parse().expect("URL"));
                            subcommand_submitted = true;
                            return Some(
                                media_tracker::Intent::UntrackDirectories {
                                    collection_uid,
                                    params: aoide_core_api::media::tracker::untrack_directories::Params {
                                        root_url: media_root_url,
                                        status: Some(DirTrackingStatus::Orphaned),
                                    }
                                }
                                .into(),
                            );
                        }
                        ("import-files", matches) => {
                            let collection_uid = collection.hdr.uid.clone();
                            let media_root_url = matches
                                .and_then(|m| m.value_of(MEDIA_ROOT_URL_PARAM))
                                .map(|s| s.parse().expect("URL"));
                            subcommand_submitted = true;
                            return Some(
                                media_tracker::Intent::StartImportFiles {
                                    collection_uid,
                                    params: aoide_core_api::media::tracker::import_files::Params {
                                        root_url: media_root_url,
                                        ..Default::default()
                                    },
                                }
                                .into(),
                            );
                        }
                        ("find-untracked-files", find_untracked_files_matches) => {
                            let collection_uid = collection.hdr.uid.clone();
                            let media_root_url = find_untracked_files_matches
                                .and_then(|m| m.value_of(MEDIA_ROOT_URL_PARAM))
                                .map(|s| s.parse().expect("URL"))
                                .or_else(|| {
                                    collection
                                        .body
                                        .media_source_config
                                        .source_path
                                        .root_url()
                                        .cloned()
                                        .map(Into::into)
                                });
                            subcommand_submitted = true;
                            return Some(
                                media_tracker::Intent::StartFindUntrackedFiles {
                                    collection_uid,
                                    params: aoide_core_api::media::tracker::find_untracked_files::Params {
                                        root_url: media_root_url,
                                        ..Default::default()
                                    }
                                }
                                .into(),
                            );
                        }
                        (subcommand, _) => {
                            debug_assert!(subcommand.is_empty());
                            println!("{}", matches.usage());
                        }
                    },
                    (subcommand, _) => {
                        debug_assert!(subcommand.is_empty());
                        println!("{}", matches.usage());
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