// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide::desktop_app::collection;
use egui::Context;

use crate::{fs::choose_directory_path, library};

use super::{
    Action, CentralPanelData, CollectionAction, Event, LibraryAction, Message, MessageSender,
    Model, MusicDirSelection, MusicDirectoryAction, TrackListItem, TrackSearchAction,
};

const MUSIC_DIR_SYNC_PROGRESS_LOG_MAX_LINES: usize = 100;

pub(super) struct UpdateContext<'a> {
    pub(super) rt: &'a tokio::runtime::Handle,
    pub(super) msg_tx: &'a MessageSender,
    pub(super) mdl: &'a mut Model,
}

impl<'a> UpdateContext<'a> {
    pub(super) fn on_message(&mut self, ctx: &Context, msg: Message) {
        match msg {
            Message::Action(action) => self.on_action(action),
            Message::Event(event) => self.on_event(ctx, event),
        }
    }

    #[allow(clippy::too_many_lines)] // TODO
    fn on_action(&mut self, action: Action) {
        let Self { rt, msg_tx, mdl } = self;
        match action {
            Action::Library(action) => match action {
                LibraryAction::MusicDirectory(action) => match action {
                    MusicDirectoryAction::Reset => {
                        mdl.library.try_reset_music_dir();
                    }
                    MusicDirectoryAction::Select => {
                        if matches!(mdl.music_dir_selection, Some(MusicDirSelection::Selecting)) {
                            log::debug!("Already selecting music directory");
                            return;
                        }
                        let on_dir_path_chosen = {
                            let msg_tx = msg_tx.clone();
                            move |dir_path| {
                                msg_tx.send_action(MusicDirectoryAction::Update(dir_path));
                            }
                        };
                        choose_directory_path(
                            rt,
                            mdl.library.state().last_observed_music_dir.as_ref(),
                            on_dir_path_chosen,
                        );
                        mdl.music_dir_selection = Some(MusicDirSelection::Selecting);
                    }
                    MusicDirectoryAction::Update(music_dir) => {
                        mdl.music_dir_selection = Some(MusicDirSelection::Selected);
                        if let Some(music_dir) = music_dir {
                            mdl.library.try_update_music_dir(Some(&music_dir));
                        }
                    }
                    MusicDirectoryAction::SpawnSyncTask => {
                        if mdl.library.try_spawn_music_dir_sync_task(rt, *msg_tx) {
                            log::debug!("Switching to music dir sync progress view");
                            mdl.central_panel_data = Some(CentralPanelData::MusicDirSync {
                                progress_log: vec![],
                            });
                        }
                    }
                    MusicDirectoryAction::AbortPendingSyncTask => {
                        mdl.library.try_abort_pending_music_dir_sync_task();
                    }
                    MusicDirectoryAction::ViewList => {
                        let params = aoide::api::media::tracker::count_sources_in_directories::Params {
                            ordering: Some(
                                aoide::api::media::tracker::count_sources_in_directories::Ordering::CountDescending,
                            ),
                            ..Default::default()
                        };
                        if mdl.library.try_view_music_dir_list(rt, *msg_tx, params) {
                            log::debug!("Switching to music dir list view");
                            mdl.central_panel_data = Some(CentralPanelData::MusicDirList {
                                content_paths_with_count: vec![],
                            });
                        }
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
                    TrackSearchAction::UpdateStateAndList {
                        memo,
                        memo_delta,
                        fetched_entities_diff,
                        fetched_items,
                    } => {
                        if !mdl.library.on_track_search_state_changed_part2(&memo) {
                            // Not applicable.
                            log::debug!("Discarding track search memo change: {memo:?} {memo_delta:?} {fetched_entities_diff:?}");
                            return;
                        }
                        log::debug!("Finalizing track search memo change: {memo:?} {memo_delta:?} {fetched_entities_diff:?}");
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
                                &mut mdl.central_panel_data
                            else {
                                unreachable!()
                            };
                            track_list
                        };
                        match fetched_entities_diff {
                            aoide::desktop_app::track::repo_search::FetchedEntitiesDiff::Replace => {
                                if let Some(fetched_items) = fetched_items {
                                    log::debug!(
                                        "Track search list changed: Replacing all {count_before} with {count_after} items",
                                        count_before = track_search_list.len(),
                                        count_after = fetched_items.len()
                                    );
                                    track_search_list.clear();
                                    track_search_list.extend(fetched_items);
                                } else {
                                    log::debug!(
                                        "Track search list changed: No fetched items available",
                                    );
                                    mdl.central_panel_data = None;
                                }
                            }
                            aoide::desktop_app::track::repo_search::FetchedEntitiesDiff::Append => {
                                let Some(fetched_items) = fetched_items else {
                                    unreachable!();
                                };
                                let offset = track_search_list.len();
                                debug_assert_eq!(
                                    Some(offset),
                                    memo.fetch
                                        .fetched_entities
                                        .as_ref()
                                        .map(|memo| memo.offset),
                                );
                                log::debug!(
                                            "Track search list changed: Appending {count_append} fetched items to {count_before} existing items",
                                            count_before = track_search_list.len(),
                                            count_append = fetched_items.len());
                                track_search_list.extend(fetched_items);
                            }
                        };
                        mdl.library
                            .on_track_search_state_changed_part3(&memo, memo_delta);
                    }
                },
            },
        }
    }

    fn on_event(&mut self, ctx: &Context, event: Event) {
        match event {
            Event::Library(event) => self.on_library_event(ctx, event),
        }
    }

    #[allow(clippy::too_many_lines)] // TODO
    fn on_library_event(&mut self, ctx: &Context, event: library::Event) {
        let Self {
            rt, msg_tx, mdl, ..
        } = self;
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
                        collection::State::Ready { summary, .. } => {
                            if matches!(mdl.music_dir_selection, Some(MusicDirSelection::Selected))
                            {
                                mdl.music_dir_selection = None;
                                if summary.media_sources.total_count == 0 {
                                    log::info!("Synchronizing music directory after empty collection has been selected");
                                    msg_tx.send_action(MusicDirectoryAction::SpawnSyncTask);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            library::Event::TrackSearch(event) => match event {
                library::track_search::Event::StateChanged => {
                    let (memo, memo_delta, memo_diff) =
                        mdl.library.on_track_search_state_changed_part1();
                    match memo_diff {
                        aoide::desktop_app::track::repo_search::MemoDiff::Unchanged => {
                            log::debug!("Track search memo unchanged");
                        }
                        aoide::desktop_app::track::repo_search::MemoDiff::Changed {
                            fetched_entities: fetched_entities_diff,
                        } => {
                            let memo = memo.clone();
                            let ctx = ctx.clone();
                            let msg_tx = msg_tx.clone();
                            let subscriber = mdl.library.subscribe_track_search_state_changed();
                            rt.spawn_blocking(move || {
                                let offset = match fetched_entities_diff {
                                    aoide::desktop_app::track::repo_search::FetchedEntitiesDiff::Replace => 0,
                                    aoide::desktop_app::track::repo_search::FetchedEntitiesDiff::Append => {
                                        memo
                                            .fetch
                                            .fetched_entities
                                            .as_ref()
                                            .map_or(0, |memo| memo.offset)
                                    }
                                };
                                let mut discard = false;
                                let state = subscriber.read();
                                let fetched_items = state.fetched_entities().and_then(|fetched_entities| {
                                    if offset > fetched_entities.len() {
                                        discard = true;
                                        return None;
                                    }
                                    let fetched_items = fetched_entities[offset..].iter().map(
                                        |fetched_entity| {
                                            TrackListItem::new(&ctx, fetched_entity.entity.hdr.uid.clone(), &fetched_entity.entity.body.track)
                                        },
                                    ).collect();
                                    Some(fetched_items)
                                });
                                if discard {
                                    log::debug!("Discarding inapplicable track search state and memo update: {memo:?} {memo_delta:?} {memo_diff:?}");
                                    return;
                                }
                                msg_tx.send_action(TrackSearchAction::UpdateStateAndList {
                                    memo,
                                    memo_delta,
                                    fetched_entities_diff,
                                    fetched_items,
                                });
                            });
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
            library::Event::MusicDirListResult {
                collection_uid,
                params: _,
                result,
            } => {
                let new_content_paths_with_count = match result {
                    Ok(content_paths_with_count) => content_paths_with_count,
                    Err(err) => {
                        log::warn!("Failed to view music directory list: {err}");
                        // TODO: Set last error.
                        return;
                    }
                };
                if Some(&collection_uid)
                    != mdl.library.state().last_observed_collection.entity_uid()
                {
                    log::debug!(
                        "Discarding unexpected music directory list with {num_items} item(s) for collection {collection_uid}",
                        num_items = new_content_paths_with_count.len()
                    );
                    return;
                }
                if let Some(CentralPanelData::MusicDirList {
                    content_paths_with_count,
                }) = &mut mdl.central_panel_data
                {
                    *content_paths_with_count = new_content_paths_with_count;
                } else {
                    log::debug!(
                        "Discarding unexpected music directory list with {num_items} item(s)",
                        num_items = new_content_paths_with_count.len()
                    );
                }
            }
        }
    }
}
