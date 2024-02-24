// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide::desktop_app::collection;
use egui::Context;

use crate::{
    app::TrackSearchFetchedItems,
    fs::choose_directory_path,
    library::{self, Library, OnTrackSearchStateChangedCompletionError},
};

use super::{
    Action, CentralPanelData, CollectionAction, Event, LibraryAction, Message, MessageSender,
    Model, MusicDirSelection, MusicDirectoryAction, TrackListItem, TrackSearchAction,
};

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
                            mdl.library.state().music_dir.as_ref(),
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
                                last_progress: None,
                                final_outcome: None,
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
                    TrackSearchAction::AbortPendingStateChange => {
                        let (memo, memo_delta) =
                            match mdl.library.on_track_search_state_changed_complete_pending() {
                                Ok((memo, memo_delta)) => (memo, memo_delta),
                                Err(err) => {
                                    on_library_track_search_state_changed_completion_error(
                                        &mut mdl.library,
                                        err,
                                        msg_tx,
                                    );
                                    return;
                                }
                            };
                        log::debug!("Aborting track search memo change: {memo:?} {memo_delta:?}");
                        mdl.library.on_track_search_state_changed_abort();
                    }
                    TrackSearchAction::ApplyPendingStateChange { fetched_items } => {
                        let (memo, memo_delta) =
                            match mdl.library.on_track_search_state_changed_complete_pending() {
                                Ok((memo, memo_delta)) => (memo, memo_delta),
                                Err(err) => {
                                    on_library_track_search_state_changed_completion_error(
                                        &mut mdl.library,
                                        err,
                                        msg_tx,
                                    );
                                    return;
                                }
                            };
                        log::debug!("Applying track search memo change");
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
                                mdl.library.on_track_search_state_changed_abort();
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
                        let new_offset = match fetched_items {
                            TrackSearchFetchedItems::Reset => {
                                log::debug!(
                                    "Track search list changed: No fetched items available",
                                );
                                mdl.central_panel_data = None;
                                None
                            }
                            TrackSearchFetchedItems::Replace(fetched_items) => {
                                log::debug!(
                                    "Track search list changed: Replacing all {count_before} with {count_after} items",
                                    count_before = track_search_list.len(),
                                    count_after = fetched_items.len()
                                );
                                track_search_list.clear();
                                track_search_list.extend(fetched_items);
                                Some(track_search_list.len())
                            }
                            TrackSearchFetchedItems::Append(fetched_items) => {
                                let offset = track_search_list.len();
                                debug_assert_eq!(
                                    Some(offset),
                                    memo.fetch.fetched_entities.as_ref().map(|memo| memo.offset),
                                );
                                log::debug!(
                                            "Track search list changed: Appending {count_append} fetched items to {count_before} existing items",
                                            count_before = track_search_list.len(),
                                            count_append = fetched_items.len());
                                track_search_list.extend(fetched_items);
                                Some(track_search_list.len())
                            }
                        };
                        debug_assert_eq!(
                            new_offset,
                            memo_delta.fetch.as_ref().and_then(|fetch| fetch
                                .fetched_entities
                                .as_ref()
                                .map(|memo| memo.offset))
                        );
                        mdl.library.on_track_search_state_changed_apply();
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
                    match &mdl.library.state().collection {
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
                    on_library_track_search_state_changed(&mut mdl.library, ctx, rt, msg_tx);
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
                if let Some(CentralPanelData::MusicDirSync {
                    last_progress,
                    final_outcome,
                }) = &mut mdl.central_panel_data
                {
                    debug_assert!(final_outcome.is_none());
                    *last_progress = progress;
                } else {
                    log::debug!(
                        "Discarding unexpected music directory synchronization progress: {progress:?}"
                    );
                }
            }
            library::Event::MusicDirSyncOutcome(outcome) => {
                if let Some(CentralPanelData::MusicDirSync { final_outcome, .. }) =
                    &mut mdl.central_panel_data
                {
                    debug_assert!(final_outcome.is_none());
                    *final_outcome = outcome;
                } else {
                    log::debug!(
                        "Discarding unexpected music directory synchronization outcome: {outcome:?}"
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
                if Some(&collection_uid) != mdl.library.state().collection.entity_uid() {
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

fn on_library_track_search_state_changed(
    library: &mut Library,
    ctx: &Context,
    rt: &tokio::runtime::Handle,
    msg_tx: &MessageSender,
) {
    let Some((memo, memo_diff)) = library.on_track_search_state_changed() else {
        return;
    };
    match memo_diff {
        aoide::desktop_app::track::repo_search::MemoDiff::Unchanged => {
            log::debug!("Track search memo unchanged");
        }
        aoide::desktop_app::track::repo_search::MemoDiff::Changed {
            fetched_entities: fetched_entities_diff,
        } => {
            let offset = match fetched_entities_diff {
                aoide::desktop_app::track::repo_search::FetchedEntitiesDiff::Replace => 0,
                aoide::desktop_app::track::repo_search::FetchedEntitiesDiff::Append => memo
                    .fetch
                    .fetched_entities
                    .as_ref()
                    .map_or(0, |memo| memo.offset),
            };
            let ctx = ctx.clone();
            let msg_tx = msg_tx.clone();
            let subscriber = library.subscribe_track_search_state_changed();
            rt.spawn_blocking(move || {
                let mut discard = false;
                let state = subscriber.read();
                let fetched_items = state.fetched_entities().and_then(|fetched_entities| {
                    if offset > fetched_entities.len() {
                        // Race condition after the observable state has changed (again) in the meantime.
                        discard = true;
                        return None;
                    }
                    let fetched_items = fetched_entities[offset..]
                        .iter()
                        .map(|fetched_entity| {
                            TrackListItem::new(
                                &ctx,
                                fetched_entity.entity.hdr.uid.clone(),
                                &fetched_entity.entity.body.track,
                            )
                        })
                        .collect();
                    Some(fetched_items)
                });
                if discard {
                    log::debug!(
                        "Discarding inapplicable track search state and memo update: {memo_diff:?}"
                    );
                    msg_tx.send_action(TrackSearchAction::AbortPendingStateChange);
                    return;
                }
                let fetched_items = if let Some(fetched_items) = fetched_items {
                    match fetched_entities_diff {
                        aoide::desktop_app::track::repo_search::FetchedEntitiesDiff::Replace => {
                            TrackSearchFetchedItems::Replace(fetched_items)
                        }
                        aoide::desktop_app::track::repo_search::FetchedEntitiesDiff::Append => {
                            TrackSearchFetchedItems::Append(fetched_items)
                        }
                    }
                } else {
                    TrackSearchFetchedItems::Reset
                };
                msg_tx.send_action(TrackSearchAction::ApplyPendingStateChange { fetched_items });
            });
        }
    }
}

fn on_library_track_search_state_changed_completion_error(
    library: &mut Library,
    err: OnTrackSearchStateChangedCompletionError,
    msg_tx: &MessageSender,
) {
    match err {
        OnTrackSearchStateChangedCompletionError::NotPending => {
            // Nothing to do.
            log::debug!("Ignoring track search state change completion: Not pending");
        }
        OnTrackSearchStateChangedCompletionError::AbortPendingAndRetry => {
            log::debug!("Aborting track search state change completion and retrying");
            library.on_track_search_state_changed_abort();
            // Replay the corresponding event.
            msg_tx
                .emit_event(library::Event::from(
                    library::track_search::Event::StateChanged,
                ))
                .unwrap();
        }
    }
}
