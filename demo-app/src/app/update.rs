// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide::desktop_app::collection;

use crate::{fs::choose_directory_path, library};

use super::{
    Action, CentralPanelData, CollectionAction, Event, LibraryAction, Message, MessageSender,
    Model, MusicDirectoryAction, TrackSearchAction,
};

const MUSIC_DIR_SYNC_PROGRESS_LOG_MAX_LINES: usize = 100;

pub(super) struct UpdateContext<'a> {
    pub(super) rt: &'a tokio::runtime::Handle,
    pub(super) msg_tx: &'a MessageSender,
    pub(super) mdl: &'a mut Model,
}

impl<'a> UpdateContext<'a> {
    pub(super) fn on_message(&mut self, msg: Message) {
        match msg {
            Message::Action(action) => self.on_action(action),
            Message::Event(event) => self.on_event(event),
        }
    }

    fn on_action(&mut self, action: Action) {
        let Self { rt, msg_tx, mdl } = self;
        match action {
            Action::Library(action) => match action {
                LibraryAction::MusicDirectory(action) => match action {
                    MusicDirectoryAction::Reset => {
                        mdl.library.try_reset_music_dir();
                    }
                    MusicDirectoryAction::Select => {
                        if mdl.selecting_music_dir {
                            log::debug!("Already selecting music directory");
                            return;
                        }
                        let on_dir_path_chosen = {
                            let msg_tx = msg_tx.clone();
                            move |dir_path| {
                                msg_tx.send_action(MusicDirectoryAction::Selected(dir_path));
                            }
                        };
                        choose_directory_path(
                            rt,
                            mdl.library.state().last_observed_music_dir.as_ref(),
                            on_dir_path_chosen,
                        );
                        mdl.selecting_music_dir = true;
                    }
                    MusicDirectoryAction::Selected(music_dir) => {
                        mdl.selecting_music_dir = false;
                        if let Some(music_dir) = music_dir {
                            mdl.library.try_update_music_dir(Some(&music_dir));
                        }
                    }
                    MusicDirectoryAction::SpawnSyncTask => {
                        if mdl.library.try_spawn_music_dir_sync_task(rt, *msg_tx) {
                            // Switch to synchronization progress view.
                            log::debug!("Switching to music dir sync progress view");
                            mdl.central_panel_data = Some(CentralPanelData::MusicDirSync {
                                progress_log: vec![],
                            });
                        }
                    }
                    MusicDirectoryAction::AbortPendingSyncTask => {
                        mdl.library.try_abort_pending_music_dir_sync_task();
                    }
                },
                LibraryAction::Collection(action) => match action {
                    CollectionAction::RefreshFromDb => {
                        mdl.library.try_refresh_collection_from_db(rt);
                    }
                },
                LibraryAction::TrackSearch(action) => match action {
                    TrackSearchAction::Search(input) => {
                        mdl.library.try_search_tracks(&input);
                    }
                    TrackSearchAction::FetchMore => {
                        mdl.library
                            .try_spawn_fetch_more_track_search_results_task(rt, *msg_tx);
                    }
                },
            },
        }
    }

    fn on_event(&mut self, event: Event) {
        match event {
            Event::Library(event) => self.on_library_event(event),
        }
    }

    #[allow(clippy::too_many_lines)] // TODO
    fn on_library_event(&mut self, event: library::Event) {
        let Self { mdl, .. } = self;
        match event {
            library::Event::Settings(library::settings::Event::StateChanged) => {
                mdl.library.on_settings_state_changed();
            }
            library::Event::Collection(library::collection::Event::StateChanged) => {
                if mdl.library.on_collection_state_changed() {
                    // Determine a follow-up effect or action dependent on the new state.
                    // TODO: Store or report outcomes and errors from these dead end states.
                    match &mdl.library.state().last_observed_collection {
                        collection::State::Void => {
                            // Nothing to show with no collection available. This prevents to
                            // show stale data after the collection has been reset.
                            if mdl.central_panel_data.is_some() {
                                log::debug!("Resetting central panel view");
                                mdl.central_panel_data = None;
                            }
                        }
                        collection::State::LoadingFailed { .. }
                        | collection::State::RestoringOrCreatingFromMusicDirectoryFailed {
                            ..
                        }
                        | collection::State::NestedMusicDirectoriesConflict { .. } => {
                            mdl.library.try_reset_music_dir();
                        }
                        _ => {}
                    }
                }
            }
            library::Event::TrackSearch(event) => match event {
                library::track_search::Event::StateChanged => {
                    let last_memo_offset = mdl
                        .library
                        .state()
                        .last_observed_track_search_memo
                        .fetch
                        .fetched_entities
                        .as_ref()
                        .map(|memo| memo.offset);
                    let memo_updated = mdl.library.on_track_search_state_changed();
                    match memo_updated {
                        aoide::desktop_app::track::repo_search::MemoUpdated::Unchanged => {
                            log::debug!("Track search memo unchanged");
                        }
                        aoide::desktop_app::track::repo_search::MemoUpdated::Changed {
                            fetched_entities_diff,
                        } => {
                            let track_search_list = if let Some(CentralPanelData::TrackSearch {
                                track_list,
                            }) = &mut mdl.central_panel_data
                            {
                                track_list
                            } else {
                                if matches!(
                                    mdl.central_panel_data,
                                    Some(CentralPanelData::MusicDirSync { .. })
                                ) && mdl.library.state().pending_music_dir_sync_task.is_some()
                                {
                                    log::debug!("Ignoring track search memo change: Music directory synchronization in progress");
                                    return;
                                }
                                log::debug!("Switching to track search view");
                                mdl.central_panel_data = Some(CentralPanelData::TrackSearch {
                                    track_list: Default::default(),
                                });
                                let Some(CentralPanelData::TrackSearch { track_list }) =
                                    mdl.central_panel_data.as_mut()
                                else {
                                    unreachable!()
                                };
                                track_list
                            };
                            let state = mdl.library.read_lock_track_search_state();
                            match fetched_entities_diff {
                                    aoide::desktop_app::track::repo_search::FetchedEntitiesDiff::Replace => {
                                        if let Some(fetched_entities) = state.fetched_entities() {
                                            log::debug!(
                                                "Track search memo changed: Replacing all {count_before} with {count_after} fetched entities",
                                                count_before = track_search_list.len(),
                                                count_after = fetched_entities.len()
                                            );
                                            track_search_list.clear();
                                            track_search_list.extend(fetched_entities.iter().map(
                                                |fetched_entity| {
                                                    track_to_string(&fetched_entity.entity.body.track)
                                                },
                                            ));
                                        } else {
                                            log::debug!(
                                                "Track search memo changed: No fetched entities available",
                                            );
                                            mdl.central_panel_data = None;
                                        }
                                    }
                                    aoide::desktop_app::track::repo_search::FetchedEntitiesDiff::Append => {
                                        let Some(fetched_entities) = state.fetched_entities() else {
                                            unreachable!();
                                        };
                                        let offset = track_search_list.len();
                                        debug_assert_eq!(
                                            Some(offset),
                                            last_memo_offset,
                                        );
                                        debug_assert!(offset <= fetched_entities.len());
                                        log::debug!(
                                                    "Track search memo changed: Appending {count} fetched entities",
                                                    count = fetched_entities.len() - offset);
                                        track_search_list.extend(fetched_entities[offset..].iter().map(
                                            |fetched_entity| {
                                                track_to_string(&fetched_entity.entity.body.track)
                                            },
                                        ));
                                    }
                                }
                        }
                    }
                }
                library::track_search::Event::FetchMoreTaskCompleted {
                    result,
                    continuation,
                } => {
                    mdl.library
                        .on_fetch_more_track_search_results_task_completed(result, continuation);
                }
            },
            library::Event::MusicDirSyncProgress(progress) => {
                if let Some(CentralPanelData::MusicDirSync { progress_log }) =
                    &mut mdl.central_panel_data
                {
                    if progress_log.len() >= MUSIC_DIR_SYNC_PROGRESS_LOG_MAX_LINES {
                        // Shrink the log to avoid excessive memory usage.
                        progress_log.drain(..progress_log.len() / 2);
                    }
                    if let Some(progress) = progress {
                        progress_log.push(format!("{progress:?}"));
                    }
                } else {
                    log::debug!(
                        "Discarding unexpected music directory synchronization progress: {progress:?}"
                    );
                }
            }
        }
    }
}

fn track_to_string(track: &aoide::Track) -> String {
    let track_artist = track.track_artist();
    let track_title = track.track_title().unwrap_or("Untitled");
    let album_title = track.album_title();
    let album_artist = track.album_artist();
    match (track_artist, album_title, album_artist) {
        (Some(track_artist), Some(album_title), Some(album_artist)) => {
            if track_artist == album_artist {
                format!("{track_artist} - {track_title} [{album_title}]")
            } else {
                format!("{track_artist} - {track_title} [{album_title} by {album_artist}]")
            }
        }
        (None, Some(album_title), Some(album_artist)) => {
            format!("{track_title} [{album_title} by {album_artist}]")
        }
        (Some(track_artist), Some(album_title), None) => {
            format!("{track_artist} - {track_title} [{album_title}]")
        }
        (Some(track_artist), None, _) => {
            format!("{track_artist} - {track_title}")
        }
        (None, Some(album_title), None) => {
            format!("{track_title} [{album_title}]")
        }
        (None, None, _) => track_title.to_string(),
    }
}
