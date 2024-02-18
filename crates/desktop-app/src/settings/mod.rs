// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use url::Url;

use aoide_backend_embedded::storage::DatabaseConfig;

use crate::{fs::DirPath, Observable, ObservableReader, ObservableRef};

pub const FILE_NAME: &str = "aoide_desktop_settings";

pub const FILE_SUFFIX: &str = "ron";

pub const DEFAULT_DATABASE_FILE_NAME: &str = "aoide";

pub const DEFAULT_DATABASE_FILE_SUFFIX: &str = "sqlite";

pub mod tasklet;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// File path of the SQLite database.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_url: Option<Url>,

    /// The root music directory.
    ///
    /// Used as to select the corresponding collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub music_dir: Option<DirPath<'static>>,

    /// Filter for a collection kind.
    ///
    /// If set only collections of this kind should be considered
    /// by an application and all other collections should be
    /// ignored.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_kind: Option<String>,
}

impl State {
    pub fn restore_from_parent_dir(parent_dir: &Path) -> anyhow::Result<Self> {
        log::info!(
            "Loading saved settings from: {parent_dir}",
            parent_dir = parent_dir.display()
        );
        let mut settings = Self::load(parent_dir)
            .map_err(|err| {
                log::warn!("Failed to load saved settings: {err}");
            })
            .unwrap_or_default();
        if settings.database_url.is_none() {
            let database_file_path = default_database_file_path(parent_dir.to_path_buf());
            log::info!(
                "Using default SQLite database: {database_file_path}",
                database_file_path = database_file_path.display()
            );
            settings.database_url = Url::from_file_path(&database_file_path).ok();
        }
        debug_assert!(settings.database_url.is_some());
        Ok(settings)
    }

    pub fn load(parent_dir: &Path) -> anyhow::Result<State> {
        let file_path = new_settings_file_path(parent_dir.to_path_buf());
        log::info!(
            "Loading settings from file: {file_path}",
            file_path = file_path.display()
        );
        match fs::read(&file_path) {
            Ok(bytes) => ron::de::from_bytes(&bytes).map_err(Into::into),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Default::default()),
            Err(err) => Err(err.into()),
        }
    }

    pub fn save(&self, parent_dir: &Path) -> anyhow::Result<()> {
        let file_path = new_settings_file_path(parent_dir.to_path_buf());
        log::info!(
            "Saving current settings into file: {file_path}",
            file_path = file_path.display()
        );
        let mut bytes = vec![];
        ron::ser::to_writer_pretty(&mut bytes, self, Default::default())?;
        if let Some(parent_path) = file_path.parent() {
            fs::create_dir_all(parent_path)?;
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
                anyhow::bail!("failed to save: {err}");
            }
            Err(err) => {
                anyhow::bail!("failed to join blocking task after saving: {err}");
            }
        }
    }

    #[must_use]
    pub fn storage_dir(&self) -> Option<DirPath<'static>> {
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

    #[allow(clippy::missing_panics_doc)]
    pub fn create_database_config(&self) -> anyhow::Result<DatabaseConfig> {
        let url = self
            .database_url
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing database URL"))?;
        let file_path = url
            .to_file_path()
            .map_err(|()| anyhow::anyhow!("unsupported database URL: {url}"))?;
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
            migrate_schema: None,
        };
        Ok(config)
    }

    pub fn try_update_music_dir(&mut self, music_dir: Option<&DirPath<'_>>) -> bool {
        if self.music_dir.as_ref() == music_dir {
            log::debug!("Unchanged music directory: {music_dir:?}");
            return false;
        }
        if let Some(music_dir) = music_dir {
            log::info!(
                "Updating music directory: {music_dir}",
                music_dir = music_dir.display()
            );
        } else {
            log::info!("Resetting music directory");
        }
        self.music_dir = music_dir.map(ToOwned::to_owned).map(DirPath::into_owned);
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

pub type StateSubscriber = discro::Subscriber<State>;

/// Manages the mutable, observable state
#[derive(Debug)]
pub struct ObservableState(Observable<State>);

impl ObservableState {
    #[must_use]
    pub fn new(initial_state: State) -> Self {
        Self(Observable::new(initial_state))
    }

    #[must_use]
    pub fn read(&self) -> ObservableStateRef<'_> {
        self.0.read()
    }

    #[must_use]
    pub fn subscribe_changed(&self) -> StateSubscriber {
        self.0.subscribe_changed()
    }

    pub fn set_modified(&self) {
        self.0.set_modified();
    }

    #[allow(clippy::must_use_candidate)]
    pub fn try_update_music_dir(&self, music_dir: Option<&DirPath<'_>>) -> bool {
        self.0.modify(|state| state.try_update_music_dir(music_dir))
    }
}

impl Default for ObservableState {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

pub type ObservableStateRef<'a> = ObservableRef<'a, State>;

impl ObservableReader<State> for ObservableState {
    fn read_lock(&self) -> ObservableStateRef<'_> {
        self.0.read_lock()
    }
}
