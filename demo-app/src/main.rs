// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

// Required for `#[derive(Lens)]`!?
#![allow(clippy::expl_impl_clone_on_copy)]
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

use aoide::desktop_app::{collection, fs::DirPath, ObservableReader as _};

mod library;
#[allow(unused_imports)]
use self::library::{Library, LibraryEvent, LibraryEventEmitter, LibraryState};

#[derive(Debug)]
struct NoReceiverForEvent;

#[derive(Debug)]
struct NoReceiverForAppMessage(pub AppMessage);

#[allow(missing_debug_implementations)]
#[derive(Clone)]
struct AppMessageSender {
    ctx: Context,
    message_tx: mpsc::Sender<AppMessage>,
}

impl AppMessageSender {
    const fn new(ctx: Context, message_tx: mpsc::Sender<AppMessage>) -> Self {
        Self { ctx, message_tx }
    }

    fn on_action<T>(&self, action: T)
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
        self.message_tx.send(msg).map_err(|err| {
            log::warn!("Failed to send message: {err}");
            NoReceiverForAppMessage(err.0)
        })?;
        // Queued messages are consumed before rendering the next frame.
        self.ctx.request_repaint();
        Ok(())
    }
}

impl LibraryEventEmitter for AppMessageSender {
    fn emit_event(&self, event: LibraryEvent) -> Result<(), NoReceiverForEvent> {
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
            let app = App::new(ctx, rt, library, config_dir);
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
    Library(LibraryEvent),
}

impl From<AppEvent> for AppMessage {
    fn from(event: AppEvent) -> Self {
        Self::Event(event)
    }
}

#[allow(missing_debug_implementations)]
struct App {
    rt: tokio::runtime::Handle,

    message_rx: mpsc::Receiver<AppMessage>,
    message_sender: AppMessageSender,

    library: Library,

    music_dir: Option<DirPath<'static>>,
    selecting_music_dir: bool,

    collection_state: collection::State,
    track_search_input: String,
    track_search_memo: aoide::desktop_app::track::repo_search::Memo,
    // TODO: Replace string with "renderable" track item.
    track_search_list: Option<Vec<String>>,
}

impl App {
    #[must_use]
    fn new(
        ctx: &CreationContext<'_>,
        rt: tokio::runtime::Handle,
        library: Library,
        settings_dir: PathBuf,
    ) -> Self {
        let (message_tx, message_rx) = mpsc::channel();
        let message_sender = AppMessageSender::new(ctx.egui_ctx.clone(), message_tx);
        library.spawn_background_tasks(&rt, settings_dir);
        library.spawn_event_tasks(&rt, &message_sender);
        Self {
            rt,
            message_sender,
            message_rx,
            library,
            music_dir: Default::default(),
            selecting_music_dir: false,
            collection_state: Default::default(),
            track_search_input: Default::default(),
            track_search_memo: Default::default(),
            track_search_list: Default::default(),
        }
    }

    fn on_action(&mut self, ctx: &Context, action: AppAction) {
        match action {
            AppAction::Library(action) => match action {
                LibraryAction::MusicDirectory(action) => match action {
                    MusicDirectoryAction::Reset => {
                        self.library.reset_music_dir();
                    }
                    MusicDirectoryAction::Select => {
                        if self.selecting_music_dir {
                            log::debug!("Already selecting music directory");
                            return;
                        }
                        let on_dir_path_chosen = {
                            let message_sender = self.message_sender.clone();
                            move |dir_path| {
                                message_sender.on_action(MusicDirectoryAction::Selected(dir_path));
                            }
                        };
                        choose_directory_path(
                            &self.rt,
                            &self.music_dir.as_deref(),
                            on_dir_path_chosen,
                        );
                        self.selecting_music_dir = true;
                        // Reflect the state change in the UI.
                        ctx.request_repaint();
                    }
                    MusicDirectoryAction::Selected(music_dir) => {
                        self.selecting_music_dir = false;
                        if let Some(music_dir) = music_dir {
                            self.library.update_music_dir(Some(&music_dir));
                        }
                        // Reflect the state change in the UI.
                        ctx.request_repaint();
                    }
                    MusicDirectoryAction::SpawnSyncTask => {
                        if self.library.spawn_synchronize_music_dir_task(&self.rt) {
                            // Reflect the state change in the UI.
                            ctx.request_repaint();
                        }
                    }
                    MusicDirectoryAction::AbortPendingSyncTask => {
                        if self.library.abort_pending_synchronize_music_dir_task() {
                            // Reflect the state change in the UI.
                            ctx.request_repaint();
                        }
                    }
                },
                LibraryAction::Collection(action) => match action {
                    CollectionAction::RefreshFromDb => {
                        self.library.refresh_collection_from_db(&self.rt);
                    }
                },
                LibraryAction::TrackSearch(action) => match action {
                    TrackSearchAction::Search(input) => {
                        self.library.search_tracks(&input);
                    }
                    TrackSearchAction::FetchMore => {
                        self.library
                            .fetch_more_track_search_results(&self.rt, &self.message_sender);
                    }
                },
            },
        }
    }

    fn on_input_event(&mut self, input: AppInputEvent) {
        match input {
            AppInputEvent::TrackSearch(input) => {
                self.track_search_input = input.clone();
            }
        }
    }

    #[allow(clippy::too_many_lines)] // TODO
    fn on_library_event(&mut self, ctx: &Context, event: LibraryEvent) {
        match event {
            LibraryEvent::SettingsStateChanged => {
                let new_music_dir = {
                    let settings_state = self.library.state().settings().read_observable();
                    if settings_state.music_dir == self.music_dir {
                        log::debug!(
                            "Music directory unchanged: {music_dir:?}",
                            music_dir = self.music_dir,
                        );
                        return;
                    }
                    settings_state.music_dir.clone()
                };
                log::debug!(
                    "Music directory changed: {old_music_dir:?} -> {new_music_dir:?}",
                    old_music_dir = self.music_dir,
                );
                self.music_dir = new_music_dir;
            }
            LibraryEvent::CollectionStateChanged => {
                let new_state = {
                    let new_state = self.library.state().collection().read_observable();
                    if *new_state == self.collection_state {
                        log::debug!(
                            "Collection state unchanged: {old_state:?}",
                            old_state = self.collection_state,
                        );
                        return;
                    }
                    new_state.clone()
                };
                log::debug!(
                    "Collection state changed: {old_state:?} -> {new_state:?}",
                    old_state = self.collection_state,
                );
                if self
                    .library
                    .on_collection_state_changed(&self.rt, &new_state)
                {
                    // Reflect the state change in the UI.
                    ctx.request_repaint();
                }
                self.collection_state = new_state;
            }
            LibraryEvent::TrackSearch(event) => match event {
                library::track_search::Event::StateChanged => {
                    debug_assert_eq!(
                        self.track_search_list.as_ref().map(Vec::len),
                        self.track_search_memo
                            .fetch
                            .fetched_entities
                            .as_ref()
                            .map(|memo| memo.offset)
                    );
                    let state = self.library.state().track_search().read_observable();
                    let memo_updated = state.update_memo(&mut self.track_search_memo);
                    match memo_updated {
                        aoide::desktop_app::track::repo_search::MemoUpdated::Unchanged => {
                            log::debug!("Track search memo unchanged",);
                        }
                        aoide::desktop_app::track::repo_search::MemoUpdated::Changed {
                            fetched_entities_diff,
                        } => match fetched_entities_diff {
                            aoide::desktop_app::track::repo_search::FetchedEntitiesDiff::Replace => {
                                log::debug!(
                                    "Track search memo changed: Replacing all fetched entities",
                                );
                                if let Some(fetched_entities) = state.fetched_entities() {
                                    let mut track_search_list =
                                        self.track_search_list.take().unwrap_or_default();
                                    track_search_list.clear();
                                    track_search_list.extend(fetched_entities.iter().map(
                                        |fetched_entity| {
                                            track_to_string(&fetched_entity.entity.body.track)
                                        },
                                    ));
                                    self.track_search_list = Some(track_search_list);
                                } else {
                                    self.track_search_list = None;
                                }
                            }
                            aoide::desktop_app::track::repo_search::FetchedEntitiesDiff::Append => {
                                let Some(fetched_entities) = state.fetched_entities() else {
                                    unreachable!();
                                };
                                let track_search_list = self.track_search_list.as_mut().unwrap();
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
                        },
                    }
                }
                library::track_search::Event::FetchMoreTaskCompleted {
                    result,
                    continuation,
                } => {
                    self.library
                        .track_search_fetch_more_task_completed(result, continuation);
                }
            },
        }
    }
}

impl eframe::App for App {
    #[allow(clippy::too_many_lines)] // TODO
    fn update(&mut self, ctx: &Context, _frm: &mut Frame) {
        loop {
            match self.message_rx.try_recv() {
                Ok(msg) => {
                    log::debug!("Received message: {msg:?}");
                    match msg {
                        AppMessage::Action(action) => {
                            self.on_action(ctx, action);
                        }
                        AppMessage::Event(event) => match event {
                            AppEvent::Input(input) => {
                                self.on_input_event(input.clone());
                            }
                            AppEvent::Library(event) => {
                                self.on_library_event(ctx, event);
                            }
                        },
                    }
                }
                Err(mpsc::TryRecvError::Empty) => {
                    break;
                }
                Err(mpsc::TryRecvError::Disconnected) => unreachable!(),
            }
        }

        let message_sender = &self.message_sender;
        let current_library_state = self.library.state().read_current();

        TopBottomPanel::top("top-panel").show(ctx, |ui| {
            egui::Grid::new("grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    let music_dir = current_library_state.settings().music_dir.as_ref();
                    ui.label("Music directory:");
                    ui.label(
                        music_dir
                            .map(|path| path.display().to_string())
                            .unwrap_or_default(),
                    );
                    ui.end_row();

                    ui.label("");
                    if ui
                        .add_enabled(
                            !self.selecting_music_dir,
                            Button::new("Select music directory..."),
                        )
                        .on_hover_text("Switch collections or create a new one.")
                        .clicked()
                    {
                        message_sender
                            .on_action(MusicDirectoryAction::Select);
                    }
                    ui.label("");
                    if ui
                        .add_enabled(
                            !self.selecting_music_dir
                                && current_library_state.could_reset_music_dir(),
                            Button::new("Reset music directory"),
                        )
                        .on_hover_text("Disconnect from the corresponding collection.")
                        .clicked()
                    {
                        message_sender
                            .on_action(MusicDirectoryAction::Reset);
                    }
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

                    ui.label("");
                    if current_library_state.could_abort_synchronize_music_dir_task() {
                        debug_assert!(
                            !current_library_state.could_spawn_synchronize_music_dir_task()
                        );
                        if ui
                            .button("Abort synchronize music directory")
                            .on_hover_text("Stop the current synchronization task.")
                            .clicked()
                        {
                            message_sender.on_action(MusicDirectoryAction::AbortPendingSyncTask);
                        }
                    } else if ui
                        .add_enabled(
                            current_library_state.could_spawn_synchronize_music_dir_task(),
                            Button::new("Synchronize music directory..."),
                        )
                        .on_hover_text(
                            "Rescan the music directory for added/modified/deleted files and update the collection.",
                        )
                        .clicked()
                    {
                        message_sender.on_action(MusicDirectoryAction::SpawnSyncTask);
                    }
                    ui.end_row();

                    ui.label("Search tracks:");
                    if ui
                        .add_enabled(
                            current_library_state.could_search_tracks(),
                            TextEdit::singleline(&mut self.track_search_input),
                        )
                        .lost_focus()
                    {
                        message_sender.on_action(TrackSearchAction::Search(self.track_search_input.clone()),
                        );
                    }
                    ui.end_row();
                });
        });

        CentralPanel::default().show(ctx, |ui| {
            for track in self.track_search_list.as_deref().unwrap_or_default() {
                ui.label(track);
            }
        });

        TopBottomPanel::bottom("bottem-panel").show(ctx, |ui| {
            egui::Grid::new("grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    if ui
                        .add_enabled(
                            current_library_state.could_spawn_fetch_more_track_search_results(),
                            Button::new("Fetch more"),
                        )
                        .clicked()
                    {
                        message_sender.on_action(TrackSearchAction::FetchMore);
                    }
                    ui.end_row();

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
#[allow(clippy::unused_self)] // TODO
fn choose_directory_path<P>(
    rt: &tokio::runtime::Handle,
    dir_path: &Option<P>,
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
