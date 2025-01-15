// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{anyhow, Result};
use aoide_core::{
    collection::MediaSourceConfig,
    media::content::{ContentPath, ContentPathConfig, VirtualFilePathConfig},
    util::url::BaseUrl,
    Collection,
};
use diesel::Connection;

use aoide_repo_sqlite::{initialize_database, run_migrations, DbConnection};
use url::Url;

struct DbFixture {
    connection: DbConnection,
}

impl DbFixture {
    pub(super) fn new() -> Result<Self> {
        let mut connection =
            DbConnection::establish(":memory:").expect("in-memory database connection");
        initialize_database(&mut connection)?;
        run_migrations(&mut connection).map_err(|err| anyhow!(err))?;
        Ok(Self { connection })
    }
}

#[cfg(not(target_family = "windows"))]
const FILE_URL_PREFIX: &str = "file://";

#[cfg(target_family = "windows")]
const FILE_URL_PREFIX: &str = "file://C:";

#[test]
fn resolve_content_path_from_url() -> anyhow::Result<()> {
    let mut fixture = DbFixture::new()?;
    let root_url = BaseUrl::parse_strict(&format!("{FILE_URL_PREFIX}/a/b/"))?;
    let collection = Collection {
        title: "Test Collection".into(),
        notes: Some("Some personal notes".into()),
        kind: None,
        color: None,
        media_source_config: MediaSourceConfig {
            content_path: ContentPathConfig::VirtualFilePath(VirtualFilePathConfig {
                root_url: root_url.clone(),
                excluded_paths: vec![],
            }),
        },
    };
    let collection_uid = super::create(&mut fixture.connection, collection)?
        .hdr
        .uid
        .clone();
    assert_eq!(
        Some(ContentPath::default()),
        super::resolve_content_path_from_url(&mut fixture.connection, &collection_uid, &root_url)?
    );
    assert_eq!(
        Some(ContentPath::new("c".into())),
        super::resolve_content_path_from_url(
            &mut fixture.connection,
            &collection_uid,
            &root_url.join("c")?,
        )?
    );
    assert_eq!(
        Some(ContentPath::new("c/".into())),
        super::resolve_content_path_from_url(
            &mut fixture.connection,
            &collection_uid,
            &root_url.join("c/")?,
        )?
    );
    // Root directory without trailing slash
    assert_eq!(
        None,
        super::resolve_content_path_from_url(
            &mut fixture.connection,
            &collection_uid,
            &Url::parse(&format!("{FILE_URL_PREFIX}/a/b"))?,
        )?
    );
    // Other directory with trailing slash
    assert_eq!(
        None,
        super::resolve_content_path_from_url(
            &mut fixture.connection,
            &collection_uid,
            &Url::parse(&format!("{FILE_URL_PREFIX}/a/c/"))?,
        )?
    );
    assert_eq!(
        None,
        super::resolve_content_path_from_url(
            &mut fixture.connection,
            &collection_uid,
            &Url::parse(&format!("{FILE_URL_PREFIX}/c/"))?,
        )?
    );
    // Other directory without trailing slash
    assert_eq!(
        None,
        super::resolve_content_path_from_url(
            &mut fixture.connection,
            &collection_uid,
            &Url::parse(&format!("{FILE_URL_PREFIX}/a/c"))?,
        )?
    );
    assert_eq!(
        None,
        super::resolve_content_path_from_url(
            &mut fixture.connection,
            &collection_uid,
            &Url::parse(&format!("{FILE_URL_PREFIX}/c"))?,
        )?
    );
    Ok(())
}
