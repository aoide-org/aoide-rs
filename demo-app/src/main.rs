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
        let app = App {
            library: Library::new(aoide_handle, aoide_initial_settings),
        };
        let mdl = AppModel::new(app, config_dir);
        mdl.build(&rt, cx);

        Label::new(
            cx,
            AppModel::config_dir.map(|config_dir| {
                format!(
                    "Config directory: {config_dir}",
                    config_dir = config_dir.display()
                )
            }),
        );

        Label::new(
            cx,
            AppModel::music_dir.map(|music_dir| format!("Music directory: {music_dir:?}")),
        );

        Label::new(
            cx,
            AppModel::collection_entity
                .map(|collection_entity| format!("Collection entity: {collection_entity:?}")),
        );
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

#[derive(Debug, Clone)]
enum AppEvent {
    Command(AppCommand),
    Notification(AppNotification),
}

#[derive(Debug, Clone)]
enum AppCommand {
    Quit,
}

#[derive(Debug, Clone)]
enum AppNotification {
    Library(LibraryNotification),
}

#[allow(missing_debug_implementations)]
struct App {
    library: Library,
}

#[derive(Lens)]
#[allow(missing_debug_implementations)]
struct AppModel {
    app: App,
    config_dir: PathBuf,
    music_dir: Option<PathBuf>,
    collection_entity: Option<aoide::collection::Entity>,
}

impl AppModel {
    const fn new(app: App, config_dir: PathBuf) -> Self {
        Self {
            app,
            config_dir,
            music_dir: None,
            collection_entity: None,
        }
    }

    fn build(self, rt: &tokio::runtime::Handle, cx: &mut Context) {
        self.app
            .library
            .spawn_background_tasks(rt, self.config_dir.clone());
        let event_emitter = Arc::new(AppEventEmitter {
            cx: Mutex::new(cx.get_proxy()),
        });
        self.app
            .library
            .spawn_notification_tasks(rt, &event_emitter);
        <Self as Model>::build(self, cx);
    }
}

impl Model for AppModel {
    fn event(&mut self, _cx: &mut EventContext<'_>, event: &mut Event) {
        event.map(|event, _meta| match event {
            AppEvent::Command(command) => match command {
                AppCommand::Quit => {
                    log::warn!("TODO: Quit");
                }
            },
            AppEvent::Notification(notification) => match notification {
                AppNotification::Library(library) => match library {
                    LibraryNotification::MusicDirectoryChanged(music_dir) => {
                        if *music_dir == self.music_dir {
                            log::info!("Music directory unchanged: {music_dir:?}");
                            return;
                        }
                        let old_music_dir = self.music_dir.take();
                        self.music_dir = music_dir.clone();
                        log::info!(
                            "Music directory changed: {old_music_dir:?} -> {new_music_dir:?}",
                            new_music_dir = self.music_dir
                        );
                    }
                    LibraryNotification::CollectionEntityChanged(collection) => {
                        if *collection == self.collection_entity {
                            log::info!("Collection unchanged: {collection:?}");
                            return;
                        }
                        let old_collection_entity = self.collection_entity.take();
                        self.collection_entity = collection.clone();
                        log::info!(
                            "Collection changed: {old_collection_entity:?} -> \
                             {new_collection_entity:?}",
                            new_collection_entity = self.collection_entity
                        );
                    }
                },
            },
        });
    }
}
