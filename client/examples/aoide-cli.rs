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
    collection, handle_messages, media::tracker as media_tracker, prelude::Environment, Intent,
};

use aoide_core::{
    collection::{Collection, MediaSourceConfig},
    entity::EntityUid,
};

use clap::{App, Arg};
use std::{
    env,
    sync::Arc,
    time::{Duration, Instant},
};

const DEFAULT_LOG_FILTER: &str = "info";

const DEFAULT_API_URL: &str = "http://[::1]:8080";

const PROGRESS_POLLING_PERIOD: Duration = Duration::from_millis(1_000);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(DEFAULT_LOG_FILTER))
        .init();

    let default_api_url = env::var("API_URL").unwrap_or(DEFAULT_API_URL.to_owned());

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
            Arg::with_name("api-url")
                .short("a")
                .long("api-url")
                .takes_value(true)
                .required(false)
                .default_value(DEFAULT_API_URL),
        )
        .subcommand(
            App::new("collections")
                .about("Manages collections")
                .subcommand(
                    App::new("create-mixxx")
                        .about("Creates a new mixxx.org collection for Mixxx")
                        .arg(
                            Arg::with_name("title")
                                .help("The title of the new collection")
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("root-url")
                                .help("The file URL of the common root directory that contains all media sources")
                                .required(true),
                        ),
                )
        )
        .subcommand(
            App::new("media-tracker")
                .about("Controls the media tracker")
                .subcommand(
                    App::new("progress").about("Query progress of the running scan/import task"),
                )
                .subcommand(App::new("abort").about("Abort the running task"))
                .subcommand(
                    App::new("status")
                        .about("Queries the status of the media tracker")
                        .arg(
                            Arg::with_name("root-url")
                                .help("The root URL to scan")
                                .required(false),
                        ),
                )
                .subcommand(
                    App::new("scan")
                        .about("Scans directories on the file system for added/modified/removed media sources")
                        .arg(
                            Arg::with_name("root-url")
                                .help("The root URL to scan")
                                .required(false),
                        ),
                )
                .subcommand(
                    App::new("import")
                        .about("Imports media sources on the file system from scanned directories")
                        .arg(
                            Arg::with_name("root-url")
                                .help("The root URL to scan")
                                .required(false),
                        ),
                )
                .subcommand(
                    App::new("untrack")
                        .about("Untracks directories on the file system")
                        .arg(
                            Arg::with_name("root-url")
                                .help("The root URL to scan")
                                .required(true),
                        ),
                ),
        )
        .get_matches();

    let api_url = matches
        .value_of("api-url")
        .unwrap_or(&default_api_url)
        .parse()
        .expect("URL");
    let mut collection_uid = matches
        .value_of("collection-uid")
        .map(|s| s.parse::<EntityUid>().expect("Collection UID"));

    let shared_env = Arc::new(Environment::new(api_url));
    let mut last_media_tracker_progress_fetched = None;
    let mut last_media_tracker_status = None;
    let mut last_media_tracker_progress = None;
    let mut last_media_tracker_scan_outcome = None;
    let mut last_media_tracker_import_outcome = None;
    let mut last_media_tracker_untrack_outcome = None;
    let mut subcommand_submitted = false;
    let message_loop = tokio::spawn(handle_messages(
        shared_env,
        Default::default(),
        Intent::RenderState,
        Box::new(move |state| {
            if !state.last_errors().is_empty() {
                for err in state.last_errors() {
                    log::error!("{}", err);
                }
                return Some(Intent::ClearFirstErrorsBeforeNextRenderState(
                    state.last_errors().len(),
                ));
            }
            if last_media_tracker_progress.as_ref() != state.media_tracker.remote().progress().get()
            {
                last_media_tracker_progress = state
                    .media_tracker
                    .remote()
                    .progress()
                    .get()
                    .map(ToOwned::to_owned);
                if let Some(progress) = &last_media_tracker_progress {
                    log::info!("Media tracker progress: {:?}", progress);
                }
            }
            if last_media_tracker_status.as_ref() != state.media_tracker.remote().status().get() {
                last_media_tracker_status = state
                    .media_tracker
                    .remote()
                    .status()
                    .get()
                    .map(ToOwned::to_owned);
                if let Some(status) = &last_media_tracker_status {
                    log::info!("Media tracker status: {:?}", status);
                }
            }
            if last_media_tracker_scan_outcome.as_ref()
                != state.media_tracker.remote().last_scan_outcome().get_ready()
            {
                last_media_tracker_scan_outcome = state
                    .media_tracker
                    .remote()
                    .last_scan_outcome()
                    .get_ready()
                    .map(ToOwned::to_owned);
                if let Some(outcome) = &last_media_tracker_scan_outcome {
                    log::info!("Scan finished: {:?}", outcome);
                }
            }
            if last_media_tracker_import_outcome.as_ref()
                != state
                    .media_tracker
                    .remote()
                    .last_import_outcome()
                    .get_ready()
            {
                last_media_tracker_import_outcome = state
                    .media_tracker
                    .remote()
                    .last_import_outcome()
                    .get_ready()
                    .map(ToOwned::to_owned);
                if let Some(outcome) = &last_media_tracker_import_outcome {
                    log::info!("Import finished: {:?}", outcome);
                }
            }
            if last_media_tracker_untrack_outcome.as_ref()
                != state
                    .media_tracker
                    .remote()
                    .last_untrack_outcome()
                    .get_ready()
            {
                last_media_tracker_untrack_outcome = state
                    .media_tracker
                    .remote()
                    .last_untrack_outcome()
                    .get_ready()
                    .map(ToOwned::to_owned);
                if let Some(outcome) = &last_media_tracker_untrack_outcome {
                    log::info!("Untrack finished: {:?}", outcome);
                }
            }

            // Only submit a single subcommand
            if subcommand_submitted {
                let next_intent = if state.media_tracker.is_idle() {
                    // Terminate when idle and no task is pending
                    None
                } else {
                    // Periodically refetch and report progress while busy
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
                };
                return next_intent;
            }

            // Commands that don't require an active collection
            if let ("collections", Some(collections_matches)) = matches.subcommand() {
                match collections_matches.subcommand() {
                    ("create-mixxx", Some(create_matches)) => {
                        let title = create_matches.value_of("title").expect("title");
                        let root_url = create_matches
                            .value_of("root-url")
                            .map(|s| s.parse().expect("root-url"))
                            .expect("root-url");
                        let new_collection = Collection {
                            title: title.to_owned(),
                            kind: Some("mixxx.org".to_owned()),
                            notes: None,
                            color: None,
                            media_source_config: MediaSourceConfig {
                                path_kind: aoide_core::media::SourcePathKind::VirtualFilePath,
                                root_url: Some(root_url),
                            },
                        };
                        subcommand_submitted = true;
                        return Some(
                            collection::Intent::CreateNewCollection(new_collection).into(),
                        );
                    }
                    (subcommand, _) => {
                        debug_assert!(subcommand.is_empty());
                        println!("{}", matches.usage());
                    }
                }
            }
            if let ("media-tracker", Some(media_tracker_matches)) = matches.subcommand() {
                if matches!(media_tracker_matches.subcommand(), ("progress", _)) {
                    subcommand_submitted = true;
                    last_media_tracker_progress_fetched = Some(Instant::now());
                    return Some(media_tracker::Intent::FetchProgress.into());
                }
                if matches!(media_tracker_matches.subcommand(), ("abort", _)) {
                    subcommand_submitted = true;
                    return Some(media_tracker::Intent::Abort.into());
                }
            }

            // Select an active collection
            if let Some(available_collections) = state
                .collection
                .remote()
                .available_collections()
                .get_ready()
            {
                if state.collection.active_collection_uid().is_none() {
                    if available_collections.value.is_empty() {
                        log::warn!("No collections available");
                        return None;
                    }
                    if collection_uid.is_none() && available_collections.value.len() == 1 {
                        collection_uid = available_collections
                            .value
                            .iter()
                            .next()
                            .map(|e| e.hdr.uid.clone());
                        debug_assert!(collection_uid.is_some());
                        log::info!(
                            "Activating single collection: {}",
                            collection_uid.as_ref().unwrap()
                        );
                    }
                    if let Some(collection_uid) = &collection_uid {
                        if state
                            .collection
                            .remote()
                            .find_available_collections_by_uid(collection_uid)
                            .is_some()
                        {
                            return Some(
                                collection::Intent::ActivateCollection(Some(
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
                            available_collection
                                .body
                                .notes
                                .as_ref()
                                .map(String::as_str)
                                .unwrap_or(""),
                        );
                    }
                    return None;
                }
            } else {
                if state
                    .collection
                    .remote()
                    .available_collections()
                    .is_unknown()
                {
                    return Some(collection::Intent::FetchAvailableCollections.into());
                }
            }

            if subcommand_submitted {
                return None;
            }

            // Commands that require an active collection
            if let Some(collection) = state.collection.active_collection() {
                log::info!("Active collection: {}", collection.hdr.uid);
                // Only allowed while idle
                if !state.media_tracker.is_idle() {
                    last_media_tracker_progress_fetched = Some(Instant::now());
                    return Some(media_tracker::Intent::FetchProgress.into());
                }
                match matches.subcommand() {
                    ("media-tracker", Some(media_tracker_matches)) => {
                        match media_tracker_matches.subcommand() {
                            ("status", status_matches) => {
                                let collection_uid = collection.hdr.uid.clone();
                                let root_url = status_matches
                                    .and_then(|m| m.value_of("root-url"))
                                    .map(|s| s.parse().expect("URL"))
                                    .or_else(|| {
                                        collection.body.media_source_config.root_url.clone()
                                    });
                                subcommand_submitted = true;
                                last_media_tracker_status = None;
                                return Some(
                                    media_tracker::Intent::FetchStatus {
                                        collection_uid,
                                        root_url,
                                    }
                                    .into(),
                                );
                            }
                            ("scan", scan_matches) => {
                                let collection_uid = collection.hdr.uid.clone();
                                let root_url = scan_matches
                                    .and_then(|m| m.value_of("root-url"))
                                    .map(|s| s.parse().expect("URL"))
                                    .or_else(|| {
                                        collection.body.media_source_config.root_url.clone()
                                    });
                                subcommand_submitted = true;
                                return Some(
                                    media_tracker::Intent::StartScan {
                                        collection_uid,
                                        root_url,
                                    }
                                    .into(),
                                );
                            }
                            ("import", import_matches) => {
                                let collection_uid = collection.hdr.uid.clone();
                                let root_url = import_matches
                                    .and_then(|m| m.value_of("root-url"))
                                    .map(|s| s.parse().expect("URL"))
                                    .or_else(|| {
                                        collection.body.media_source_config.root_url.clone()
                                    });
                                subcommand_submitted = true;
                                return Some(
                                    media_tracker::Intent::StartImport {
                                        collection_uid,
                                        root_url,
                                    }
                                    .into(),
                                );
                            }
                            ("untrack", untrack_matches) => {
                                let collection_uid = collection.hdr.uid.clone();
                                let root_url = untrack_matches
                                    .and_then(|m| m.value_of("root-url"))
                                    .map(|s| s.parse().expect("URL"))
                                    .expect("required");
                                subcommand_submitted = true;
                                return Some(
                                    media_tracker::Intent::Untrack {
                                        collection_uid,
                                        root_url,
                                    }
                                    .into(),
                                );
                            }
                            (subcommand, _) => {
                                debug_assert!(subcommand.is_empty());
                                println!("{}", media_tracker_matches.usage());
                            }
                        }
                    }
                    (subcommand, _) => {
                        debug_assert!(subcommand.is_empty());
                        println!("{}", matches.usage());
                    }
                }
            }
            return None;
        }),
    ));
    message_loop.await?;
    Ok(())
}
