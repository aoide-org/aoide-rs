// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use egui::Context;

use crate::{
    app::TrackSearchFetchedItems,
    fs::choose_directory_path,
    library::{self, collection, track_search, Library},
};

use super::{
    Action, CollectionAction, Event, LibraryAction, Message, MessageSender, Model, ModelMode,
    MusicDirSelection, MusicDirectoryAction, TrackListItem, TrackSearchAction, TrackSearchMode,
};

pub(super) struct UpdateContext<'a> {
    pub(super) rt: &'a tokio::runtime::Handle,
    pub(super) msg_tx: &'a MessageSender,
    pub(super) mdl: &'a mut Model,
}

impl<'a> UpdateContext<'a> {
    pub(super) fn on_message(&mut self, ctx: &Context, msg: Message) {
        match msg {
            Message::Action(action) => self.on_action(ctx, action),
            Message::Event(event) => self.on_event(ctx, event),
        }
    }

    fn on_action(&mut self, ctx: &Context, action: Action) {
        let Self { rt, msg_tx, mdl } = self;
        let Model { library, mode, .. } = mdl;
        match action {
            Action::Library(action) => match action {
                LibraryAction::MusicDirectory(action) => {
                    on_library_music_directory_action(ctx, mdl, rt, msg_tx, action);
                }
                LibraryAction::Collection(action) => match action {
                    CollectionAction::RefreshFromDb => {
                        if !library.try_refresh_collection_from_db(rt) {
                            return;
                        }
                        *mode = None;
                        ctx.request_repaint();
                    }
                },
                LibraryAction::TrackSearch(action) => {
                    let Self { rt, msg_tx, mdl } = self;
                    let Model { library, mode, .. } = mdl;
                    let mode = mode.get_or_insert_with(|| {
                        ctx.request_repaint();
                        ModelMode::TrackSearch(Default::default())
                    });
                    let ModelMode::TrackSearch(track_search) = mode else {
                        log::debug!("Rejecting track search action (invalid mode): {action:?}");
                        return;
                    };
                    on_library_track_search_action(ctx, rt, msg_tx, library, track_search, action);
                }
            },
        }
    }

    fn on_event(&mut self, ctx: &Context, event: Event) {
        match event {
            Event::Library(event) => {
                let Self {
                    rt, msg_tx, mdl, ..
                } = self;
                on_library_event(ctx, mdl, rt, msg_tx, event);
            }
        }
    }
}

fn on_library_music_directory_action(
    ctx: &Context,
    mdl: &mut Model,
    rt: &tokio::runtime::Handle,
    msg_tx: &MessageSender,
    action: MusicDirectoryAction,
) {
    let Model {
        library,
        mode,
        music_dir_selection,
    } = mdl;
    match action {
        MusicDirectoryAction::Reset => {
            library.try_reset_music_dir();
        }
        MusicDirectoryAction::Select => {
            if matches!(music_dir_selection, Some(MusicDirSelection::Selecting)) {
                log::debug!("Already selecting music directory");
                return;
            }
            let on_dir_path_chosen = {
                let msg_tx = msg_tx.clone();
                move |dir_path| {
                    msg_tx.send_action(MusicDirectoryAction::Update(dir_path));
                }
            };
            choose_directory_path(rt, library.state().music_dir.as_ref(), on_dir_path_chosen);
            *music_dir_selection = Some(MusicDirSelection::Selecting);
        }
        MusicDirectoryAction::Update(music_dir) => {
            *music_dir_selection = Some(MusicDirSelection::Selected);
            if let Some(music_dir) = music_dir {
                library.try_update_music_dir(Some(&music_dir));
            }
        }
        MusicDirectoryAction::SpawnSyncTask => {
            if library.try_spawn_music_dir_sync_task(rt, msg_tx) {
                log::debug!("Switching to music dir sync progress view");
                *mode = Some(ModelMode::MusicDirSync {
                    last_progress: None,
                    final_outcome: None,
                });
                ctx.request_repaint();
            }
        }
        MusicDirectoryAction::AbortPendingSyncTask => {
            library.try_abort_pending_music_dir_sync_task();
        }
        MusicDirectoryAction::FinishSync => {
            if let Some(ModelMode::MusicDirSync { .. }) = mode {
                *mode = None;
                msg_tx.send_action(CollectionAction::RefreshFromDb);
            }
        }
        MusicDirectoryAction::ViewList => {
            let params = aoide::api::media::tracker::count_sources_in_directories::Params {
                ordering: Some(
                    aoide::api::media::tracker::count_sources_in_directories::Ordering::CountDescending,
                ),
                ..Default::default()
            };
            if library.try_view_music_dir_list(rt, msg_tx, params) {
                log::debug!("Switching to music dir list view");
                *mode = Some(ModelMode::MusicDirList {
                    content_paths_with_count: vec![],
                });
                ctx.request_repaint();
            }
        }
        MusicDirectoryAction::FinishViewList => {
            if let Some(ModelMode::MusicDirList { .. }) = mode {
                *mode = None;
                msg_tx.send_action(CollectionAction::RefreshFromDb);
            }
        }
    }
}

fn on_library_event(
    ctx: &Context,
    mdl: &mut Model,
    rt: &tokio::runtime::Handle,
    msg_tx: &MessageSender,
    event: library::Event,
) {
    let Model { library, mode, .. } = mdl;
    match event {
        library::Event::Settings(library::settings::Event::StateChanged) => {
            library.on_settings_state_changed();
        }
        library::Event::Collection(library::collection::Event::StateChanged) => {
            on_library_collection_state_changed(ctx, mdl, msg_tx);
        }
        library::Event::TrackSearch(event) => {
            let mode = mode.get_or_insert_with(|| {
                ctx.request_repaint();
                ModelMode::TrackSearch(Default::default())
            });
            let ModelMode::TrackSearch(TrackSearchMode { memo_state, .. }) = mode else {
                log::debug!("Ignoring track search event (invalid mode): {event:?}");
                return;
            };
            match event {
                library::track_search::Event::StateChanged => {
                    if on_library_track_search_state_changed(ctx, library, memo_state, rt, msg_tx) {
                        // `memo_state` in `mode` has changed.
                        ctx.request_repaint();
                    }
                }
                library::track_search::Event::FetchMoreTaskCompleted {
                    result,
                    continuation,
                } => {
                    library.on_fetch_more_track_search_results_task_completed(result, continuation);
                }
            }
        }
        library::Event::MusicDirSyncProgress(progress) => {
            if let Some(ModelMode::MusicDirSync {
                last_progress,
                final_outcome,
            }) = mode
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
            if let Some(ModelMode::MusicDirSync { final_outcome, .. }) = mode {
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
            if Some(&collection_uid) != library.state().collection.entity_uid() {
                log::debug!(
                    "Discarding unexpected music directory list with {num_items} item(s) for collection {collection_uid}",
                    num_items = new_content_paths_with_count.len()
                );
                return;
            }
            if let Some(ModelMode::MusicDirList {
                content_paths_with_count,
            }) = mode
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

fn on_library_track_search_action(
    ctx: &Context,
    rt: &tokio::runtime::Handle,
    msg_tx: &MessageSender,
    library: &mut Library,
    track_search: &mut TrackSearchMode,
    action: TrackSearchAction,
) {
    let TrackSearchMode {
        track_list,
        memo_state,
    } = track_search;
    match action {
        TrackSearchAction::Search(input) => {
            memo_state.abort();
            debug_assert!(matches!(memo_state, track_search::MemoState::Ready(_)));
            library.try_search_tracks(&input);
        }
        TrackSearchAction::FetchMore => {
            memo_state.abort();
            debug_assert!(matches!(memo_state, track_search::MemoState::Ready(_)));
            library.try_spawn_fetch_more_track_search_results_task(rt, msg_tx);
        }
        TrackSearchAction::AbortPendingStateChange => {
            if !matches!(memo_state, track_search::MemoState::Pending { .. }) {
                log::debug!("No track search state change pending");
                return;
            }
            let (memo, memo_delta) = match memo_state.complete() {
                Ok((memo, memo_delta)) => (memo, memo_delta),
                Err(err) => {
                    on_library_track_search_state_changed_pending_abort_with_completion_error(
                        memo_state, err, msg_tx,
                    );
                    debug_assert!(matches!(memo_state, track_search::MemoState::Ready(_)));
                    return;
                }
            };
            log::debug!("Aborting track search memo change: {memo:?} {memo_delta:?}");
            memo_state.abort();
            debug_assert!(matches!(memo_state, track_search::MemoState::Ready(_)));
        }
        TrackSearchAction::ApplyPendingStateChange { fetched_items } => {
            if !matches!(memo_state, track_search::MemoState::Pending { .. }) {
                log::debug!("No track search state change pending");
                return;
            }
            let (memo, memo_delta) = match memo_state.complete() {
                Ok((memo, memo_delta)) => (memo, memo_delta),
                Err(err) => {
                    on_library_track_search_state_changed_pending_abort_with_completion_error(
                        memo_state, err, msg_tx,
                    );
                    debug_assert!(matches!(memo_state, track_search::MemoState::Ready(_)));
                    return;
                }
            };
            log::debug!("Applying track search memo change");
            let new_offset = match fetched_items {
                TrackSearchFetchedItems::Reset => {
                    log::debug!("Track search list changed: No fetched items available");
                    *track_list = None;
                    None
                }
                TrackSearchFetchedItems::Replace(fetched_items) => {
                    let track_list =
                        track_list.get_or_insert(Vec::with_capacity(fetched_items.len()));
                    log::debug!(
                        "Track search list changed: Replacing all {count_before} with {count_after} items",
                        count_before = track_list.len(),
                        count_after = fetched_items.len()
                    );
                    track_list.clear();
                    track_list.extend(fetched_items);
                    Some(track_list.len())
                }
                TrackSearchFetchedItems::Append(fetched_items) => {
                    let track_list =
                        track_list.get_or_insert(Vec::with_capacity(fetched_items.len()));
                    let offset = track_list.len();
                    debug_assert_eq!(
                        Some(offset),
                        memo.fetch.fetched_entities.as_ref().map(|memo| memo.offset),
                    );
                    log::debug!(
                                "Track search list changed: Appending {count_append} fetched items to {count_before} existing items",
                                count_before = track_list.len(),
                                count_append = fetched_items.len());
                    track_list.extend(fetched_items);
                    Some(track_list.len())
                }
            };
            debug_assert_eq!(
                new_offset,
                memo_delta
                    .fetch
                    .as_ref()
                    .and_then(|fetch| fetch.fetched_entities.as_ref().map(|memo| memo.offset))
            );
            library.on_track_search_state_changed_pending_apply(memo_state);
            debug_assert!(matches!(memo_state, track_search::MemoState::Ready(_)));
            ctx.request_repaint();
        }
    }
}

fn on_library_collection_state_changed(ctx: &Context, mdl: &mut Model, msg_tx: &MessageSender) {
    let Model {
        library,
        music_dir_selection,
        mode,
    } = mdl;
    if !library.on_collection_state_changed() {
        return;
    }
    // Update the UI to reflect the new state.
    ctx.request_repaint();
    // Determine a follow-up effect or action dependent on the new state.
    // TODO: Store or report outcomes and errors from these dead end states.
    match &library.state().collection {
        collection::State::Void => {
            // Nothing to show with no collection available. This prevents to
            // show stale data after the collection has been reset.
            if mode.is_some() {
                log::debug!("Resetting central panel view");
                *mode = None;
            }
        }
        collection::State::LoadingFailed { .. }
        | collection::State::RestoringOrCreatingFromMusicDirectoryFailed { .. }
        | collection::State::NestedMusicDirectoriesConflict { .. } => {
            library.try_reset_music_dir();
        }
        collection::State::Ready { summary, .. } => {
            if matches!(music_dir_selection, Some(MusicDirSelection::Selected)) {
                *music_dir_selection = None;
                if summary.media_sources.total_count == 0 {
                    log::info!(
                        "Synchronizing music directory after empty collection has been selected"
                    );
                    msg_tx.send_action(MusicDirectoryAction::SpawnSyncTask);
                }
            }
        }
        _ => {}
    }
    // Reset mode if the collection is not synchronizing anymore.
    if matches!(mode, Some(ModelMode::MusicDirSync { .. }))
        && !library.state().collection.is_synchronizing()
    {
        *mode = None;
    }
}

fn on_library_track_search_state_changed(
    ctx: &Context,
    library: &Library,
    memo_state: &mut track_search::MemoState,
    rt: &tokio::runtime::Handle,
    msg_tx: &MessageSender,
) -> bool {
    let Some((memo, memo_diff)) = library.on_track_search_state_changed(memo_state) else {
        log::debug!("Ignoring track search state change");
        return false;
    };
    let fetched_entities_diff = match memo_diff {
        aoide::desktop_app::track::repo_search::MemoDiff::Unchanged => {
            log::debug!("Track search memo unchanged");
            return false;
        }
        aoide::desktop_app::track::repo_search::MemoDiff::Changed {
            fetched_entities: fetched_entities_diff,
        } => fetched_entities_diff,
    };
    log::debug!("Track search memo changed: {memo:?} {memo_diff:?}");
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
        collect_and_forward_fetched_track_list_items(
            &ctx,
            &msg_tx,
            &subscriber,
            &memo_diff,
            fetched_entities_diff,
            offset,
        );
    });
    true
}

fn collect_and_forward_fetched_track_list_items(
    ctx: &Context,
    msg_tx: &MessageSender,
    subscriber: &track_search::StateSubscriber,
    memo_diff: &track_search::MemoDiff,
    fetched_entities_diff: track_search::FetchedEntitiesDiff,
    offset: usize,
) {
    let mut discard = false;
    let fetched_items = {
        let state = subscriber.read();
        state.fetched_entities().and_then(|fetched_entities| {
            if offset > fetched_entities.len() {
                // Race condition after the observable state has changed (again) in the meantime.
                discard = true;
                return None;
            }
            let fetched_items = fetched_entities[offset..]
                .iter()
                .map(|fetched_entity| {
                    TrackListItem::new(
                        ctx,
                        fetched_entity.entity.hdr.uid.clone(),
                        &fetched_entity.entity.body.track,
                    )
                })
                .collect();
            Some(fetched_items)
        })
    };
    if discard {
        // Should happen only rarely.
        log::warn!("Discarding inapplicable track search state update: {memo_diff:?}");
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
}

fn on_library_track_search_state_changed_pending_abort_with_completion_error(
    memo_state: &mut track_search::MemoState,
    err: track_search::MemoStateCompletionError,
    msg_tx: &MessageSender,
) {
    match err {
        track_search::MemoStateCompletionError::NotPending => {
            // Nothing to do.
            log::debug!("Ignoring track search state change completion: Not pending");
        }
        track_search::MemoStateCompletionError::AbortPendingAndRetry => {
            log::debug!("Aborting track search state change completion and retrying");
            memo_state.abort();
            // Replay the corresponding event.
            msg_tx
                .emit_event(library::Event::from(
                    library::track_search::Event::StateChanged,
                ))
                .unwrap();
        }
    }
}
