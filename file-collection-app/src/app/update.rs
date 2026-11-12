// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide::{
    CollectionUid, TrackEntity, TrackUid,
    desktop_app::{
        ActionEffect, collection::SynchronizingVfsFinishedState,
        track::repo_search::FetchedEntitiesMemo,
    },
    media::content::ContentPath,
};
use egui::Context;

use crate::{
    app::TrackSearchFetchedItems,
    fs::choose_directory_path,
    library::{self, Library, collection, track_edit, track_search, ui::TrackListItem},
};

use super::{
    Action, CloseState, CollectionAction, EditorSettingsAction, Event, LibraryAction, Message,
    MessageSender, Model, ModelMode, MusicDirSelection, MusicDirectoryAction, TagCloud,
    TagCloudAction, TrackEditAction, TrackEditor, TrackEditorState, TrackListColumnsAction,
    TrackSearchAction, TrackSearchEdit, TrackSearchMode, TrackSelection, TrackSelectionAction,
    TrackSelectionState,
    message::{
        ExportUnsynchronizedTracksAction, MediaTrackerAction, MediaTrackerDirListAction,
        MediaTrackerSyncAction, PendingEditsAction, RevertUnsynchronizedTracksAction,
    },
};

pub(super) struct UpdateContext<'a> {
    pub(super) rt: &'a tokio::runtime::Handle,
    pub(super) msg_tx: &'a MessageSender,
    pub(super) mdl: &'a mut Model,
}

impl UpdateContext<'_> {
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
                LibraryAction::TrackEdit(action) => self.on_library_track_edit_action(action),
                LibraryAction::TrackSelection(action) => {
                    self.on_library_track_selection_action(action)
                }
                LibraryAction::ExportUnsynchronizedTracks(action) => {
                    self.on_library_export_unsynchronized_tracks_action(action)
                }
                LibraryAction::RevertUnsynchronizedTracks(action) => {
                    self.on_library_revert_unsynchronized_tracks_action(action)
                }
            },
            Action::TrackListColumns(action) => self.on_track_list_columns_action(action),
            Action::EditorSettings(action) => self.on_editor_settings_action(action),
            Action::PendingEdits(action) => self.on_pending_edits_action(ctx, action),
            Action::TagCloud(action) => self.on_tag_cloud_action(action),
        };
        if matches!(action_effect, ActionEffect::Unchanged) {
            return;
        }
        ctx.request_repaint();
    }

    fn on_pending_edits_action(
        &mut self,
        ctx: &Context,
        action: PendingEditsAction,
    ) -> ActionEffect {
        match action {
            PendingEditsAction::Save { draft: Some(track) } => {
                let effect = self.on_library_track_edit_action(TrackEditAction::Save {
                    track,
                    write_file: false,
                });
                if !matches!(effect, ActionEffect::Unchanged) {
                    self.mdl.write_files_after_saving_draft = true;
                }
                effect
            }
            PendingEditsAction::Save { draft: None } => self
                .on_library_export_unsynchronized_tracks_action(
                    ExportUnsynchronizedTracksAction::SpawnTask,
                ),
            PendingEditsAction::RequestRevert => {
                self.mdl.is_revert_confirmation_open = true;
                ActionEffect::Changed
            }
            PendingEditsAction::CancelRevert => {
                self.mdl.is_revert_confirmation_open = false;
                ActionEffect::Changed
            }
            PendingEditsAction::ConfirmRevert => {
                self.mdl.is_revert_confirmation_open = false;
                self.spawn_revert_unsynchronized_tracks() + ActionEffect::Changed
            }
            PendingEditsAction::RequestClose => {
                self.mdl.close_state = CloseState::Requested;
                // Decide with the current count.
                refresh_modified_track_count(self.rt, self.msg_tx, self.mdl);
                ActionEffect::Changed
            }
            PendingEditsAction::CancelClose => {
                self.mdl.close_state = CloseState::NotRequested;
                ActionEffect::Changed
            }
            PendingEditsAction::CloseWithoutWriting => {
                log::info!("Closing the application without writing pending edits");
                self.mdl.close_state = CloseState::Confirmed;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                ActionEffect::Changed
            }
        }
    }

    fn on_tag_cloud_action(&mut self, action: TagCloudAction) -> ActionEffect {
        let Self { rt, msg_tx, mdl } = self;
        match action {
            TagCloudAction::Open(entity_uid) => {
                if let Some(tag_cloud) = &mut mdl.tag_cloud {
                    if tag_cloud.entity_uid == entity_uid {
                        return ActionEffect::Unchanged;
                    }
                    // The tags of the collection remain the same.
                    tag_cloud.entity_uid = entity_uid;
                    return ActionEffect::Changed;
                }
                let mut tag_cloud = TagCloud::new(entity_uid);
                refresh_tag_cloud(rt, msg_tx, &mdl.library, &mut tag_cloud);
                mdl.tag_cloud = Some(tag_cloud);
                ActionEffect::Changed
            }
            TagCloudAction::Close => {
                if mdl.tag_cloud.take().is_none() {
                    return ActionEffect::Unchanged;
                }
                ActionEffect::Changed
            }
        }
    }

    fn on_track_list_columns_action(&mut self, action: TrackListColumnsAction) -> ActionEffect {
        let Model {
            track_list_columns,
            settings_dir,
            ..
        } = &mut *self.mdl;
        let changed = match action {
            TrackListColumnsAction::Toggle(column) => track_list_columns.toggle(column),
            TrackListColumnsAction::Move {
                column,
                insertion_index,
            } => track_list_columns.move_column(column, insertion_index),
        };
        if !changed {
            return ActionEffect::Unchanged;
        }
        if let Err(err) = track_list_columns.save(settings_dir) {
            log::warn!("Failed to save track list columns: {err}");
        }
        ActionEffect::Changed
    }

    fn on_editor_settings_action(&mut self, action: EditorSettingsAction) -> ActionEffect {
        let Model {
            editor_settings,
            settings_dir,
            ..
        } = &mut *self.mdl;
        match action {
            EditorSettingsAction::SetWriteFiles(write_files) => {
                if editor_settings.write_files == write_files {
                    return ActionEffect::Unchanged;
                }
                editor_settings.write_files = write_files;
            }
        }
        if let Err(err) = editor_settings.save(settings_dir) {
            log::warn!("Failed to save editor settings: {err}");
        }
        ActionEffect::Changed
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
                choose_directory_path(
                    rt,
                    library.read_settings_state().music_dir.as_ref(),
                    on_dir_path_chosen,
                );
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

    fn on_library_media_tracker_action(&mut self, action: MediaTrackerAction) -> ActionEffect {
        let Self { rt, msg_tx, mdl } = self;
        let Model { library, mode, .. } = mdl;
        match action {
            MediaTrackerAction::Sync(action) => match action {
                MediaTrackerSyncAction::SpawnTask => {
                    let (mut effect, result) = library.sync_music_dir(rt, *msg_tx);
                    match result {
                        Ok(()) => {
                            log::debug!("Switching to synchronize music directory progress view");
                            *mode = Some(ModelMode::MusicDirSync {
                                last_progress: None,
                                final_outcome: None,
                            });
                            effect += ActionEffect::MaybeChanged;
                        }
                        Err(err) => {
                            log::warn!("Failed to synchronize music directory: {err:#}");
                        }
                    }
                    effect
                }
                MediaTrackerSyncAction::AbortPendingTask => library.sync_music_dir_abort(),
                MediaTrackerSyncAction::Finish => {
                    let Some(some_mode) = mode else {
                        return ActionEffect::Unchanged;
                    };
                    if !matches!(some_mode, ModelMode::MusicDirSync { .. }) {
                        return ActionEffect::Unchanged;
                    }
                    log::debug!("Closing synchronize music directory progress view");
                    *mode = None;
                    msg_tx.send_action(CollectionAction::RefreshFromDb);
                    ActionEffect::Changed
                }
            },
            MediaTrackerAction::DirList(action) => match action {
                MediaTrackerDirListAction::OpenView => {
                    let params = aoide::api::media::tracker::count_sources_in_directories::Params {
                        order: Some(
                            aoide::api::media::tracker::count_sources_in_directories::Order::CountDescending,
                        ),
                        ..Default::default()
                    };
                    let mut effect = library.view_music_dir_list(rt, *msg_tx, params);
                    if matches!(effect, ActionEffect::Unchanged) {
                        return effect;
                    }
                    log::debug!("Switching to music directory list view");
                    *mode = Some(ModelMode::MusicDirList {
                        content_paths_with_count: Vec::new(),
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
                    log::debug!("Closing music directory list view");
                    *mode = None;
                    msg_tx.send_action(CollectionAction::RefreshFromDb);
                    ActionEffect::Changed
                }
            },
        }
    }

    #[expect(clippy::needless_pass_by_value)]
    fn on_library_collection_action(&mut self, action: CollectionAction) -> ActionEffect {
        let Self { rt, mdl, .. } = self;
        let Model { library, mode, .. } = mdl;
        match action {
            CollectionAction::RefreshFromDb => {
                let (mut effect, abort_handle) = library.refresh_collection_from_db(rt);
                if abort_handle.is_some() && mode.is_some() {
                    log::debug!("Resetting mode {mode:?} with stale collection");
                    *mode = None;
                    effect += ActionEffect::Changed;
                }
                effect
            }
        }
    }

    #[expect(clippy::too_many_lines)] // TODO
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
            ..
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
                            let expected_offset = memo_delta.fetch.as_ref().and_then(|fetch| {
                                fetch
                                    .fetched_entities
                                    .as_ref()
                                    .map(FetchedEntitiesMemo::offset)
                            });
                            let new_offset = match &fetched_items {
                                TrackSearchFetchedItems::Reset => None,
                                TrackSearchFetchedItems::Replace(fetched_items) => {
                                    Some(fetched_items.len())
                                }
                                TrackSearchFetchedItems::Append(fetched_items) => Some(
                                    track_list.as_ref().map_or(0, Vec::len) + fetched_items.len(),
                                ),
                            };
                            if new_offset != expected_offset {
                                // The items are collected after the memo delta has been determined
                                // and the state might have changed in the meantime. Applying them
                                // would leave the track list inconsistent with the memo.
                                log::info!(
                                    "Discarding outdated track search items: {new_offset:?} <> {expected_offset:?}"
                                );
                                memo_state.abort();
                                drop(msg_tx.emit_event(library::Event::from(
                                    library::track_search::Event::StateChanged,
                                )));
                                return mode_effect + ActionEffect::MaybeChanged;
                            }
                            log::debug!("Applying track search memo change");
                            match fetched_items {
                                TrackSearchFetchedItems::Reset => {
                                    log::debug!(
                                        "Track search list changed: No fetched items available"
                                    );
                                    *track_list = None;
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
                                            .map(FetchedEntitiesMemo::offset),
                                    );
                                    log::debug!(
                                        "Track search list changed: Appending {count_append} fetched items to {count_before} existing items",
                                        count_before = track_list.len(),
                                        count_append = fetched_items.len()
                                    );
                                    track_list.extend(fetched_items);
                                }
                            }
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

    fn spawn_revert_unsynchronized_tracks(&mut self) -> ActionEffect {
        let Self { rt, msg_tx, mdl } = self;
        let Model { library, mode, .. } = mdl;
        let (mut effect, result) = library.revert_unsynchronized_tracks(rt, *msg_tx);
        match result {
            Ok(()) => {
                // Also discards the track editor including its draft.
                log::debug!("Switching to revert unsynchronized tracks progress view");
                *mode = Some(ModelMode::RevertUnsynchronizedTracks {
                    last_progress: None,
                    final_outcome: None,
                });
                effect += ActionEffect::MaybeChanged;
            }
            Err(err) => {
                log::warn!("Failed to revert unsynchronized tracks: {err:#}");
            }
        }
        effect
    }

    fn on_library_revert_unsynchronized_tracks_action(
        &mut self,
        action: RevertUnsynchronizedTracksAction,
    ) -> ActionEffect {
        let Self { msg_tx, mdl, .. } = self;
        let Model { library, mode, .. } = mdl;
        match action {
            RevertUnsynchronizedTracksAction::AbortPendingTask => {
                library.revert_unsynchronized_tracks_abort()
            }
            RevertUnsynchronizedTracksAction::Finish => {
                if !matches!(mode, Some(ModelMode::RevertUnsynchronizedTracks { .. })) {
                    return ActionEffect::Unchanged;
                }
                log::debug!("Closing revert unsynchronized tracks progress view");
                *mode = None;
                // Reload the reverted tracks.
                msg_tx.send_action(CollectionAction::RefreshFromDb);
                ActionEffect::Changed
            }
        }
    }

    fn on_library_export_unsynchronized_tracks_action(
        &mut self,
        action: ExportUnsynchronizedTracksAction,
    ) -> ActionEffect {
        let Self { rt, msg_tx, mdl } = self;
        let Model { library, mode, .. } = mdl;
        match action {
            ExportUnsynchronizedTracksAction::SpawnTask => {
                let (mut effect, result) = library.export_unsynchronized_tracks(rt, *msg_tx);
                match result {
                    Ok(()) => {
                        log::debug!("Switching to export unsynchronized tracks progress view");
                        *mode = Some(ModelMode::ExportUnsynchronizedTracks {
                            last_progress: None,
                            final_outcome: None,
                        });
                        effect += ActionEffect::MaybeChanged;
                    }
                    Err(err) => {
                        log::warn!("Failed to export unsynchronized tracks: {err:#}");
                    }
                }
                effect
            }
            ExportUnsynchronizedTracksAction::AbortPendingTask => {
                library.export_unsynchronized_tracks_abort()
            }
            ExportUnsynchronizedTracksAction::Finish => {
                if !matches!(mode, Some(ModelMode::ExportUnsynchronizedTracks { .. })) {
                    return ActionEffect::Unchanged;
                }
                log::debug!("Closing export unsynchronized tracks progress view");
                *mode = None;
                // Reload the modified tracks.
                msg_tx.send_action(CollectionAction::RefreshFromDb);
                ActionEffect::Changed
            }
        }
    }

    fn on_library_track_selection_action(&mut self, action: TrackSelectionAction) -> ActionEffect {
        let Self { rt, msg_tx, mdl } = self;
        let Model { library, mode, .. } = mdl;
        let Some(ModelMode::TrackSearch(TrackSearchMode {
            edit, track_list, ..
        })) = mode
        else {
            log::info!("Discarding track selection action: {action:?}");
            return ActionEffect::Unchanged;
        };
        if edit.as_ref().is_some_and(TrackSearchEdit::is_pending)
            && !matches!(action, TrackSelectionAction::AbortUpdate)
        {
            log::info!("Discarding track selection action while pending: {action:?}");
            return ActionEffect::Unchanged;
        }
        match action {
            TrackSelectionAction::Toggle(entity_uid) => {
                let mut track_selection = take_track_selection(edit, Some(&entity_uid));
                track_selection.toggle(entity_uid);
                replace_track_selection(edit, track_selection);
                ActionEffect::Changed
            }
            TrackSelectionAction::SelectRange(entity_uid) => {
                let mut track_selection = take_track_selection(edit, None);
                let ordered_entity_uids = track_list.iter().flatten().map(|item| &item.entity_uid);
                track_selection.select_range(ordered_entity_uids, entity_uid);
                replace_track_selection(edit, track_selection);
                ActionEffect::Changed
            }
            TrackSelectionAction::Clear => {
                if !matches!(edit, Some(TrackSearchEdit::Multiple(_))) {
                    return ActionEffect::Unchanged;
                }
                *edit = None;
                ActionEffect::Changed
            }
            TrackSelectionAction::Update {
                edit: multi_track_edit,
                write_files,
            } => {
                let Some(TrackSearchEdit::Multiple(track_selection)) = edit else {
                    return ActionEffect::Unchanged;
                };
                let (mut effect, result) = library.update_tracks(
                    rt,
                    *msg_tx,
                    track_selection.entity_uids.clone(),
                    *multi_track_edit,
                    write_files,
                );
                match result {
                    Ok(()) => {
                        track_selection.state = TrackSelectionState::Updating {
                            last_progress: None,
                            write_files,
                        };
                        effect += ActionEffect::Changed;
                    }
                    Err(err) => {
                        log::warn!("Failed to update the selected tracks: {err:#}");
                    }
                }
                effect
            }
            TrackSelectionAction::AbortUpdate => library.update_tracks_abort(),
        }
    }

    fn on_library_track_edit_action(&mut self, action: TrackEditAction) -> ActionEffect {
        let Self { rt, msg_tx, mdl } = self;
        let Model {
            library,
            mode,
            active_track_writes,
            ..
        } = mdl;
        let Some(ModelMode::TrackSearch(TrackSearchMode {
            edit,
            track_list_edit_error,
            ..
        })) = mode
        else {
            log::info!("Discarding track edit action: {action:?}");
            return ActionEffect::Unchanged;
        };
        match action {
            TrackEditAction::Apply {
                entity_uid,
                edit: track_edit,
            } => {
                *track_list_edit_error = None;
                let effect = library.apply_track_edit(rt, *msg_tx, entity_uid, *track_edit);
                if !matches!(effect, ActionEffect::Unchanged) {
                    *active_track_writes += 1;
                }
                effect
            }
            TrackEditAction::Open(entity_uid) => {
                if edit
                    .as_ref()
                    .and_then(TrackSearchEdit::track_selection)
                    .is_some_and(TrackSelection::is_updating)
                {
                    log::info!("Discarding open action while updating the selected tracks");
                    return ActionEffect::Unchanged;
                }
                let effect = library.load_track_for_edit(rt, *msg_tx, entity_uid.clone());
                if matches!(effect, ActionEffect::Unchanged) {
                    return effect;
                }
                log::debug!("Opening track editor for {entity_uid}");
                *edit = Some(TrackSearchEdit::Single(TrackEditor::loading(entity_uid)));
                ActionEffect::Changed
            }
            TrackEditAction::Save { track, write_file } => {
                let Some(track_editor) = edit.as_mut().and_then(TrackSearchEdit::track_editor_mut)
                else {
                    log::info!("Discarding save action without track editor");
                    return ActionEffect::Unchanged;
                };
                let TrackEditorState::Ready { entity } = &track_editor.state else {
                    log::info!(
                        "Discarding save action in state {state:?}",
                        state = track_editor.state
                    );
                    return ActionEffect::Unchanged;
                };
                let expected_entity_hdr = entity.hdr.clone();
                let effect =
                    library.save_edited_track(rt, *msg_tx, expected_entity_hdr, *track, write_file);
                if matches!(effect, ActionEffect::Unchanged) {
                    return effect;
                }
                track_editor.start_saving(write_file);
                *active_track_writes += 1;
                ActionEffect::Changed
            }
            TrackEditAction::Close => {
                let Some(TrackSearchEdit::Single(track_editor)) =
                    edit.take_if(|edit| matches!(edit, TrackSearchEdit::Single(_)))
                else {
                    return ActionEffect::Unchanged;
                };
                if matches!(track_editor.state, TrackEditorState::Saving { .. }) {
                    // The pending result will still update the track list.
                    log::info!(
                        "Closing track editor for {uid} while saving",
                        uid = track_editor.entity_uid
                    );
                }
                ActionEffect::Changed
            }
        }
    }

    fn on_event(&mut self, ctx: &Context, event: Event) {
        match event {
            Event::Library(event) => {
                self.on_library_event(ctx, event);
            }
        }
    }

    fn on_library_event(&mut self, ctx: &Context, event: library::Event) {
        let is_modified_track_count_outdated = match &event {
            library::Event::Collection(library::collection::Event::StateChanged)
            | library::Event::MusicDirSyncFinished(_)
            | library::Event::ExportUnsynchronizedTracksFinished(_)
            | library::Event::RevertUnsynchronizedTracksFinished(_)
            | library::Event::UpdateTracksFinished(_) => true,
            library::Event::TrackEdit(event) => matches!(
                **event,
                track_edit::Event::Saved { .. } | track_edit::Event::Applied { .. }
            ),
            _ => false,
        };
        self.on_library_event_inner(ctx, event);
        if is_modified_track_count_outdated {
            refresh_modified_track_count(self.rt, self.msg_tx, self.mdl);
            ctx.request_repaint();
        }
        // The tags might have changed. Edits of multiple tracks are
        // applied one by one and only the last one needs to be awaited.
        if is_modified_track_count_outdated
            && self.mdl.active_track_writes == 0
            && let Some(tag_cloud) = &mut self.mdl.tag_cloud
        {
            refresh_tag_cloud(self.rt, self.msg_tx, &self.mdl.library, tag_cloud);
        }
    }

    fn on_library_event_inner(&mut self, ctx: &Context, event: library::Event) {
        let Self { rt, msg_tx, mdl } = self;
        let Model {
            library,
            mode,
            modified_track_count,
            active_track_writes,
            write_files_after_saving_draft,
            tag_cloud,
            ..
        } = mdl;
        match event {
            library::Event::Settings(library::settings::Event::StateChanged) => {
                // Nothing to do.
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
            library::Event::TrackEdit(event) => {
                let write_finished = TrackWriteFinished::from_event(&event);
                on_library_track_edit_event(ctx, library, mode, *event);
                if let Some(write_finished) = write_finished {
                    *active_track_writes = active_track_writes.saturating_sub(1);
                    // Edits that are unrelated to the draft must not affect the pending decision.
                    if !matches!(write_finished, TrackWriteFinished::Applied)
                        && std::mem::take(write_files_after_saving_draft)
                        && matches!(write_finished, TrackWriteFinished::Saved)
                    {
                        msg_tx.send_action(ExportUnsynchronizedTracksAction::SpawnTask);
                    }
                }
            }
            library::Event::UnsynchronizedTracksCounted { request_id, result } => {
                let count = result
                    .inspect_err(|err| {
                        log::warn!("Failed to count unsynchronized tracks: {err:#}");
                    })
                    .ok();
                if modified_track_count.finish_counting(request_id, count) {
                    ctx.request_repaint();
                }
            }
            library::Event::TagLabelsCounted { request_id, result } => {
                on_library_tag_labels_counted(ctx, tag_cloud.as_mut(), request_id, result);
            }
            library::Event::MusicDirSyncProgress(progress) => {
                if let Some(ModelMode::MusicDirSync {
                    last_progress,
                    final_outcome,
                }) = mode
                {
                    // The last progress might arrive after the outcome.
                    if final_outcome.is_none() {
                        *last_progress = progress;
                    }
                } else {
                    log::debug!(
                        "Discarding unexpected music directory synchronization progress: {progress:?}"
                    );
                }
            }
            library::Event::MusicDirSyncFinished(finished_state) => {
                on_library_music_dir_sync_finished(library, mode, *finished_state);
            }
            library::Event::ExportUnsynchronizedTracksProgress(progress) => {
                on_library_export_unsynchronized_tracks_progress(mode, progress);
            }
            library::Event::ExportUnsynchronizedTracksFinished(result) => {
                on_library_export_unsynchronized_tracks_finished(library, mode, *result);
            }
            library::Event::RevertUnsynchronizedTracksProgress(progress) => {
                if let Some(ModelMode::RevertUnsynchronizedTracks {
                    last_progress,
                    final_outcome: None,
                }) = mode
                {
                    *last_progress = Some(progress);
                }
            }
            library::Event::RevertUnsynchronizedTracksFinished(result) => {
                on_library_revert_unsynchronized_tracks_finished(library, mode, *result);
            }
            library::Event::UpdateTracksProgress(progress) => {
                on_library_update_tracks_progress(mode, progress);
            }
            library::Event::UpdateTracksFinished(result) => {
                on_library_update_tracks_finished(ctx, library, mode, *result);
            }
            library::Event::MusicDirListResult {
                collection_uid,
                params: _,
                result,
            } => {
                on_library_music_dir_list_result(library, mode, &collection_uid, result);
            }
        }
    }
}

fn on_library_update_tracks_progress(
    mode: &mut Option<ModelMode>,
    progress: library::update_tracks::Progress,
) {
    if let Some(ModelMode::TrackSearch(TrackSearchMode {
        edit:
            Some(TrackSearchEdit::Multiple(TrackSelection {
                state: TrackSelectionState::Updating { last_progress, .. },
                ..
            })),
        ..
    })) = mode
    {
        *last_progress = Some(progress);
    } else {
        log::debug!("Discarding unexpected update tracks progress: {progress:?}");
    }
}

fn on_library_music_dir_sync_finished(
    library: &mut Library,
    mode: &mut Option<ModelMode>,
    finished_state: SynchronizingVfsFinishedState,
) {
    let _ = library.sync_music_dir_finished();
    let outcome = match finished_state {
        SynchronizingVfsFinishedState::Succeeded { outcome } => Ok(Box::new(outcome)),
        SynchronizingVfsFinishedState::Aborted => Ok(Box::new(
            aoide::backend_embedded::batch::synchronize_collection_vfs::Outcome {
                completion:
                    aoide::backend_embedded::batch::synchronize_collection_vfs::Completion::Aborted,
                ..Default::default()
            },
        )),
        SynchronizingVfsFinishedState::Failed { error } => {
            log::warn!("Synchronizing music directory failed: {error}");
            Err(error)
        }
    };
    if let Some(ModelMode::MusicDirSync { final_outcome, .. }) = mode {
        debug_assert!(final_outcome.is_none());
        *final_outcome = Some(outcome);
    } else {
        log::debug!(
            "Discarding unexpected music directory synchronization outcome {outcome:?}: {mode:?}"
        );
    }
}

/// A single track edit has been written into the database.
#[derive(Debug, Clone, Copy)]
enum TrackWriteFinished {
    Applied,
    Saved,
    SavingFailed,
}

impl TrackWriteFinished {
    #[must_use]
    const fn from_event(event: &track_edit::Event) -> Option<Self> {
        match event {
            track_edit::Event::Loaded { .. } => None,
            track_edit::Event::Applied { .. } => Some(Self::Applied),
            track_edit::Event::Saved {
                result: Ok(track_edit::Outcome::Updated { .. }),
                ..
            } => Some(Self::Saved),
            track_edit::Event::Saved { .. } => Some(Self::SavingFailed),
        }
    }
}

/// Count the modified tracks of the current collection.
fn refresh_modified_track_count(
    rt: &tokio::runtime::Handle,
    msg_tx: &MessageSender,
    mdl: &mut Model,
) {
    let request_id = mdl.modified_track_count.start_counting();
    let effect = mdl
        .library
        .count_unsynchronized_tracks(rt, msg_tx, request_id);
    if matches!(effect, ActionEffect::Unchanged) {
        mdl.modified_track_count.reset();
    }
}

fn on_library_tag_labels_counted(
    ctx: &Context,
    tag_cloud: Option<&mut TagCloud>,
    request_id: u64,
    result: anyhow::Result<Vec<(String, usize)>>,
) {
    let Some(tag_cloud) = tag_cloud else {
        return;
    };
    let result = result.map_err(|err| {
        log::warn!("Failed to load tags: {err:#}");
        format!("Failed to load tags: {err:#}")
    });
    if tag_cloud.finish_loading(request_id, result) {
        ctx.request_repaint();
    }
}

/// Load the tags of the current collection.
fn refresh_tag_cloud(
    rt: &tokio::runtime::Handle,
    msg_tx: &MessageSender,
    library: &Library,
    tag_cloud: &mut TagCloud,
) {
    let request_id = tag_cloud.start_loading();
    let effect = library.count_tag_labels(rt, msg_tx, request_id);
    if matches!(effect, ActionEffect::Unchanged) {
        tag_cloud.abort_loading();
    }
}

fn on_library_collection_state_changed(ctx: &Context, mdl: &mut Model, msg_tx: &MessageSender) {
    let Model {
        library,
        music_dir_selection,
        mode,
        ..
    } = mdl;
    // Update the UI unconditionally to reflect the new state.
    ctx.request_repaint();

    let mut reset_music_dir = false;
    {
        let collection_state = library.read_collection_state();
        // Determine a follow-up effect or action dependent on the new state.
        // TODO: Store or report outcomes and errors from these dead end states.
        match &*collection_state {
            collection::State::Void
                // Nothing to show with no collection available. This prevents to
                // show stale data after the collection has been reset.
                if mode.is_some() => {
                    log::debug!("Resetting mode {mode:?} without active collection");
                    *mode = None;
            }
            collection::State::LoadingFromDatabase {
                state: collection::LoadingFromDatabaseState::Finished(collection::LoadingFromDatabaseFinishedState::Failed { .. }),
                ..
            }
            | collection::State::RestoringFromMusicDirectory {
                state:
                    collection::RestoringFromMusicDirectoryState::Finished(collection::RestoringFromMusicDirectoryFinishedState::Failed { .. }
                        | collection::RestoringFromMusicDirectoryFinishedState::NestedDirectoriesConflict { .. }),
                ..
            } => {
                reset_music_dir = true;
            }
            collection::State::Ready { summary, .. }
                if matches!(music_dir_selection, Some(MusicDirSelection::Selected)) => {
                    *music_dir_selection = None;
                    if summary.media_sources.total_count == 0 {
                        log::info!(
                            "Synchronizing music directory after empty collection has been selected"
                        );
                        msg_tx.send_action(MediaTrackerSyncAction::SpawnTask);
                    }
            }
            _ => (),
        }
    }

    // Reset the music directory after releasing the read-lock.
    if reset_music_dir {
        // The UI will be repainted in any case (see above).
        let _ = library.reset_music_dir();
    }
}

fn on_library_music_dir_list_result(
    library: &Library,
    mode: &mut Option<ModelMode>,
    collection_uid: &CollectionUid,
    result: anyhow::Result<Vec<(ContentPath<'static>, usize)>>,
) {
    let new_content_paths_with_count = match result {
        Ok(content_paths_with_count) => content_paths_with_count,
        Err(err) => {
            log::warn!("Failed to view music directory list: {err}");
            // TODO: Set last error.
            return;
        }
    };
    if Some(collection_uid) != library.read_collection_state().entity_uid() {
        log::debug!(
            "Discarding unexpected music directory list with {num_items} item(s) for collection {collection_uid}: {mode:?}",
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
            "Discarding unexpected music directory list with {num_items} item(s): {mode:?}",
            num_items = new_content_paths_with_count.len()
        );
    }
}

/// Continue with the current selection or start a new one.
///
/// Starting from the editor of a single track continues with the
/// edited track, unless it is about to be toggled.
#[must_use]
fn take_track_selection(
    edit: &mut Option<TrackSearchEdit>,
    toggled_entity_uid: Option<&TrackUid>,
) -> TrackSelection {
    match edit.take() {
        None => TrackSelection::new(Vec::new(), None),
        Some(TrackSearchEdit::Single(track_editor)) => {
            if Some(&track_editor.entity_uid) == toggled_entity_uid {
                TrackSelection::new(Vec::new(), None)
            } else {
                let entity_uid = track_editor.entity_uid;
                TrackSelection::new(vec![entity_uid.clone()], Some(entity_uid))
            }
        }
        Some(TrackSearchEdit::Multiple(track_selection)) => {
            // Discard the outcome of the last update after modifying the selection.
            let TrackSelection {
                entity_uids,
                anchor_entity_uid,
                state: _,
            } = track_selection;
            TrackSelection::new(entity_uids, anchor_entity_uid)
        }
    }
}

fn replace_track_selection(edit: &mut Option<TrackSearchEdit>, track_selection: TrackSelection) {
    *edit = (!track_selection.entity_uids.is_empty())
        .then_some(TrackSearchEdit::Multiple(track_selection));
}

fn on_library_update_tracks_finished(
    ctx: &Context,
    library: &mut Library,
    mode: &mut Option<ModelMode>,
    result: anyhow::Result<library::update_tracks::Outcome>,
) {
    let _ = library.update_tracks_finished();
    let (track_list, track_selection) = match mode {
        Some(ModelMode::TrackSearch(TrackSearchMode {
            track_list, edit, ..
        })) => (
            track_list.as_deref_mut(),
            edit.as_mut().and_then(TrackSearchEdit::track_selection_mut),
        ),
        _ => (None, None),
    };
    let outcome = match result {
        Ok(outcome) => {
            log::info!(
                "Updating tracks finished: {completion:?}, {updated} updated, {unchanged} \
                 unchanged, {skipped} skipped, {failed} failed",
                completion = outcome.completion,
                updated = outcome.updated.len(),
                unchanged = outcome.unchanged.len(),
                skipped = outcome.skipped.len(),
                failed = outcome.failed.len(),
            );
            // The track list must be updated independent of the selection.
            let mut track_list = track_list;
            for entity in &outcome.updated {
                let _ = library.replace_track_search_result(entity.clone());
                if let Some(track_list) = track_list.as_deref_mut() {
                    replace_track_list_item(ctx, track_list, entity);
                }
            }
            Ok(Box::new(outcome))
        }
        Err(err) => {
            log::warn!("Updating tracks failed: {err:#}");
            Err(format!("{err:#}"))
        }
    };
    if let Some(track_selection) = track_selection
        && track_selection.is_updating()
    {
        track_selection.state = TrackSelectionState::Ready {
            last_outcome: Some(outcome),
        };
    } else {
        log::debug!("Discarding unexpected update tracks outcome");
    }
    ctx.request_repaint();
}

fn on_library_export_unsynchronized_tracks_progress(
    mode: &mut Option<ModelMode>,
    progress: library::export_unsynchronized_tracks::Progress,
) {
    if let Some(ModelMode::ExportUnsynchronizedTracks {
        last_progress,
        final_outcome,
    }) = mode
    {
        debug_assert!(final_outcome.is_none());
        *last_progress = Some(progress);
    } else {
        log::debug!("Discarding unexpected export unsynchronized tracks progress: {progress:?}");
    }
}

fn on_library_revert_unsynchronized_tracks_finished(
    library: &mut Library,
    mode: &mut Option<ModelMode>,
    result: anyhow::Result<library::revert_unsynchronized_tracks::Outcome>,
) {
    let _ = library.revert_unsynchronized_tracks_finished();
    let outcome = match result {
        Ok(outcome) => {
            log::info!(
                "Reverting unsynchronized tracks finished: {completion:?}, {reverted} reverted, \
                 {failed} failed",
                completion = outcome.completion,
                reverted = outcome.reverted.len(),
                failed = outcome.failed.len(),
            );
            Ok(Box::new(outcome))
        }
        Err(err) => {
            log::warn!("Reverting unsynchronized tracks failed: {err:#}");
            Err(format!("{err:#}"))
        }
    };
    if let Some(ModelMode::RevertUnsynchronizedTracks { final_outcome, .. }) = mode {
        *final_outcome = Some(outcome);
    } else {
        log::debug!("Discarding unexpected revert unsynchronized tracks outcome: {mode:?}");
    }
}

fn on_library_export_unsynchronized_tracks_finished(
    library: &mut Library,
    mode: &mut Option<ModelMode>,
    result: anyhow::Result<library::export_unsynchronized_tracks::Outcome>,
) {
    let _ = library.export_unsynchronized_tracks_finished();
    let outcome = match result {
        Ok(outcome) => {
            log::info!(
                "Exporting unsynchronized tracks finished: {completion:?}, {exported} \
                     exported, {skipped} skipped, {failed} failed",
                completion = outcome.completion,
                exported = outcome.exported.len(),
                skipped = outcome.skipped.len(),
                failed = outcome.failed.len(),
            );
            Ok(Box::new(outcome))
        }
        Err(err) => {
            log::warn!("Exporting unsynchronized tracks failed: {err:#}");
            Err(format!("{err:#}"))
        }
    };
    if let Some(ModelMode::ExportUnsynchronizedTracks { final_outcome, .. }) = mode {
        debug_assert!(final_outcome.is_none());
        *final_outcome = Some(outcome);
    } else {
        log::debug!("Discarding unexpected export unsynchronized tracks outcome: {mode:?}");
    }
}

#[must_use]
fn is_current_collection(library: &Library, collection_uid: &CollectionUid) -> bool {
    Some(collection_uid) == library.read_collection_state().entity_uid()
}

fn on_library_track_edit_event(
    ctx: &Context,
    library: &Library,
    mode: &mut Option<ModelMode>,
    event: track_edit::Event,
) {
    match event {
        track_edit::Event::Loaded {
            collection_uid,
            entity_uid,
            result,
        } => {
            if !is_current_collection(library, &collection_uid) {
                log::debug!("Discarding loaded track {entity_uid} of collection {collection_uid}");
                return;
            }
            let Some(ModelMode::TrackSearch(TrackSearchMode {
                edit: Some(TrackSearchEdit::Single(track_editor)),
                ..
            })) = mode
            else {
                log::debug!("Discarding loaded track {entity_uid} without track editor");
                return;
            };
            if track_editor.entity_uid != entity_uid
                || !matches!(track_editor.state, TrackEditorState::Loading)
            {
                log::debug!("Discarding unexpected loaded track {entity_uid}: {track_editor:?}");
                return;
            }
            match result {
                Ok(entity) => {
                    log::debug!("Loaded track {hdr:?} for editing", hdr = entity.hdr);
                    track_editor.state = TrackEditorState::Ready {
                        entity: Box::new(entity),
                    };
                    track_editor.last_error = None;
                }
                Err(err) => {
                    log::warn!("Failed to load track {entity_uid} for editing: {err:#}");
                    track_editor.state = TrackEditorState::LoadingFailed;
                    track_editor.last_error = Some(format!("Failed to load track: {err:#}"));
                }
            }
            ctx.request_repaint();
        }
        track_edit::Event::Saved {
            collection_uid,
            entity_uid,
            result,
        } => {
            if !is_current_collection(library, &collection_uid) {
                log::debug!("Discarding saved track {entity_uid} of collection {collection_uid}");
                return;
            }
            let (track_list, track_editor) = match mode {
                Some(ModelMode::TrackSearch(TrackSearchMode {
                    track_list, edit, ..
                })) => (
                    track_list.as_deref_mut(),
                    edit.as_mut().and_then(TrackSearchEdit::track_editor_mut),
                ),
                _ => (None, None),
            };
            // The editor might have been closed or re-opened for a different track in the meantime.
            let track_editor = track_editor.filter(|track_editor| {
                track_editor.entity_uid == entity_uid
                    && matches!(track_editor.state, TrackEditorState::Saving { .. })
            });
            let result = match result {
                Ok(track_edit::Outcome::Updated { entity, exported }) => {
                    log::info!(
                        "Saved track {hdr:?} (file written: {exported})",
                        hdr = entity.hdr
                    );
                    // The track list must be updated independent of the editor.
                    let _ = library.replace_track_search_result(entity.clone());
                    if let Some(track_list) = track_list {
                        replace_track_list_item(ctx, track_list, &entity);
                    }
                    Ok(entity)
                }
                Ok(track_edit::Outcome::Rejected(conflict)) => {
                    log::info!("Saving track {entity_uid} rejected: {conflict:?}");
                    Err(track_edit_conflict_message(conflict).to_owned())
                }
                Err(err) => {
                    log::warn!("Failed to save track {entity_uid}: {err:#}");
                    Err(format!("Failed to save track: {err:#}"))
                }
            };
            if let Some(track_editor) = track_editor {
                track_editor.finish_saving(result);
            }
            ctx.request_repaint();
        }
        track_edit::Event::Applied {
            collection_uid,
            entity_uid,
            result,
        } => {
            if !is_current_collection(library, &collection_uid) {
                log::debug!("Discarding edit of track {entity_uid} of collection {collection_uid}");
                return;
            }
            on_library_track_edit_applied(ctx, library, mode, &entity_uid, result);
        }
    }
}

fn on_library_track_edit_applied(
    ctx: &Context,
    library: &Library,
    mode: &mut Option<ModelMode>,
    entity_uid: &TrackUid,
    result: anyhow::Result<library::update_tracks::Outcome>,
) {
    let (updated, error) = match result {
        Ok(outcome) => {
            let error = outcome
                .skipped
                .into_iter()
                .map(|(_, conflict)| track_edit_conflict_message(conflict).to_owned())
                .chain(outcome.failed.into_iter().map(|(_, error)| error))
                .next();
            (outcome.updated, error)
        }
        Err(err) => (Vec::new(), Some(format!("{err:#}"))),
    };
    if let Some(error) = &error {
        log::warn!("Failed to edit track {entity_uid}: {error}");
    }
    for entity in &updated {
        // The track list must be updated independent of the current mode.
        let _ = library.replace_track_search_result(entity.clone());
    }
    if let Some(ModelMode::TrackSearch(TrackSearchMode {
        track_list,
        edit,
        track_list_edit_error,
        ..
    })) = mode
    {
        *track_list_edit_error = error.map(|error| format!("Failed to edit track: {error}"));
        for entity in updated {
            if let Some(track_list) = track_list.as_deref_mut() {
                replace_track_list_item(ctx, track_list, &entity);
            }
            // Otherwise saving in the track editor would be rejected as a conflict.
            // Edits are not possible while the draft of the track editor is modified.
            if let Some(TrackEditor {
                entity_uid,
                state: TrackEditorState::Ready { entity: edited },
                ..
            }) = edit.as_mut().and_then(TrackSearchEdit::track_editor_mut)
                && *entity_uid == entity.hdr.uid
            {
                **edited = entity;
            }
        }
    }
    ctx.request_repaint();
}

#[must_use]
const fn track_edit_conflict_message(conflict: track_edit::Conflict) -> &'static str {
    match conflict {
        track_edit::Conflict::EntityRevision { .. } => {
            "The track has been modified in the meantime. Reload the track to discard your changes."
        }
        track_edit::Conflict::ContentRevision { .. } => {
            "The file has been modified on disk. Synchronize the music directory and reload the \
             track to discard your changes."
        }
    }
}

fn replace_track_list_item(ctx: &Context, track_list: &mut [TrackListItem], entity: &TrackEntity) {
    let Some(item) = track_list
        .iter_mut()
        .find(|item| item.entity_uid == entity.hdr.uid)
    else {
        return;
    };
    // The content URL is not resolved after saving and the content path is not supposed to change.
    let content_url = entity
        .body
        .content_url
        .clone()
        .or_else(|| item.content_url.take());
    *item = TrackListItem::new(
        ctx,
        entity.hdr.uid.clone(),
        content_url,
        is_synchronized(entity),
        &entity.body.track,
    );
}

/// Check if the current revision of the track has been written into the file.
#[must_use]
pub(super) fn is_synchronized(entity: &TrackEntity) -> bool {
    entity.body.last_synchronized_rev == Some(entity.hdr.rev)
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
            unreachable!("not pending without changes");
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
            .map_or(0, FetchedEntitiesMemo::offset),
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
                        is_synchronized(&fetched_entity.entity),
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
            drop(msg_tx.emit_event(library::Event::from(
                library::track_search::Event::StateChanged,
            )));
            ActionEffect::MaybeChanged
        }
    }
}
