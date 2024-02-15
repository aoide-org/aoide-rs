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
use egui::{CentralPanel, Context, TopBottomPanel};

use aoide::desktop_app::{
    collection::State as CollectionState, fs::DirPath, ObservableReader as _,
};

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

    fn on_action(&self, action: AppAction) {
        if let Err(NoReceiverForAppMessage(msg)) = self.send_message(AppMessage::Action(action)) {
            let AppMessage::Action(action) = msg else {
                unreachable!()
            };
            log::warn!("No receiver for action: {action:?}");
        }
    }

    fn emit_event(&self, event: AppEvent) -> Result<(), NoReceiverForEvent> {
        if let Err(NoReceiverForAppMessage(msg)) = self.send_message(AppMessage::Event(event)) {
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
const fn disable_rescan_collection(state: &CollectionState) -> bool {
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

#[derive(Debug, Clone)]
enum AppAction {
    SelectMusicDirectory,
    UpdateMusicDirectory(DirPath<'static>),
    ResetMusicDirectory,
    RescanCollection,
    SearchTracks,
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

#[allow(missing_debug_implementations)]
struct App {
    rt: tokio::runtime::Handle,

    message_rx: mpsc::Receiver<AppMessage>,
    message_sender: AppMessageSender,

    library: Library,

    music_dir: Option<DirPath<'static>>,
    collection_state: CollectionState,
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
            collection_state: Default::default(),
            track_search_input: Default::default(),
            track_search_memo: Default::default(),
            track_search_list: Default::default(),
        }
    }

    fn on_action(&mut self, action: AppAction) {
        match action {
            AppAction::SelectMusicDirectory => {
                let on_dir_path_chosen = {
                    let message_sender = self.message_sender.clone();
                    move |dir_path: DirPath<'static>| {
                        message_sender.on_action(AppAction::UpdateMusicDirectory(dir_path));
                    }
                };
                choose_directory_path(&self.rt, &self.music_dir.as_deref(), on_dir_path_chosen);
            }
            AppAction::UpdateMusicDirectory(music_dir) => {
                self.library.update_music_dir(Some(&music_dir));
            }
            AppAction::ResetMusicDirectory => {
                self.library.reset_music_dir();
            }
            AppAction::RescanCollection => {
                self.library.spawn_rescan_collection_task(&self.rt);
            }
            AppAction::SearchTracks => {
                self.library.search_tracks(&self.track_search_input);
            }
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
    fn on_library_event(&mut self, event: &LibraryEvent) {
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
                self.collection_state = new_state;
            }
            LibraryEvent::TrackSearchStateChanged => {
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
                };
            }
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
                            self.on_action(action);
                        }
                        AppMessage::Event(event) => match event {
                            AppEvent::Input(input) => {
                                self.on_input_event(input.clone());
                            }
                            AppEvent::Library(event) => {
                                self.on_library_event(&event);
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

        let message_sender = self.message_sender.clone();
        let settings_state = self.library.state().settings().read_observable();
        let collection_state = self.library.state().collection().read_observable();
        let track_search_state = self.library.state().track_search().read_observable();

        TopBottomPanel::top("config_panel").show(ctx, |ui| {
            egui::Grid::new("config_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    let music_dir = settings_state.music_dir.as_ref();
                    ui.label("Music directory:");
                    ui.label(
                        music_dir
                            .map(|path| path.display().to_string())
                            .unwrap_or_default(),
                    );
                    ui.end_row();

                    ui.label("");
                    if ui.button("Select music directory...").clicked() {
                        message_sender.on_action(AppAction::SelectMusicDirectory);
                    }
                    ui.label("");
                    if ui.button("Reset music directory").clicked() {
                        message_sender.on_action(AppAction::ResetMusicDirectory);
                    }
                    ui.end_row();

                    let collection_uid = collection_state
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

                    let collection_title =
                        collection_state.entity_brief().and_then(|(_, collection)| {
                            collection.map(|collection| collection.title.as_str())
                        });
                    ui.label("Collection title:");
                    ui.label(collection_title.unwrap_or_default());
                    ui.end_row();

                    let collection_summary = collection_state
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
                    if ui.button("Rescan collection").clicked() {
                        message_sender.on_action(AppAction::RescanCollection);
                    }
                    ui.end_row();

                    ui.label("Search tracks:");
                    if ui
                        .text_edit_singleline(&mut self.track_search_input)
                        .lost_focus()
                    {
                        message_sender.on_action(AppAction::SearchTracks);
                    }
                    ui.end_row();
                });
        });

        CentralPanel::default().show(ctx, |ui| {
            for track in self.track_search_list.as_deref().unwrap_or_default() {
                ui.label(track);
            }
        });

        TopBottomPanel::bottom("status_panel").show(ctx, |ui| {
            egui::Grid::new("config_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Last error:");
                    let last_error = collection_state
                        .last_error()
                        .map(ToOwned::to_owned)
                        .or_else(|| {
                            track_search_state
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
    on_dir_path_chosen: impl FnOnce(DirPath<'static>) + Send + Sync + 'static,
) -> Option<PathBuf>
where
    P: AsRef<Path>,
{
    let dir_path = dir_path.as_ref().map(AsRef::as_ref).map(PathBuf::from);
    rt.spawn(async move {
        let dir_path = aoide::desktop_app::fs::choose_directory(dir_path.as_deref()).await;
        let Some(dir_path) = dir_path else {
            return;
        };
        on_dir_path_chosen(dir_path);
    });
    None
}
