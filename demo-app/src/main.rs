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

use aoide::desktop_app::fs::DirPath;

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
#[allow(clippy::too_many_lines)] // TODO
async fn main() {
    pretty_env_logger::init();

    let rt = match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle,
        Err(err) => {
            log::error!("No Tokio runtime: {err}");
            return;
        }
    };

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

    Application::new(move |cx: &mut Context| {
        let mdl = AppModel::new(App::new(
            rt,
            Library::new(aoide_handle, aoide_initial_settings),
        ));
        mdl.build(config_dir, cx);

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
                );
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
                AppModel::collection_with_summary.map(|collection_with_summary| {
                    if let Some(collection_with_summary) = &collection_with_summary {
                        if let Some(summary) = &collection_with_summary.summary {
                            format!(
                                "Collection: uid = {uid}, title = \"{title}\", #tracks = \
                             {tracks_count}, #playlists = {playlists_count}",
                                uid = collection_with_summary.entity.hdr.uid,
                                title = collection_with_summary.entity.body.title,
                                tracks_count = summary.tracks.total_count,
                                playlists_count = summary.playlists.total_count,
                            )
                        } else {
                            format!(
                                "Collection: uid = {uid}, title = \"{title}\"",
                                uid = collection_with_summary.entity.hdr.uid,
                                title = collection_with_summary.entity.body.title,
                            )
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
                .disabled(AppModel::collection_state_tag.map(|state_tag| !state_tag.is_ready()));
                Button::new(
                    cx,
                    |ex| ex.emit(AppEvent::Command(AppCommand::ResetCollection)),
                    |cx| Label::new(cx, "Reset collection"),
                )
                .disabled(AppModel::collection_state_tag.map(|state_tag| !state_tag.is_ready()));
            });
        });
    })
    .title(app_name())
    .run();
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
    Command(AppCommand),
    Notification(AppNotification),
}

#[derive(Debug, Clone)]
enum AppCommand {
    SelectMusicDirectory,
    ResetMusicDirectory,
    ResetCollection,
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
    event_emitter_keepalive: Option<Arc<AppEventEmitter>>,
}

impl App {
    #[must_use]
    const fn new(rt: tokio::runtime::Handle, library: Library) -> Self {
        Self {
            rt,
            library,
            event_emitter_keepalive: None,
        }
    }
}

#[derive(Lens)]
#[allow(missing_debug_implementations)]
struct AppModel {
    app: App,
    music_dir: Option<DirPath<'static>>,
    collection_state_tag: aoide::desktop_app::collection::StateTag,
    collection_with_summary: Option<aoide::api::collection::EntityWithSummary>,
}

impl AppModel {
    fn new(app: App) -> Self {
        Self {
            app,
            music_dir: None,
            collection_state_tag: Default::default(),
            collection_with_summary: None,
        }
    }

    fn build(mut self, settings_dir: PathBuf, cx: &mut Context) {
        self.app
            .library
            .spawn_background_tasks(&self.app.rt, settings_dir);
        let event_emitter = Arc::new(AppEventEmitter {
            cx: Mutex::new(cx.get_proxy()),
        });
        self.app
            .library
            .spawn_notification_tasks(&self.app.rt, &event_emitter);
        // Keep the event emitter alive while the application is running.
        self.app.event_emitter_keepalive = Some(event_emitter);
        <Self as Model>::build(self, cx);
    }

    /// Open a file dialog to choose a directory path
    ///
    /// Start with the given path if available.
    ///
    /// Returns `Some` if a path has been chosen and `None` otherwise.
    #[allow(clippy::unused_self)] // TODO
    fn choose_directory_path<P>(&self, dir_path: &Option<P>) -> Option<PathBuf>
    where
        P: AsRef<Path>,
    {
        let dir_path = dir_path.as_ref().map(AsRef::as_ref);
        log::warn!("TODO: Open file dialog to choose directory path: {dir_path:?}",);
        None
    }
}

impl Model for AppModel {
    fn name(&self) -> Option<&'static str> {
        Some(app_name())
    }

    fn event(&mut self, _cx: &mut EventContext<'_>, event: &mut Event) {
        event.map(|event, _meta| match event {
            AppEvent::Command(command) => match command {
                AppCommand::SelectMusicDirectory => {
                    if let Some(music_dir) = self.choose_directory_path(&self.music_dir.as_deref())
                    {
                        self.app.library.update_music_directory(Some(music_dir));
                    }
                }
                AppCommand::ResetMusicDirectory => {
                    self.app.library.reset_music_directory();
                }
                AppCommand::ResetCollection => {
                    self.app.library.reset_collection();
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
                    LibraryNotification::SettingsStateChanged(state) => {
                        if state.music_dir == self.music_dir {
                            log::info!(
                                "Music directory unchanged: {music_dir:?}",
                                music_dir = self.music_dir
                            );
                            return;
                        }
                        let old_music_dir = self.music_dir.take();
                        self.music_dir = state.music_dir.clone();
                        log::info!(
                            "Music directory changed: {old_music_dir:?} -> {new_music_dir:?}",
                            new_music_dir = self.music_dir
                        );
                    }
                    LibraryNotification::CollectionStateChanged(state) => {
                        self.collection_state_tag = state.state_tag();
                        let new_collection_with_summary = state.entity_with_summary();
                        if new_collection_with_summary == self.collection_with_summary.as_ref() {
                            log::info!(
                                "Collection unchanged: {collection_with_summary:?}",
                                collection_with_summary = self.collection_with_summary
                            );
                            return;
                        }
                        let old_collection_with_summary = self.collection_with_summary.take();
                        self.collection_with_summary = new_collection_with_summary.cloned();
                        log::info!(
                            "Collection changed: {old_collection_with_summary:?} -> \
                             {new_collection_with_summary:?}",
                            new_collection_with_summary = self.collection_with_summary
                        );
                    }
                    LibraryNotification::TrackSearchStateChanged(reader) => {
                        log::warn!("TODO: Track search state changed: {reader:?}");
                    }
                },
            },
        });
    }
}
