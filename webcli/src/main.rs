// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    env,
    ops::Not as _,
    sync::Arc,
    time::{Duration, Instant},
};

use aoide_client::{
    models::{collection, media_source, media_tracker},
    util::remote::DataSnapshot,
};
use aoide_core::{
    collection::MediaSourceConfig, media::content::ContentPath, Collection, CollectionEntity,
    CollectionUid,
};
use aoide_core_api::{
    media::{
        tracker::{import_files::ImportedSourceWithIssues, DirTrackingStatus},
        SyncMode,
    },
    track::search::{SortField, SortOrder},
};
use clap::{builder::StyledStr, Arg, ArgMatches, Command};
use infect::{
    consume_messages, message_channel, MessagePort, MessagesConsumed, ModelChanged, ModelRender,
    TaskContext,
};
use model::{EffectApplied, IntentHandled};
use tokio::signal;

mod model;
use self::model::{Effect, Environment, Intent, Model, Task};
use crate::model::ExportTracksParams;

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

const MESSAGE_CHANNEL_CAPACITY: usize = 1000;

#[derive(Debug)]
struct CliModel {
    matches: ArgMatches,
    model: Model,
}

impl CliModel {
    fn new(matches: ArgMatches) -> Self {
        Self {
            matches,
            model: Default::default(),
        }
    }
}

impl infect::Model for CliModel {
    type Intent = Intent;
    type IntentRejected = Intent;
    type Effect = Effect;
    type Task = Task;
    type RenderHint = ModelChanged;

    fn handle_intent(&mut self, intent: Self::Intent) -> IntentHandled {
        intent.handle_on(&mut self.model)
    }

    fn apply_effect(&mut self, effect: Self::Effect) -> EffectApplied {
        effect.apply_on(&mut self.model)
    }
}

struct RenderCliModel {
    app_usage: StyledStr,

    collection_uid: Option<CollectionUid>,
    subcommand_submitted: bool,

    last_media_sources_purge_orphaned_outcome:
        Option<DataSnapshot<aoide_core_api::media::source::purge_orphaned::Outcome>>,
    last_media_sources_purge_untracked_outcome:
        Option<DataSnapshot<aoide_core_api::media::source::purge_untracked::Outcome>>,
    last_media_tracker_progress_fetched: Option<Instant>,
    last_media_tracker_progress: Option<DataSnapshot<aoide_core_api::media::tracker::Progress>>,
    last_media_tracker_status: Option<DataSnapshot<aoide_core_api::media::tracker::Status>>,
    last_media_tracker_scan_directories_outcome:
        Option<DataSnapshot<aoide_core_api::media::tracker::scan_directories::Outcome>>,
    last_media_tracker_untrack_directories_outcome:
        Option<DataSnapshot<aoide_core_api::media::tracker::untrack_directories::Outcome>>,
    last_media_tracker_import_files_outcome:
        Option<DataSnapshot<aoide_core_api::media::tracker::import_files::Outcome>>,
    last_media_tracker_find_untracked_files_outcome:
        Option<DataSnapshot<aoide_core_api::media::tracker::find_untracked_files::Outcome>>,
}

impl ModelRender for RenderCliModel {
    type Model = CliModel;

    #[allow(clippy::too_many_lines)] // TODO
    fn render_model(
        &mut self,
        cli_model: &Self::Model,
        model_changed: ModelChanged,
    ) -> Option<<Self::Model as infect::Model>::Intent> {
        debug_assert_eq!(ModelChanged::MaybeChanged, model_changed);
        let Self {
            app_usage,
            collection_uid,
            last_media_sources_purge_orphaned_outcome,
            last_media_sources_purge_untracked_outcome,
            last_media_tracker_find_untracked_files_outcome,
            last_media_tracker_import_files_outcome,
            last_media_tracker_progress,
            last_media_tracker_progress_fetched,
            last_media_tracker_scan_directories_outcome,
            last_media_tracker_status,
            last_media_tracker_untrack_directories_outcome,
            subcommand_submitted,
        } = self;
        let CliModel { matches, model } = cli_model;

        if model.last_errors().next().is_some() {
            for err in model.last_errors() {
                log::error!("{err}");
            }
            // Terminate after errors occurred
            return Some(Intent::Terminate);
        }

        if last_media_sources_purge_orphaned_outcome.as_ref()
            != model
                .media_sources
                .remote_view()
                .last_purge_orphaned_outcome
                .last_snapshot()
        {
            *last_media_sources_purge_orphaned_outcome = model
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
            != model
                .media_sources
                .remote_view()
                .last_purge_untracked_outcome
                .last_snapshot()
        {
            *last_media_sources_purge_untracked_outcome = model
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
            != model.media_tracker.remote_view().progress.last_snapshot()
        {
            *last_media_tracker_progress = model
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
            != model.media_tracker.remote_view().status.last_snapshot()
        {
            *last_media_tracker_status = model
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
            != model
                .media_tracker
                .remote_view()
                .last_scan_directories_outcome
                .last_snapshot()
        {
            *last_media_tracker_scan_directories_outcome = model
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
            != model
                .media_tracker
                .remote_view()
                .last_untrack_directories_outcome
                .last_snapshot()
        {
            *last_media_tracker_untrack_directories_outcome = model
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
            != model
                .media_tracker
                .remote_view()
                .last_import_files_outcome
                .last_snapshot()
        {
            *last_media_tracker_import_files_outcome = model
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
                    let ImportedSourceWithIssues { path, messages } = imported_source_with_issues;
                    debug_assert!(!messages.is_empty());
                    log::warn!("{}: {}", path, messages.join(" | "));
                }
            }
        }
        if last_media_tracker_find_untracked_files_outcome.as_ref()
            != model
                .media_tracker
                .remote_view()
                .last_find_untracked_files_outcome
                .last_snapshot()
        {
            *last_media_tracker_find_untracked_files_outcome = model
                .media_tracker
                .remote_view()
                .last_find_untracked_files_outcome
                .last_snapshot()
                .map(ToOwned::to_owned);
            if let Some(outcome) = &last_media_tracker_find_untracked_files_outcome {
                log::info!("Finding untracked media files succeeded: {outcome:?}");
                if !outcome.value.content_paths.is_empty() {
                    log::info!(
                        "Found {num_untracked_entities} untracked entries on file \
                         system:\n{content_paths}",
                        num_untracked_entities = outcome.value.content_paths.len(),
                        content_paths = outcome
                            .value
                            .content_paths
                            .iter()
                            .map(ContentPath::as_str)
                            .collect::<Vec<_>>()
                            .join("\n"),
                    );
                }
            }
        }

        // Only submit a single subcommand
        if *subcommand_submitted {
            // Periodically refetch and report progress while busy
            if model.is_terminating() {
                return None;
            }
            let next_intent = if let Some(last_fetched) = last_media_tracker_progress_fetched {
                let now = Instant::now();
                if now < *last_fetched {
                    return None;
                }
                let not_before = now + PROGRESS_POLLING_PERIOD;
                *last_media_tracker_progress_fetched = Some(not_before);
                Intent::Schedule {
                    not_before,
                    intent: Box::new(media_tracker::Intent::FetchProgress.into()),
                }
            } else {
                *last_media_tracker_progress_fetched = Some(Instant::now());
                media_tracker::Intent::FetchProgress.into()
            };
            return Some(next_intent);
        }

        // Commands that don't require an active collection
        if let Some(("media-tracker", matches)) = matches.subcommand() {
            if matches!(matches.subcommand(), Some(("progress", _))) {
                *subcommand_submitted = true;
                *last_media_tracker_progress_fetched = Some(Instant::now());
                let intent = media_tracker::Intent::FetchProgress;
                return Some(intent.into());
            }
            if matches!(matches.subcommand(), Some(("abort", _))) {
                *subcommand_submitted = true;
                let intent = Intent::AbortPendingRequest;
                return Some(intent);
            }
        }

        if *subcommand_submitted {
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
                    title: title.clone(),
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
                *subcommand_submitted = true;
                let intent = collection::Intent::CreateEntity { new_collection };
                return Some(intent.into());
            }
            Some(("media-sources", matches)) => match matches.subcommand() {
                Some(("purge-orphaned", matches)) => {
                    require_active_collection(matches, model, collection_uid).map(|entity| {
                        let collection_uid = entity.hdr.uid.clone();
                        let media_root_url = matches
                            .get_one::<String>(MEDIA_ROOT_URL_ARG)
                            .map(|s| s.parse().expect("URL"));
                        let params = aoide_core_api::media::source::purge_orphaned::Params {
                            root_url: media_root_url,
                        };
                        *subcommand_submitted = true;
                        let intent = media_source::Intent::PurgeOrphaned {
                            collection_uid,
                            params,
                        };
                        Some(intent.into())
                    })
                }
                Some(("purge-untracked", matches)) => {
                    require_active_collection(matches, model, collection_uid).map(|entity| {
                        let collection_uid = entity.hdr.uid.clone();
                        let media_root_url = matches
                            .get_one::<String>(MEDIA_ROOT_URL_ARG)
                            .map(|s| s.parse().expect("URL"));
                        let params = aoide_core_api::media::source::purge_untracked::Params {
                            root_url: media_root_url,
                        };
                        *subcommand_submitted = true;
                        let intent = media_source::Intent::PurgeUntracked {
                            collection_uid,
                            params,
                        };
                        Some(intent.into())
                    })
                }
                Some((subcommand, _)) => {
                    unreachable!("Unknown subcommand {subcommand}");
                }
                None => Err(None),
            },
            Some(("media-tracker", matches)) => match matches.subcommand() {
                Some(("query-status", matches)) => {
                    require_active_collection(matches, model, collection_uid).map(|entity| {
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
                        *last_media_tracker_status = None;
                        let params = aoide_core_api::media::tracker::query_status::Params {
                            root_url: media_root_url,
                        };
                        *subcommand_submitted = true;
                        let intent =
                            media_tracker::Intent::FetchStatus(media_tracker::FetchStatus {
                                collection_uid,
                                params,
                            });
                        Some(intent.into())
                    })
                }
                Some(("scan-directories", matches)) => {
                    require_active_collection(matches, model, collection_uid).map(|entity| {
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
                        let params = aoide_core_api::media::tracker::scan_directories::Params {
                            root_url: media_root_url,
                            ..Default::default()
                        };
                        *subcommand_submitted = true;
                        let intent = media_tracker::Intent::StartScanDirectories(
                            media_tracker::StartScanDirectories {
                                collection_uid,
                                params,
                            },
                        );
                        Some(intent.into())
                    })
                }
                Some(("untrack-directories", matches)) => {
                    require_active_collection(matches, model, collection_uid).map(|entity| {
                        let collection_uid = entity.hdr.uid.clone();
                        let media_root_url = matches
                            .get_one::<String>(MEDIA_ROOT_URL_ARG)
                            .map(|s| s.parse().expect("URL"))
                            .expect("required");
                        let params = aoide_core_api::media::tracker::untrack_directories::Params {
                            root_url: Some(media_root_url),
                            status: None,
                        };
                        *subcommand_submitted = true;
                        let intent = media_tracker::Intent::UntrackDirectories(
                            media_tracker::UntrackDirectories {
                                collection_uid,
                                params,
                            },
                        );
                        Some(intent.into())
                    })
                }
                Some(("untrack-orphaned-directories", matches)) => {
                    require_active_collection(matches, model, collection_uid).map(|entity| {
                        let collection_uid = entity.hdr.uid.clone();
                        let media_root_url = matches
                            .get_one::<String>(MEDIA_ROOT_URL_ARG)
                            .map(|s| s.parse().expect("URL"));
                        let params = aoide_core_api::media::tracker::untrack_directories::Params {
                            root_url: media_root_url,
                            status: Some(DirTrackingStatus::Orphaned),
                        };
                        *subcommand_submitted = true;
                        let intent = media_tracker::Intent::UntrackDirectories(
                            media_tracker::UntrackDirectories {
                                collection_uid,
                                params,
                            },
                        );
                        Some(intent.into())
                    })
                }
                Some(("import-files", matches)) => {
                    require_active_collection(matches, model, collection_uid).map(|entity| {
                        let collection_uid = entity.hdr.uid.clone();
                        let media_root_url = matches
                            .get_one::<String>(MEDIA_ROOT_URL_ARG)
                            .map(|s| s.parse().expect("URL"));
                        let params = aoide_core_api::media::tracker::import_files::Params {
                            root_url: media_root_url,
                            sync_mode: SyncMode::Modified,
                        };
                        *subcommand_submitted = true;
                        let intent = media_tracker::Intent::StartImportFiles(
                            media_tracker::StartImportFiles {
                                collection_uid,
                                params,
                            },
                        );
                        Some(intent.into())
                    })
                }
                Some(("find-untracked-files", matches)) => {
                    require_active_collection(matches, model, collection_uid).map(|entity| {
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
                        let params = aoide_core_api::media::tracker::find_untracked_files::Params {
                            root_url: media_root_url,
                            ..Default::default()
                        };
                        *subcommand_submitted = true;
                        let intent = media_tracker::Intent::StartFindUntrackedFiles(
                            media_tracker::StartFindUntrackedFiles {
                                collection_uid,
                                params,
                            },
                        );
                        Some(intent.into())
                    })
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
                    require_active_collection(matches, model, collection_uid).map(|entity| {
                        let collection_uid = entity.hdr.uid.clone();
                        let params = aoide_core_api::track::find_unsynchronized::Params {
                            resolve_url_from_content_path: Some(Default::default()),
                            content_path_predicate: None,
                        };
                        *subcommand_submitted = true;
                        let intent = Intent::FindUnsynchronizedTracks {
                            collection_uid,
                            params,
                        };
                        Some(intent)
                    })
                }
                Some(("export-all-into-file", matches)) => {
                    require_active_collection(matches, model, collection_uid).map(|entity| {
                        let collection_uid = entity.hdr.uid.clone();
                        let output_file_path = matches
                            .get_one::<String>(OUTPUT_FILE_ARG)
                            .expect(OUTPUT_FILE_ARG)
                            .clone();
                        let params = ExportTracksParams {
                            output_file_path: output_file_path.into(),
                            track_search: aoide_core_api::track::search::Params {
                                filter: None,
                                ordering: vec![SortOrder {
                                    field: SortField::UpdatedAt,
                                    direction: aoide_core_api::sorting::SortDirection::Descending,
                                }],
                                // TODO: Configurable?
                                resolve_url_from_content_path: Some(Default::default()),
                            },
                        };
                        *subcommand_submitted = true;
                        let intent = Intent::ExportTracks {
                            collection_uid,
                            params,
                        };
                        Some(intent)
                    })
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
            if intent.is_none() && !model.is_pending() {
                println!("{app_usage}");
            }
            intent
        })
    }
}

#[tokio::main]
#[allow(clippy::too_many_lines)] // TODO
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
                .default_value(DEFAULT_WEBSRV_URL),
        )
        .subcommand(
            Command::new("create-collection")
                .about("Creates a new collection")
                .arg(
                    Arg::new(CREATE_COLLECTION_TITLE_ARG)
                        .long(CREATE_COLLECTION_TITLE_ARG)
                        .help("The `title` of the new collection")
                        .num_args(1)
                        .required(true),
                )
                .arg(
                    Arg::new(CREATE_COLLECTION_KIND_ARG)
                        .long(CREATE_COLLECTION_KIND_ARG)
                        .help("The `kind` of the new collection")
                        .num_args(1)
                        .required(false),
                )
                .arg(
                    Arg::new(CREATE_COLLECTION_VFS_ROOT_URL_ARG)
                        .long(CREATE_COLLECTION_VFS_ROOT_URL_ARG)
                        .help(
                            "The file URL of the common root directory that contains all media \
                             sources",
                        )
                        .num_args(1)
                        .required(true),
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
                        .arg(active_collection_title_arg.clone())
                        .arg(media_root_url_arg.clone()),
                )
                .subcommand(
                    Command::new("purge-untracked")
                        .about("Purges untracked media sources including their tracks")
                        .arg(active_collection_title_arg.clone())
                        .arg(media_root_url_arg),
                )
        })
        .subcommand(
            Command::new("tracks")
                .about("Tasks for tracks")
                .subcommand(
                    Command::new("find-unsynchronized")
                        .about("Find all tracks with unsynchronized media sources")
                        .arg(active_collection_title_arg.clone()),
                )
                .subcommand(
                    Command::new("export-all-into-file")
                        .about("Exports all tracks of the collection into a JSON file")
                        .arg(active_collection_title_arg.clone())
                        .arg(
                            Arg::new(OUTPUT_FILE_ARG)
                                .help("The output file path for writing JSON data")
                                .num_args(1)
                                .required(true),
                        ),
                ),
        )
        .subcommand({
            let media_root_url_arg = Arg::new(MEDIA_ROOT_URL_ARG)
                .help("The URL of the root directory containing tracked media files")
                .num_args(1)
                .required(false);
            Command::new("media-tracker")
                .about("Tasks for the media tracker")
                .subcommand(Command::new("progress").about("Query progress of a pending task"))
                .subcommand(Command::new("abort").about("Abort the current task"))
                .subcommand(
                    Command::new("status")
                        .about("Queries the status of the media tracker")
                        .arg(active_collection_title_arg.clone())
                        .arg(media_root_url_arg.clone()),
                )
                .subcommand(
                    Command::new("scan-directories")
                        .about(
                            "Scans directories on the file system for added/modified/removed \
                             media sources",
                        )
                        .arg(active_collection_title_arg.clone())
                        .arg(media_root_url_arg.clone()),
                )
                .subcommand(
                    Command::new("untrack-directories")
                        .about("Untracks directories on the file system")
                        .arg(active_collection_title_arg.clone())
                        .arg(media_root_url_arg.clone()),
                )
                .subcommand(
                    Command::new("untrack-orphaned-directories")
                        .about(
                            "Untracks orphaned directories that have disappeared from the file \
                             system (deleted)",
                        )
                        .arg(active_collection_title_arg.clone())
                        .arg(media_root_url_arg.clone()),
                )
                .subcommand(
                    Command::new("import-files")
                        .about("Imports media sources on the file system from scanned directories")
                        .arg(active_collection_title_arg.clone())
                        .arg(media_root_url_arg.clone()),
                )
                .subcommand(
                    Command::new("find-untracked-files")
                        .about("Scans directories on the file system for untracked entries")
                        .arg(active_collection_title_arg.clone())
                        .arg(media_root_url_arg),
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
    let (message_tx, mut message_rx) = message_channel(MESSAGE_CHANNEL_CAPACITY);
    let mut message_port = MessagePort::new(message_tx);
    let mut task_context = TaskContext {
        message_port: message_port.clone(),
        task_executor: Arc::clone(&shared_env),
    };

    let mut model = CliModel::new(matches);
    let mut render_model = RenderCliModel {
        app_usage,
        collection_uid: None,
        last_media_sources_purge_orphaned_outcome: None,
        last_media_sources_purge_untracked_outcome: None,
        last_media_tracker_find_untracked_files_outcome: None,
        last_media_tracker_import_files_outcome: None,
        last_media_tracker_progress: None,
        last_media_tracker_progress_fetched: None,
        last_media_tracker_scan_directories_outcome: None,
        last_media_tracker_status: None,
        last_media_tracker_untrack_directories_outcome: None,
        subcommand_submitted: false,
    };
    let message_loop = {
        let mut message_port = message_port.clone();
        async move {
            let message_port = &mut message_port;
            loop {
                match consume_messages(
                    &mut message_rx,
                    &mut task_context,
                    &mut model,
                    &mut render_model,
                )
                .await
                {
                    MessagesConsumed::IntentRejected(intent) => match intent {
                        Intent::MediaTracker(media_tracker::Intent::FetchProgress) => {
                            message_port.submit_intent(Intent::Terminate);
                            continue;
                        }
                        Intent::Terminate => {
                            debug_assert!(model.model.is_terminating());
                        }
                        _ => {
                            log::warn!("Continuing message loop after intent rejected: {intent:?}");
                            continue;
                        }
                    },
                    MessagesConsumed::ChannelClosed => (),
                    MessagesConsumed::NoProgress => {
                        if !shared_env.all_tasks_finished() {
                            log::info!("Continuing message loop until all tasks finished");
                            continue;
                        }
                    }
                }
                log::info!("Exiting message loop");
                break;
            }
        }
    };
    let message_loop = tokio::spawn(message_loop);

    // Handle Ctrl-C/SIGINT signals to abort processing
    tokio::spawn({
        let mut message_port = message_port.clone();
        async move {
            if let Err(err) = signal::ctrl_c().await {
                log::error!("Failed to receive Ctrl-C/SIGINT signal: {err}");
            }
            log::info!("Terminating after receiving Ctrl-C/SIGINT...");
            message_port.submit_intent(Intent::Terminate);
        }
    });

    // Kick off the loop by sending a first message
    // before awaiting its termination
    message_port.submit_intent(Intent::RenderModel);
    message_loop.await?;

    Ok(())
}

fn require_active_collection<'s>(
    matches: &ArgMatches,
    model: &'s Model,
    collection_uid: &mut Option<CollectionUid>,
) -> Result<&'s CollectionEntity, Option<Intent>> {
    if let Some(entity) = model.active_collection.active_entity() {
        debug_assert!(!model.is_pending());
        log::info!(
            "Active collection: '{}' ({})",
            entity.body.title,
            entity.hdr.uid
        );
        return Ok(entity);
    }
    let Some(collection_title) = matches.get_one::<String>(ACTIVE_COLLECTION_TITLE_ARG) else {
        return Err(None);
    };
    if let Some(filtered_entities) = model
        .active_collection
        .remote_view()
        .filtered_entities
        .last_snapshot()
    {
        // Activate an existing collection
        if model.active_collection.active_entity_uid().is_none() {
            if filtered_entities.value.is_empty() {
                log::warn!("No collections available");
            } else if let Some(entity) = model
                .active_collection
                .remote_view()
                .find_entity_by_title(collection_title)
            {
                log::info!(
                    "Activating collection '{}' ({})",
                    entity.body.title,
                    entity.hdr.uid,
                );
                let entity_uid = Some(entity.hdr.uid.clone());
                *collection_uid = entity_uid.clone();
                let intent = collection::Intent::ActivateEntity { entity_uid };
                return Err(Some(intent.into()));
            } else {
                log::warn!("No collection with title '{collection_title}' found");
            }
        }
    } else if !model
        .active_collection
        .remote_view()
        .filtered_entities
        .is_pending()
    {
        let filter_by_kind = None;
        let intent = collection::Intent::FetchFilteredEntities(collection::FetchFilteredEntities {
            filter_by_kind,
        });
        return Err(Some(intent.into()));
    }
    Err(None)
}
