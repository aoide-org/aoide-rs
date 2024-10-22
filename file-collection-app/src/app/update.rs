// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide::desktop_app::{collection::SynchronizingVfsFinishedState, ActionEffect};
use egui::Context;

use crate::{
    app::TrackSearchFetchedItems,
    fs::choose_directory_path,
    library::{self, collection, track_search, ui::TrackListItem, Library},
};

use super::{
    message::{MediaTrackerAction, MediaTrackerDirListAction, MediaTrackerSyncAction},
    Action, CollectionAction, Event, LibraryAction, Message, MessageSender, Model, ModelMode,
    MusicDirSelection, MusicDirectoryAction, TrackSearchAction, TrackSearchMode,
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
        let action_effect = match action {
            Action::Library(action) => match action {
                LibraryAction::MusicDirectory(action) => {
                    self.on_library_music_directory_action(action)
                }
                LibraryAction::MediaTracker(action) => self.on_library_media_tracker_action(action),
                LibraryAction::Collection(action) => self.on_library_collection_action(action),
                LibraryAction::TrackSearch(action) => self.on_library_track_search_action(action),
            },
        };
        if matches!(action_effect, ActionEffect::Unchanged) {
            return;
        }
        ctx.request_repaint();
    }

    fn on_library_music_directory_action(&mut self, action: MusicDirectoryAction) -> ActionEffect {
        let Self { rt, msg_tx, mdl } = self;
        let Model {
            library,
            music_dir_selection,
            ..
        } = mdl;
        match action {
            MusicDirectoryAction::Reset => library.reset_music_dir(),
            MusicDirectoryAction::Select => {
                if matches!(music_dir_selection, Some(MusicDirSelection::Selecting)) {
                    log::debug!("Already selecting music directory");
                    return ActionEffect::Unchanged;
                }
                let on_dir_path_chosen = {
                    let msg_tx = msg_tx.clone();
                    move |dir_path| {
                        msg_tx.send_action(MusicDirectoryAction::Update(dir_path));
                    }
                };
                choose_directory_path(rt, library.state().music_dir.as_ref(), on_dir_path_chosen);
                *music_dir_selection = Some(MusicDirSelection::Selecting);
                ActionEffect::Changed
            }
            MusicDirectoryAction::Update(music_dir) => {
                *music_dir_selection = Some(MusicDirSelection::Selected);
                if let Some(music_dir) = music_dir {
                    library.update_music_dir(Some(&music_dir))
                } else {
                    // No effect.
                    ActionEffect::Unchanged
                }
            }
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn on_library_media_tracker_action(&mut self, action: MediaTrackerAction) -> ActionEffect {
        let Self { rt, msg_tx, mdl } = self;
        let Model { library, mode, .. } = mdl;
        match action {
            MediaTrackerAction::Sync(action) => match action {
                MediaTrackerSyncAction::SpawnTask => {
                    let (mut effect, result) = library.sync_music_dir(rt, *msg_tx);
                    if matches!(result, Ok(())) {
                        log::debug!("Switching to music dir sync progress view");
                        *mode = Some(ModelMode::MusicDirSync {
                            last_progress: None,
                            final_outcome: None,
                        });
                        effect += ActionEffect::MaybeChanged;
                    }
                    effect
                }
                MediaTrackerSyncAction::AbortPendingTask => library.sync_music_dir_abort(),
                MediaTrackerSyncAction::Finish => {
                    let Some(some_mode) = mode else {
                        return ActionEffect::Unchanged;
                    };
                    if !matches!(some_mode, ModelMode::MusicDirList { .. }) {
                        return ActionEffect::Unchanged;
                    }
                    *mode = None;
                    msg_tx.send_action(CollectionAction::RefreshFromDb);
                    return ActionEffect::Changed;
                }
            },
            MediaTrackerAction::DirList(action) => match action {
                MediaTrackerDirListAction::OpenView => {
                    let params = aoide::api::media::tracker::count_sources_in_directories::Params {
                        ordering: Some(
                            aoide::api::media::tracker::count_sources_in_directories::Ordering::CountDescending,
                        ),
                        ..Default::default()
                    };
                    let mut effect = library.view_music_dir_list(rt, *msg_tx, params);
                    if matches!(effect, ActionEffect::Unchanged) {
                        return effect;
                    }
                    log::debug!("Switching to music dir list view");
                    *mode = Some(ModelMode::MusicDirList {
                        content_paths_with_count: vec![],
                    });
                    effect += ActionEffect::MaybeChanged;
                    effect
                }
                MediaTrackerDirListAction::CloseView => {
                    let Some(some_mode) = mode else {
                        return ActionEffect::Unchanged;
                    };
                    if !matches!(some_mode, ModelMode::MusicDirList { .. }) {
                        return ActionEffect::Unchanged;
                    }
                    *mode = None;
                    msg_tx.send_action(CollectionAction::RefreshFromDb);
                    ActionEffect::Changed
                }
            },
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn on_library_collection_action(&mut self, action: CollectionAction) -> ActionEffect {
        let Self { rt, mdl, .. } = self;
        let Model { library, mode, .. } = mdl;
        match action {
            CollectionAction::RefreshFromDb => {
                let (mut effect, abort_handle) = library.refresh_collection_from_db(rt);
                if abort_handle.is_some() && mode.is_some() {
                    *mode = None;
                    effect += ActionEffect::Changed;
                }
                effect
            }
        }
    }

    #[allow(clippy::too_many_lines)] // TODO
    fn on_library_track_search_action(&mut self, action: TrackSearchAction) -> ActionEffect {
        let Self { rt, msg_tx, mdl } = self;
        let Model { library, mode, .. } = mdl;
        let mut mode_effect = ActionEffect::Unchanged;
        let mode = mode.get_or_insert_with(|| {
            mode_effect += ActionEffect::MaybeChanged;
            ModelMode::TrackSearch(Default::default())
        });
        let ModelMode::TrackSearch(track_search) = mode else {
            log::info!("Discarding track search action: {action:?}");
            return mode_effect;
        };
        let TrackSearchMode {
            track_list,
            memo_state,
        } = track_search;
        let action_effect = match action {
            TrackSearchAction::Search(input) => {
                memo_state.abort();
                debug_assert!(matches!(memo_state, track_search::MemoState::Ready(_)));
                library.search_tracks(&input)
            }
            TrackSearchAction::FetchMore => {
                memo_state.abort();
                debug_assert!(matches!(memo_state, track_search::MemoState::Ready(_)));
                library.fetch_more_track_search_results(rt)
            }
            TrackSearchAction::AbortPendingStateChange => {
                if matches!(memo_state, track_search::MemoState::Pending { .. }) {
                    match memo_state.complete() {
                        Ok((memo, memo_delta)) => {
                            log::debug!(
                                "Aborting track search memo change: {memo:?} {memo_delta:?}"
                            );
                            memo_state.abort();
                            debug_assert!(matches!(memo_state, track_search::MemoState::Ready(_)));
                            ActionEffect::MaybeChanged
                        }
                        Err(err) => {
                            let response = on_library_track_search_state_changed_pending_abort_with_completion_error(
                                memo_state, err, msg_tx,
                            );
                            debug_assert!(matches!(memo_state, track_search::MemoState::Ready(_)));
                            response
                        }
                    }
                } else {
                    log::info!("No track search state change pending");
                    ActionEffect::Unchanged
                }
            }
            TrackSearchAction::ApplyPendingStateChange { fetched_items } => {
                if matches!(memo_state, track_search::MemoState::Pending { .. }) {
                    match memo_state.complete() {
                        Ok((memo, memo_delta)) => {
                            log::debug!("Applying track search memo change");
                            let new_offset = match fetched_items {
                                TrackSearchFetchedItems::Reset => {
                                    log::debug!(
                                        "Track search list changed: No fetched items available"
                                    );
                                    *track_list = None;
                                    None
                                }
                                TrackSearchFetchedItems::Replace(fetched_items) => {
                                    let track_list = track_list
                                        .get_or_insert(Vec::with_capacity(fetched_items.len()));
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
                                    let track_list = track_list
                                        .get_or_insert(Vec::with_capacity(fetched_items.len()));
                                    let offset = track_list.len();
                                    debug_assert_eq!(
                                        Some(offset),
                                        memo.fetch
                                            .fetched_entities
                                            .as_ref()
                                            .map(|memo| memo.offset),
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
                                memo_delta.fetch.as_ref().and_then(|fetch| fetch
                                    .fetched_entities
                                    .as_ref()
                                    .map(|memo| memo.offset))
                            );
                            library.on_track_search_state_changed_pending_apply(memo_state);
                            debug_assert!(matches!(memo_state, track_search::MemoState::Ready(_)));
                            ActionEffect::MaybeChanged
                        }
                        Err(err) => {
                            let response = on_library_track_search_state_changed_pending_abort_with_completion_error(
                                memo_state, err, msg_tx,
                            );
                            debug_assert!(matches!(memo_state, track_search::MemoState::Ready(_)));
                            response
                        }
                    }
                } else {
                    log::info!("No track search state change pending");
                    ActionEffect::Unchanged
                }
            }
        };
        mode_effect + action_effect
    }

    fn on_event(&mut self, ctx: &Context, event: Event) {
        match event {
            Event::Library(event) => {
                self.on_library_event(ctx, event);
            }
        }
    }

    fn on_library_event(&mut self, ctx: &Context, event: library::Event) {
        let Self { rt, msg_tx, mdl } = self;
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
                        if on_library_track_search_state_changed(
                            ctx, library, memo_state, rt, msg_tx,
                        ) {
                            // `memo_state` in `mode` has changed.
                            ctx.request_repaint();
                        }
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
            library::Event::MusicDirSyncFinished(finished_state) => {
                let outcome = match *finished_state {
                    SynchronizingVfsFinishedState::Succeeded { outcome } => Some(outcome),
                    SynchronizingVfsFinishedState::Aborted => None,
                    SynchronizingVfsFinishedState::Failed { error } => {
                        log::warn!("Synchronizing music directory failed: {error}");
                        None
                    }
                };
                if let Some(ModelMode::MusicDirSync { final_outcome, .. }) = mode {
                    debug_assert!(final_outcome.is_none());
                    *final_outcome = outcome.map(Box::new);
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
                if Some(&collection_uid) != library.read_collection_state().entity_uid() {
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
}

fn on_library_collection_state_changed(ctx: &Context, mdl: &mut Model, msg_tx: &MessageSender) {
    let Model {
        library,
        music_dir_selection,
        mode,
    } = mdl;
    // Update the UI unconditionally to reflect the new state.
    ctx.request_repaint();

    let mut reset_music_dir = false;
    {
        let collection_state = library.read_collection_state();
        // Determine a follow-up effect or action dependent on the new state.
        // TODO: Store or report outcomes and errors from these dead end states.
        match &*collection_state {
            collection::State::Void => {
                // Nothing to show with no collection available. This prevents to
                // show stale data after the collection has been reset.
                if mode.is_some() {
                    log::debug!("Resetting central panel view");
                    *mode = None;
                }
            }
            collection::State::LoadingFromDatabase {
                state: collection::LoadingFromDatabaseState::Finished(collection::LoadingFromDatabaseFinishedState::Failed { .. }),
                ..
            }
            | collection::State::RestoringFromMusicDirectory {
                state:
                    collection::RestoringFromMusicDirectoryState::Finished(collection::RestoringFromMusicDirectoryFinishedState::Failed { .. })
                    | collection::RestoringFromMusicDirectoryState::Finished(collection::RestoringFromMusicDirectoryFinishedState::NestedDirectoriesConflict { .. }),
                ..
            } => {
                reset_music_dir = true;
            }
            collection::State::Ready { summary, .. } => {
                if matches!(music_dir_selection, Some(MusicDirSelection::Selected)) {
                    *music_dir_selection = None;
                    if summary.media_sources.total_count == 0 {
                        log::info!(
                            "Synchronizing music directory after empty collection has been selected"
                        );
                        msg_tx.send_action(MediaTrackerSyncAction::SpawnTask);
                    }
                }
            }
            _ => (),
        }

        // Reset mode if the collection is not synchronizing anymore.
        if matches!(mode, Some(ModelMode::MusicDirSync { .. }))
            && !collection_state.is_synchronizing()
        {
            *mode = None;
        }
    }

    // Reset the music directory after releasing the read-lock.
    if reset_music_dir {
        // The UI will be repainted in any case (see above).
        let _ = library.reset_music_dir();
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
                    let entity_uid = fetched_entity.entity.hdr.uid.clone();
                    let content_url = fetched_entity.entity.body.content_url.clone();
                    TrackListItem::new(
                        ctx,
                        entity_uid,
                        content_url,
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
) -> ActionEffect {
    match err {
        track_search::MemoStateCompletionError::NotPending => {
            // Nothing to do.
            log::info!("Ignoring track search state change completion: Not pending");
            ActionEffect::Unchanged
        }
        track_search::MemoStateCompletionError::AbortPendingAndRetry => {
            log::info!("Aborting track search state change completion and retrying");
            memo_state.abort();
            // Replay the corresponding event.
            msg_tx
                .emit_event(library::Event::from(
                    library::track_search::Event::StateChanged,
                ))
                .unwrap();
            ActionEffect::MaybeChanged
        }
    }
}
