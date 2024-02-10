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
    sync::{Arc, Mutex},
};

use directories::ProjectDirs;
use vizia::prelude::*;

use aoide::desktop_app::{
    collection::State as CollectionState, fs::DirPath, ObservableReader as _,
};

mod library;
#[allow(unused_imports)]
use self::library::{Library, LibraryEventEmitter, LibraryNotification, LibraryState};

#[allow(missing_debug_implementations)]
struct AppEventEmitter {
    cx: Mutex<ContextProxy>,
}

impl AppEventEmitter {
    fn new(cx: &Context) -> Self {
        let cx = cx.get_proxy();
        Self { cx: Mutex::new(cx) }
    }

    fn emit(&self, event: AppEvent) {
        let mut cx = self.cx.lock().unwrap();
        log::debug!("Emitting {event:?}");
        if let Err(err) = cx.emit(event) {
            log::warn!("Failed to emit event: {err}");
        }
    }
}

impl LibraryEventEmitter for AppEventEmitter {
    fn emit_notification(&self, notification: LibraryNotification) {
        let event = AppEvent::Notification(AppNotification::Library(notification));
        self.emit(event);
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

    run_app(rt, library, config_dir);
}

fn run_app(rt: tokio::runtime::Handle, library: Library, config_dir: PathBuf) {
    Application::new(move |cx: &mut Context| {
        let app = App::new(rt, library, config_dir, cx.get_proxy());
        let mdl = AppModel::new(app);
        mdl.build(cx);

        // Music directory
        VStack::new(cx, |cx| {
            Label::new(
                cx,
                AppModel::music_dir.map(|music_dir| {
                    if let Some(music_dir) = &music_dir {
                        format!(
                            "Music directory: {music_dir}",
                            music_dir = music_dir.display()
                        )
                    } else {
                        "Music directory: <none>".to_owned()
                    }
                }),
            );
            HStack::new(cx, |cx| {
                Button::new(
                    cx,
                    |ex| ex.emit(AppEvent::Command(AppCommand::SelectMusicDirectory)),
                    |cx| Label::new(cx, "Select music directory..."),
                )
                .disabled(AppModel::collection_state.map(disable_select_music_dir));
                Button::new(
                    cx,
                    |ex| ex.emit(AppEvent::Command(AppCommand::ResetMusicDirectory)),
                    |cx| Label::new(cx, "Reset music directory"),
                )
                .disabled(AppModel::music_dir.map(Option::is_none));
            });
        });

        // Collection
        VStack::new(cx, |cx| {
            Label::new(
                cx,
                AppModel::collection_state.map(|collection_state| {
                    if let Some((entity, summary)) = collection_state.entity_with_summary() {
                        format!(
                            "Collection: uid = {uid}, title = \"{title}\", #tracks = \
                                    {tracks_count}, #playlists = {playlists_count}",
                            uid = entity.hdr.uid,
                            title = entity.body.title,
                            tracks_count = summary.tracks.total_count,
                            playlists_count = summary.playlists.total_count,
                        )
                    } else if let Some((entity_uid, collection)) = collection_state.entity_brief() {
                        if let Some(collection) = collection {
                            format!(
                                "Collection: uid = {entity_uid}, title = \"{title}\"",
                                entity_uid = entity_uid,
                                title = collection.title,
                            )
                        } else {
                            format!("Collection: uid = {entity_uid}")
                        }
                    } else {
                        "Collection: <none>".to_owned()
                    }
                }),
            );

            HStack::new(cx, |cx| {
                Button::new(
                    cx,
                    |ex| ex.emit(AppEvent::Command(AppCommand::RescanCollection)),
                    |cx: &mut Context| Label::new(cx, "Rescan collection"),
                )
                .disabled(AppModel::collection_state.map(disable_rescan_collection));
            });
        });

        // Track search
        VStack::new(cx, |cx| {
            HStack::new(cx, |cx| {
                Label::new(cx, "Search tracks:");
                Textbox::new(cx, AppModel::track_search_input)
                    .on_edit(move |cx, text| {
                        cx.emit(AppEvent::Input(AppInput::TrackSearch(text.clone())));
                    })
                    .on_submit(|cx, text, enter_key_pressed| {
                        if enter_key_pressed {
                            cx.emit(AppEvent::Command(AppCommand::SearchTracks(text.clone())));
                        }
                    })
                    .width(Pixels(200.0));
            });
        });
    })
    .title(app_name())
    .run();
}

#[must_use]
const fn disable_select_music_dir(state: &CollectionState) -> bool {
    state.is_pending()
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
enum AppEvent {
    Input(AppInput),
    Command(AppCommand),
    Notification(AppNotification),
}

#[derive(Debug, Clone)]
enum AppInput {
    TrackSearch(String),
}

#[derive(Debug, Clone)]
enum AppCommand {
    SelectMusicDirectory,
    UpdateMusicDirectory(DirPath<'static>),
    ResetMusicDirectory,
    RescanCollection,
    SearchTracks(String),
}

/// App-level notification
///
/// Not cloneable to prevent unintended storage. Notifications are
/// supposed to be ephemeral and should disappear after being processed.
#[derive(Debug)]
enum AppNotification {
    Library(LibraryNotification),
}

#[allow(missing_debug_implementations)]
struct App {
    rt: tokio::runtime::Handle,
    library: Library,
    event_emitter: Arc<AppEventEmitter>,
}

impl App {
    #[must_use]
    fn new(
        rt: tokio::runtime::Handle,
        library: Library,
        settings_dir: PathBuf,
        cx: ContextProxy,
    ) -> Self {
        let event_emitter = Arc::new(AppEventEmitter { cx: Mutex::new(cx) });
        library.spawn_background_tasks(&rt, settings_dir);
        library.spawn_notification_tasks(&rt, &event_emitter);
        Self {
            rt,
            library,
            event_emitter,
        }
    }
}

#[derive(Lens)]
#[allow(missing_debug_implementations)]
struct AppModel {
    app: App,
    music_dir: Option<DirPath<'static>>,
    collection_state: CollectionState,
    track_search_input: String,
    track_search_state: aoide::desktop_app::track::repo_search::StateLite,
}

impl AppModel {
    fn new(app: App) -> Self {
        Self {
            app,
            music_dir: Default::default(),
            collection_state: Default::default(),
            track_search_input: Default::default(),
            track_search_state: Default::default(),
        }
    }
}

impl Model for AppModel {
    fn name(&self) -> Option<&'static str> {
        Some(app_name())
    }

    fn event(&mut self, _cx: &mut EventContext<'_>, event: &mut Event) {
        event.map(|event, _meta| match event {
            AppEvent::Input(input) => match input {
                AppInput::TrackSearch(input) => {
                    self.track_search_input = input.clone();
                }
            },
            AppEvent::Command(command) => match command {
                AppCommand::SelectMusicDirectory => {
                    let on_dir_path_chosen = {
                        // TODO: Obtain an `EmitContext` from `cx` once it is available.
                        let event_emitter = Arc::downgrade(&self.app.event_emitter);
                        move |dir_path: DirPath<'static>| {
                            let Some(event_emitter) = event_emitter.upgrade() else {
                                log::error!("No event emitter available after choosing music directory {dir_path}",
                                dir_path = dir_path.display());
                                return;
                            };
                            event_emitter.emit(AppEvent::Command(
                                AppCommand::UpdateMusicDirectory(dir_path),
                            ));
                        }
                    };
                    choose_directory_path(
                        &self.app.rt,
                        &self.music_dir.as_deref(),
                        on_dir_path_chosen,
                    );
                }
                AppCommand::UpdateMusicDirectory(music_dir) => {
                    self.app.library.update_music_dir(Some(music_dir));
                }
                AppCommand::ResetMusicDirectory => {
                    self.app.library.reset_music_dir();
                }
                AppCommand::RescanCollection => {
                    self.app.library.spawn_rescan_collection_task(&self.app.rt);
                }
                AppCommand::SearchTracks(input) => {
                    self.app.library.search_tracks(input);
                }
            },
            AppEvent::Notification(notification) => match notification {
                AppNotification::Library(library) => match library {
                    LibraryNotification::SettingsStateChanged => {
                        let new_music_dir = {
                            let settings_state = self.app.library.state().settings().read_observable();
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
                    LibraryNotification::CollectionStateChanged => {
                        let new_state = {
                            let new_state = self.app.library.state().collection().read_observable();
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
                    LibraryNotification::TrackSearchStateChanged => {
                        let new_state = {
                            let new_state = self.app.library.state().track_search().read_observable();
                            if new_state.equals_lite(&self.track_search_state) {
                                log::debug!(
                                    "Track search state unchanged: {old_state:?}",
                                    old_state = self.track_search_state,
                                );
                                return;
                            }
                            new_state.clone_lite()
                        };
                        log::debug!(
                            "Track search state changed: {old_state:?} -> {new_state:?}",
                            old_state = self.track_search_state,
                        );
                        self.track_search_state = new_state;
                        log::warn!("TODO: Show fetched entities");
                    }
                },
            },
        });
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
