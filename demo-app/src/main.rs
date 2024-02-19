// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

// Remove later.
#![allow(dead_code)]
#![allow(unreachable_pub)]

use std::{
    fs,
    path::{Path, PathBuf},
    sync::mpsc,
};

use directories::ProjectDirs;
use eframe::{CreationContext, Frame};
use egui::{Button, CentralPanel, Context, TextEdit, TopBottomPanel};

use aoide::desktop_app::{collection, fs::DirPath};

mod library;
use self::library::Library;

const MUSIC_DIR_SYNC_PROGRESS_LOG_MAX_LINES: usize = 100;

#[derive(Debug)]
struct NoReceiverForEvent;

#[derive(Debug)]
struct NoReceiverForAppMessage(pub AppMessage);

#[allow(missing_debug_implementations)]
#[derive(Clone)]
struct AppMessageSender {
    ctx: Context,
    msg_tx: mpsc::Sender<AppMessage>,
}

impl AppMessageSender {
    const fn new(ctx: Context, msg_tx: mpsc::Sender<AppMessage>) -> Self {
        Self { ctx, msg_tx }
    }

    fn send_action<T>(&self, action: T)
    where
        T: Into<AppAction>,
    {
        if let Err(NoReceiverForAppMessage(msg)) =
            self.send_message(AppMessage::Action(action.into()))
        {
            let AppMessage::Action(action) = msg else {
                unreachable!()
            };
            log::warn!("No receiver for action: {action:?}");
        }
    }

    fn emit_event<T>(&self, event: T) -> Result<(), NoReceiverForEvent>
    where
        T: Into<AppEvent>,
    {
        if let Err(NoReceiverForAppMessage(msg)) =
            self.send_message(AppMessage::Event(event.into()))
        {
            let AppMessage::Event(event) = msg else {
                unreachable!()
            };
            log::warn!("No receiver for event: {event:?}");
            return Err(NoReceiverForEvent);
        }
        Ok(())
    }

    fn send_message(&self, msg: AppMessage) -> Result<(), NoReceiverForAppMessage> {
        log::debug!("Sending message {msg:?}");
        self.msg_tx.send(msg).map_err(|err| {
            log::warn!("Failed to send message: {err}");
            NoReceiverForAppMessage(err.0)
        })?;
        // Queued messages are consumed before rendering the next frame.
        self.ctx.request_repaint();
        Ok(())
    }
}

impl library::EventEmitter for AppMessageSender {
    fn emit_event(&self, event: library::Event) -> Result<(), NoReceiverForEvent> {
        let event: AppEvent = AppEvent::Library(event);
        self.send_message(AppMessage::Event(event))
            .map_err(|NoReceiverForAppMessage(_)| NoReceiverForEvent)
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let Some(config_dir) = app_config_dir() else {
        log::error!("Config directory is unavailable");
        return;
    };

    if !config_dir.exists() {
        log::error!(
            "Config directory '{dir_path}' does not exist",
            dir_path = config_dir.display()
        );
        return;
    }

    match config_dir
        .metadata()
        .map(|metadata| metadata.permissions().readonly())
    {
        Ok(readonly) => {
            if readonly {
                log::warn!(
                    "Config directory (read-only): {dir_path}",
                    dir_path = config_dir.display()
                );
            } else {
                log::info!(
                    "Config directory: {dir_path}",
                    dir_path = config_dir.display()
                );
            }
        }
        Err(err) => {
            log::error!("Failed to query permissions of config directory: {err}");
        }
    };

    let aoide_initial_settings =
        match aoide::desktop_app::settings::State::restore_from_parent_dir(&config_dir) {
            Ok(settings) => settings,
            Err(err) => {
                log::error!("Failed to restore aoide settings: {err}");
                return;
            }
        };
    let aoide_db_config = match aoide_initial_settings.create_database_config() {
        Ok(db_config) => db_config,
        Err(err) => {
            log::error!("Failed to create aoide database config: {err}");
            return;
        }
    };
    log::debug!("Commissioning aoide library backend: {aoide_db_config:?}");
    let aoide_handle = match aoide::desktop_app::Handle::commission(&aoide_db_config) {
        Ok(library_backend) => library_backend,
        Err(err) => {
            log::error!("Failed to commission aoide library backend: {err}");
            return;
        }
    };
    let library = Library::new(aoide_handle, aoide_initial_settings);

    let rt = match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle,
        Err(err) => {
            log::error!("No Tokio runtime: {err}");
            return;
        }
    };

    eframe::run_native(
        app_name(),
        eframe::NativeOptions::default(),
        Box::new(move |ctx| {
            let mdl = AppModel::new(library);
            let app = App::new(ctx, rt, mdl, config_dir);
            Box::new(app)
        }),
    )
    .unwrap();
}

#[must_use]
const fn disable_synchronize_collection(state: &collection::State) -> bool {
    !state.is_ready()
}

#[must_use]
const fn app_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

#[must_use]
fn app_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("", "", app_name())
}

#[must_use]
fn init_config_dir(app_dirs: &ProjectDirs) -> &Path {
    let app_config_dir = app_dirs.config_dir();
    if let Err(err) = fs::create_dir_all(app_config_dir) {
        log::warn!(
            "Failed to create config directory '{dir}': {err}",
            dir = app_config_dir.display(),
        );
    }
    app_config_dir
}

fn app_config_dir() -> Option<PathBuf> {
    app_dirs()
        .as_ref()
        .map(init_config_dir)
        .map(Path::to_path_buf)
}

#[derive(Debug)]
// Not cloneable so large enum variants should be fine.
#[allow(clippy::large_enum_variant)]
enum AppMessage {
    Action(AppAction),
    Event(AppEvent),
}

#[derive(Debug, Clone)]
enum AppInputEvent {
    TrackSearch(String),
}

impl From<AppInputEvent> for AppEvent {
    fn from(event: AppInputEvent) -> Self {
        Self::Input(event)
    }
}

#[derive(Debug, Clone)]
enum AppAction {
    Library(LibraryAction),
}

impl From<AppAction> for AppMessage {
    fn from(action: AppAction) -> Self {
        Self::Action(action)
    }
}

#[derive(Debug, Clone)]
enum LibraryAction {
    MusicDirectory(MusicDirectoryAction),
    Collection(CollectionAction),
    TrackSearch(TrackSearchAction),
}

impl<T> From<T> for AppAction
where
    T: Into<LibraryAction>,
{
    fn from(action: T) -> Self {
        Self::Library(action.into())
    }
}

#[derive(Debug, Clone)]
enum MusicDirectoryAction {
    Reset,
    Select,
    Selected(Option<DirPath<'static>>),
    SpawnSyncTask,
    AbortPendingSyncTask,
}

impl From<MusicDirectoryAction> for LibraryAction {
    fn from(action: MusicDirectoryAction) -> Self {
        Self::MusicDirectory(action)
    }
}

#[derive(Debug, Clone)]
enum CollectionAction {
    RefreshFromDb,
}

impl From<CollectionAction> for LibraryAction {
    fn from(action: CollectionAction) -> Self {
        Self::Collection(action)
    }
}

#[derive(Debug, Clone)]
enum TrackSearchAction {
    Search(String),
    FetchMore,
}

impl From<TrackSearchAction> for LibraryAction {
    fn from(action: TrackSearchAction) -> Self {
        Self::TrackSearch(action)
    }
}

/// App-level event
///
/// Not cloneable to prevent unintended storage. Notifications are
/// supposed to be ephemeral and should disappear after being processed.
#[derive(Debug)]
enum AppEvent {
    Input(AppInputEvent),
    Library(library::Event),
}

impl From<AppEvent> for AppMessage {
    fn from(event: AppEvent) -> Self {
        Self::Event(event)
    }
}

enum CentralPanelData {
    TrackSearch {
        // TODO: Replace string with "renderable" track item.
        track_list: Vec<String>,
    },
    MusicDirSync {
        progress_log: Vec<String>,
    },
}

/// Application model
///
/// Immutable during rendering.
#[allow(missing_debug_implementations)]
struct AppModel {
    library: Library,

    selecting_music_dir: bool,

    central_panel_data: Option<CentralPanelData>,
}

impl AppModel {
    #[must_use]
    const fn new(library: Library) -> Self {
        Self {
            library,
            selecting_music_dir: false,
            central_panel_data: None,
        }
    }
}

/// UI state
///
/// Mutable during rendering to capture user input.
#[derive(Debug, Default)]
struct AppUi {
    track_search_input: String,
}

#[allow(missing_debug_implementations)]
struct App {
    rt: tokio::runtime::Handle,

    msg_rx: mpsc::Receiver<AppMessage>,
    msg_tx: AppMessageSender,

    mdl: AppModel,
    ui: AppUi,
}

impl App {
    #[must_use]
    fn new(
        ctx: &CreationContext<'_>,
        rt: tokio::runtime::Handle,
        mdl: AppModel,
        settings_dir: PathBuf,
    ) -> Self {
        let (msg_tx, msg_rx) = mpsc::channel();
        let msg_tx = AppMessageSender::new(ctx.egui_ctx.clone(), msg_tx);
        mdl.library.spawn_background_tasks(&rt, settings_dir);
        mdl.library.spawn_event_tasks(&rt, &msg_tx);
        Self {
            rt,
            msg_rx,
            msg_tx,
            mdl,
            ui: Default::default(),
        }
    }

    fn update(&mut self) -> (&mut mpsc::Receiver<AppMessage>, AppUpdateContext<'_>) {
        let Self {
            rt,
            msg_rx,
            msg_tx,
            mdl,
            ui,
        } = self;
        let ctx = AppUpdateContext {
            rt,
            msg_tx,
            mdl,
            ui,
        };
        (msg_rx, ctx)
    }
}

struct AppUpdateContext<'a> {
    rt: &'a tokio::runtime::Handle,
    msg_tx: &'a AppMessageSender,
    mdl: &'a mut AppModel,
    ui: &'a mut AppUi,
}

impl<'a> AppUpdateContext<'a> {
    fn on_action(&mut self, action: AppAction) {
        let Self {
            rt,
            msg_tx,
            mdl,
            ui: _,
        } = self;
        match action {
            AppAction::Library(action) => match action {
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

    fn on_input_event(&mut self, input: AppInputEvent) {
        let Self { ui, .. } = self;
        match input {
            AppInputEvent::TrackSearch(input) => {
                ui.track_search_input = input;
            }
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
                                        log::debug!(
                                            "Track search memo changed: Replacing all fetched entities",
                                        );
                                        if let Some(fetched_entities) = state.fetched_entities() {
                                            track_search_list.clear();
                                            track_search_list.extend(fetched_entities.iter().map(
                                                |fetched_entity| {
                                                    track_to_string(&fetched_entity.entity.body.track)
                                                },
                                            ));
                                        } else {
                                            mdl.central_panel_data = None;
                                        }
                                    }
                                    aoide::desktop_app::track::repo_search::FetchedEntitiesDiff::Append => {
                                        let Some(fetched_entities) = state.fetched_entities() else {
                                            unreachable!();
                                        };
                                        debug_assert_eq!(
                                            Some(track_search_list.len()),
                                            last_memo_offset,
                                        );
                                        debug_assert!(track_search_list.len() <= fetched_entities.len());
                                        let num_append_entities =
                                            fetched_entities.len() - track_search_list.len();
                                        log::debug!(
                                                    "Track search memo changed: Appending {num_append_entities} fetched entities");
                                        track_search_list.extend(
                                            (track_search_list.len()..fetched_entities.len())
                                                .map(|i| format!("TODO: Track {i}")),
                                        );
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

    fn render(self) -> AppRenderContext<'a> {
        let Self {
            msg_tx, mdl, ui, ..
        } = self;
        AppRenderContext { msg_tx, mdl, ui }
    }
}

// In contrast to `AppUpdateContext` the model is immutable during rendering.
// Only the `AppUiState` remains mutable.
struct AppRenderContext<'a> {
    msg_tx: &'a AppMessageSender,
    mdl: &'a AppModel,
    ui: &'a mut AppUi,
}

impl<'a> AppRenderContext<'a> {
    #[allow(clippy::too_many_lines)] // TODO
    fn render_ui(&mut self, ctx: &Context, _frm: &mut Frame) {
        let Self {
            msg_tx,
            mdl,
            ui: app_ui,
        } = self;
        let current_library_state = mdl.library.read_lock_current_state();

        TopBottomPanel::top("top-panel").show(ctx, |ui| {
        egui::Grid::new("grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                let music_dir = current_library_state.music_dir();
                ui.label("Music directory:");
                ui.label(
                    music_dir
                        .map(|path| path.display().to_string())
                        .unwrap_or_default(),
                );
                ui.end_row();

                ui.label("");
                egui::Grid::new("grid")
                    .num_columns(3)
                    .spacing([40.0, 4.0])
                    .show(ui, |ui| {
                    if ui
                        .add_enabled(
                            !mdl.selecting_music_dir,
                            Button::new("Select music directory..."),
                        )
                        .on_hover_text("Switch collections or create a new one.")
                        .clicked()
                    {
                        msg_tx
                            .send_action(MusicDirectoryAction::Select);
                    }
                    if ui
                        .add_enabled(
                            !mdl.selecting_music_dir && current_library_state.could_synchronize_music_dir_task(),
                            Button::new("Synchronize music directory"),
                        )
                        .on_hover_text(
                            "Rescan the music directory for added/modified/deleted files and update the collection.",
                        )
                        .clicked()
                    {
                        msg_tx.send_action(MusicDirectoryAction::SpawnSyncTask);
                    }
                    if ui
                        .add_enabled(
                            !mdl.selecting_music_dir
                                && current_library_state.could_reset_music_dir(),
                            Button::new("Reset music directory"),
                        )
                        .on_hover_text("Disconnect from the corresponding collection.")
                        .clicked()
                    {
                        msg_tx
                            .send_action(MusicDirectoryAction::Reset);
                    }
                    ui.end_row();
                });
                ui.end_row();

                let collection_uid = current_library_state
                    .collection()
                    .entity_brief()
                    .map(|(entity_uid, _)| entity_uid);
                ui.label("Collection UID:");
                ui.label(
                    collection_uid
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_default(),
                );
                ui.end_row();

                let collection_title = current_library_state
                    .collection()
                    .entity_brief()
                    .and_then(|(_, collection)| {
                        collection.map(|collection| collection.title.as_str())
                    });
                ui.label("Collection title:");
                ui.label(collection_title.unwrap_or_default());
                ui.end_row();

                let collection_summary = current_library_state
                    .collection()
                    .entity_with_summary()
                    .map(|(_, summary)| summary);
                ui.label("Collection summary:");
                ui.label(collection_summary.map_or("<none>".to_owned(), |summary| {
                    format!(
                        "#tracks = {num_tracks}, #playlists = {num_playlists}",
                        num_tracks = summary.tracks.total_count,
                        num_playlists = summary.playlists.total_count
                    )
                }));
                ui.end_row();

                ui.label("Search tracks:");
                if ui
                    .add_enabled(
                        current_library_state.could_search_tracks(),
                        TextEdit::singleline(&mut app_ui.track_search_input),
                    )
                    .lost_focus()
                {
                    msg_tx.send_action(TrackSearchAction::Search(app_ui.track_search_input.clone()),
                    );
                }
                ui.end_row();
            });
    });

        if let Some(central_panel_data) = &mdl.central_panel_data {
            CentralPanel::default().show(ctx, |ui| match central_panel_data {
                CentralPanelData::TrackSearch { track_list } => {
                    for track in track_list {
                        ui.label(track);
                    }
                }
                CentralPanelData::MusicDirSync { progress_log } => {
                    for line in progress_log.iter().rev() {
                        ui.label(line);
                    }
                }
            });
        }

        TopBottomPanel::bottom("bottem-panel").show(ctx, |ui| {
            egui::Grid::new("grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    if let Some(central_panel_data) = &mdl.central_panel_data {
                        let text;
                        let hover_text;
                        let enabled;
                        let action: AppAction;
                        match central_panel_data {
                            CentralPanelData::TrackSearch { .. } => {
                                text = "Fetch more";
                                hover_text = "Fetch the next page of search results.";
                                enabled =
                                    current_library_state.could_fetch_more_track_search_results();
                                action = TrackSearchAction::FetchMore.into();
                            }
                            CentralPanelData::MusicDirSync { .. } => {
                                if current_library_state.could_abort_synchronize_music_dir_task() {
                                    text = "Abort";
                                    hover_text = "Stop the current synchronization task.";
                                    enabled = true;
                                    action = MusicDirectoryAction::AbortPendingSyncTask.into();
                                } else {
                                    text = "Dismiss";
                                    hover_text = "Clear output and return to track search.";
                                    enabled = true;
                                    action = CollectionAction::RefreshFromDb.into();
                                }
                            }
                        }
                        if ui
                            .add_enabled(enabled, Button::new(text))
                            .on_hover_text(hover_text)
                            .clicked()
                        {
                            msg_tx.send_action(action);
                        }
                        ui.end_row();
                    }

                    ui.label("Last error:");
                    let last_error = current_library_state
                        .collection()
                        .last_error()
                        .map(ToOwned::to_owned)
                        .or_else(|| {
                            current_library_state
                                .track_search()
                                .last_fetch_error()
                                .map(ToString::to_string)
                        });
                    if let Some(last_error) = last_error.as_deref() {
                        ui.label(last_error);
                    }
                    ui.end_row();
                });
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frm: &mut Frame) {
        let (msg_rx, mut update_ctx) = self.update();
        loop {
            match msg_rx.try_recv() {
                Ok(msg) => {
                    log::debug!("Received message: {msg:?}");
                    match msg {
                        AppMessage::Action(action) => update_ctx.on_action(action),
                        AppMessage::Event(event) => match event {
                            AppEvent::Input(input) => update_ctx.on_input_event(input),
                            AppEvent::Library(event) => update_ctx.on_library_event(event),
                        },
                    };
                }
                Err(mpsc::TryRecvError::Empty) => {
                    break;
                }
                Err(mpsc::TryRecvError::Disconnected) => unreachable!(),
            }
        }

        let mut render_ctx = update_ctx.render();
        render_ctx.render_ui(ctx, frm);
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

/// Open a file dialog to choose a directory path
///
/// Start with the given path if available.
///
/// Returns `Some` if a path has been chosen and `None` otherwise.
fn choose_directory_path<P>(
    rt: &tokio::runtime::Handle,
    dir_path: Option<&P>,
    on_dir_path_chosen: impl FnOnce(Option<DirPath<'static>>) + Send + Sync + 'static,
) -> Option<PathBuf>
where
    P: AsRef<Path>,
{
    let dir_path = dir_path.as_ref().map(AsRef::as_ref).map(PathBuf::from);
    rt.spawn(async move {
        let dir_path = aoide::desktop_app::fs::choose_directory(dir_path.as_deref()).await;
        on_dir_path_chosen(dir_path);
    });
    None
}
