// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    fs,
    future::Future,
    path::{Path, PathBuf},
};

use discro::{new_pubsub, Publisher, Ref, Subscriber};
use serde::{Deserialize, Serialize};
use url::Url;

use aoide_backend_embedded::storage::DatabaseConfig;

use crate::fs::{DirPath, OwnedDirPath};

pub const FILE_NAME: &str = "aoide_desktop_settings";

pub const FILE_SUFFIX: &str = "ron";

pub const DEFAULT_DATABASE_FILE_NAME: &str = "aoide";

pub const DEFAULT_DATABASE_FILE_SUFFIX: &str = "sqlite";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    /// File path of the SQLite database.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_url: Option<Url>,

    /// The root music directory.
    ///
    /// Used as to select the corresponding collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub music_dir: Option<OwnedDirPath>,

    /// Filter for a collection kind.
    ///
    /// If set only collections of this kind should be considered
    /// by an application and all other collections should be
    /// ignored.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_kind: Option<String>,
}

impl Settings {
    pub fn load(parent_dir: &Path) -> anyhow::Result<Settings> {
        let file_path = new_settings_file_path(parent_dir.to_path_buf());
        log::info!("Loading settings from file: {}", file_path.display());
        match fs::read(&file_path) {
            Ok(bytes) => ron::de::from_bytes(&bytes).map_err(Into::into),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Default::default()),
            Err(err) => Err(err.into()),
        }
    }

    pub fn save(&self, parent_dir: &Path) -> anyhow::Result<()> {
        let file_path = new_settings_file_path(parent_dir.to_path_buf());
        log::info!("Saving current settings into file: {}", file_path.display());
        let mut bytes = vec![];
        ron::ser::to_writer_pretty(&mut bytes, self, Default::default())?;
        if let Some(parent_path) = file_path.parent() {
            fs::create_dir_all(&parent_path)?;
        }
        fs::write(&file_path, &bytes)?;
        Ok(())
    }

    pub async fn save_spawn_blocking(self, parent_dir: PathBuf) -> anyhow::Result<()> {
        match tokio::runtime::Handle::current()
            .spawn_blocking(move || self.save(&parent_dir))
            .await
        {
            Ok(Ok(())) => Ok(()),
            Ok(Err(err)) => {
                anyhow::bail!("Failed to save: {err}");
            }
            Err(err) => {
                anyhow::bail!("Failed to join blocking task after saving: {err}");
            }
        }
    }

    #[must_use]
    pub fn storage_dir(&self) -> Option<OwnedDirPath> {
        self.database_url
            .as_ref()
            .and_then(|url| {
                url.to_file_path()
                    .map(|f| f.parent().map(Path::to_path_buf))
                    .ok()
                    .flatten()
            })
            .map(DirPath::from_owned)
    }

    pub fn create_database_config(&self) -> anyhow::Result<DatabaseConfig> {
        let url = self
            .database_url
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing database URL"))?;
        let file_path = url
            .to_file_path()
            .map_err(|()| anyhow::anyhow!("Unsupported database URL: {}", url))?;
        let config = DatabaseConfig {
            connection: aoide_storage_sqlite::connection::Config {
                storage: aoide_storage_sqlite::connection::Storage::File { path: file_path },
                pool: aoide_storage_sqlite::connection::pool::Config {
                    max_size: 8.try_into().expect("non-zero"),
                    gatekeeper: aoide_storage_sqlite::connection::pool::gatekeeper::Config {
                        acquire_read_timeout_millis: 10_000.try_into().expect("non-zero"),
                        acquire_write_timeout_millis: 30_000.try_into().expect("non-zero"),
                    },
                },
            },
            migrate_schema: true,
        };
        Ok(config)
    }

    pub fn update_music_dir(&mut self, new_music_dir: Option<&DirPath<'_>>) -> bool {
        if self.music_dir.as_ref() == new_music_dir {
            return false;
        }
        if let Some(new_music_dir) = new_music_dir {
            log::info!("Updating music directory: {}", new_music_dir.display());
        } else {
            log::info!("Resetting music directory");
        }
        self.music_dir = new_music_dir
            .map(ToOwned::to_owned)
            .map(DirPath::into_owned);
        true
    }
}

#[must_use]
fn new_settings_file_path(parent_dir: PathBuf) -> PathBuf {
    let mut path_buf = parent_dir;
    path_buf.push(FILE_NAME);
    path_buf.set_extension(FILE_SUFFIX);
    path_buf
}

#[must_use]
fn default_database_file_path(parent_dir: PathBuf) -> PathBuf {
    let mut path_buf = parent_dir;
    path_buf.push(DEFAULT_DATABASE_FILE_NAME);
    path_buf.set_extension(DEFAULT_DATABASE_FILE_SUFFIX);
    path_buf
}

pub fn restore_from_parent_dir(parent_dir: &Path) -> anyhow::Result<Settings> {
    log::info!("Loading saved settings from: {}", parent_dir.display());
    let mut settings = Settings::load(parent_dir)
        .map_err(|err| {
            log::warn!("Failed to load saved settings: {}", err);
        })
        .unwrap_or_default();
    if settings.database_url.is_none() {
        let database_file_path = default_database_file_path(parent_dir.to_path_buf());
        log::info!(
            "Using default SQLite database: {}",
            database_file_path.display()
        );
        settings.database_url = Url::from_file_path(&database_file_path).ok();
    }
    debug_assert!(settings.database_url.is_some());
    Ok(settings)
}

/// Manages the mutable, observable state
#[allow(missing_debug_implementations)]
pub struct ObservableState {
    state_pub: Publisher<Settings>,
}

impl ObservableState {
    #[must_use]
    pub fn new(initial_state: Settings) -> Self {
        let (state_pub, _) = new_pubsub(initial_state);
        Self { state_pub }
    }

    #[must_use]
    pub fn state(&self) -> Ref<'_, Settings> {
        self.state_pub.read()
    }

    #[must_use]
    pub fn subscribe_state(&self) -> Subscriber<Settings> {
        self.state_pub.subscribe()
    }

    #[allow(clippy::must_use_candidate)]
    pub fn update_state(&self, modify_state: impl FnOnce(&mut Settings) -> bool) -> bool {
        self.state_pub.modify(modify_state)
    }

    /// Save the settings after changed.
    pub fn on_state_changed_saver_task(
        &self,
        settings_dir: PathBuf,
        mut report_save_error: impl FnMut(anyhow::Error) + Send + 'static,
    ) -> impl Future<Output = ()> + Send + 'static {
        let mut settings_sub = self.subscribe_state();
        // Read the initial settings immediately before spawning the async task
        let mut old_settings = settings_sub.read().to_owned();
        async move {
            log::debug!("Starting on_state_changed_saver_task");
            while settings_sub.changed().await.is_ok() {
                let new_settings = settings_sub.read_ack().to_owned();
                if old_settings != new_settings {
                    log::debug!("Saving changed settings: {old_settings:?} -> {new_settings:?}");
                    old_settings = new_settings.clone();
                    if let Err(err) = new_settings.save_spawn_blocking(settings_dir.clone()).await {
                        report_save_error(err);
                    }
                }
            }
            log::debug!("Stopping on_state_changed_saver_task");
        }
    }

    #[allow(clippy::must_use_candidate)]
    pub fn update_music_dir(&self, new_music_dir: &DirPath<'_>) -> bool {
        self.update_state(|state| state.update_music_dir(Some(new_music_dir)))
    }

    #[allow(clippy::must_use_candidate)]
    pub fn reset_music_dir(&self) -> bool {
        self.update_state(|state| state.update_music_dir(None))
    }

    /// Listen for changes of the music directory.
    ///
    /// The `on_changed` callback closure must return `true` to continue
    /// listening and `false` to abort listening.
    pub fn on_music_dir_changed_task(
        &self,
        mut on_changed: impl FnMut(Option<&OwnedDirPath>) -> bool + Send + 'static,
    ) -> impl Future<Output = ()> + Send + 'static {
        let mut settings_sub = self.subscribe_state();
        // Read the initial value immediately before spawning the async task
        let mut value = settings_sub.read_ack().music_dir.clone();
        async move {
            log::debug!("Starting on_music_dir_changed_task");
            // Enforce initial update
            let mut value_changed = true;
            loop {
                #[allow(clippy::collapsible_if)] // suppress false positive warning
                if value_changed {
                    if !on_changed(value.as_ref()) {
                        // Consumer has rejected the notification
                        log::debug!("Aborting on_music_dir_changed_task");
                        return;
                    }
                }
                value_changed = false;
                if settings_sub.changed().await.is_err() {
                    // Publisher has disappeared
                    log::debug!("Aborting on_music_dir_changed_task");
                    break;
                }
                let settings = settings_sub.read_ack();
                let new_value = settings.music_dir.as_ref();
                if value.as_ref() != new_value {
                    value = new_value.cloned();
                    value_changed = true;
                }
            }
            log::debug!("Stopping on_music_dir_changed_task");
        }
    }
}
