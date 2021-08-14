// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use super::*;

use aoide_repo::playlist::EntryRepo as _;

use aoide_repo_sqlite::prelude::*;

use diesel::sql_query;

///////////////////////////////////////////////////////////////////////

diesel_migrations::embed_migrations!("repo-sqlite/migrations");

pub fn initialize(connection: &SqliteConnection) -> Result<()> {
    log::info!("Initializing database");
    sql_query(r#"
PRAGMA journal_mode = WAL;        -- better write-concurrency
PRAGMA synchronous = NORMAL;      -- fsync only in critical moments, safe for journal_mode = WAL
PRAGMA wal_autocheckpoint = 1000; -- write WAL changes back every 1000 pages (default), for an in average 1MB WAL file
PRAGMA wal_checkpoint(TRUNCATE);  -- free some space by truncating possibly massive WAL files from the last run
PRAGMA secure_delete = 0;         -- avoid some disk I/O
PRAGMA automatic_index = 1;       -- detect and log missing indexes
PRAGMA foreign_keys = 1;          -- check foreign key constraints
PRAGMA defer_foreign_keys = 1;    -- delay enforcement of foreign key constraints until commit
PRAGMA encoding = 'UTF-8';
"#).execute(connection)?;
    Ok(())
}

pub fn migrate_schema(connection: &SqliteConnection) -> Result<()> {
    log::info!("Migrating database schema");
    embedded_migrations::run(connection)?;
    Ok(())
}

pub fn cleanse(connection: &SqliteConnection, vacuum: bool) -> Result<()> {
    log::info!("Cleansing database");
    let db = RepoConnection::new(&*connection);
    db.transaction::<_, DieselTransactionError<RepoError>, _>(|| {
        let deleted_playlist_entries =
            db.delete_playlist_entries_with_tracks_from_other_collections()?;
        if deleted_playlist_entries > 0 {
            log::warn!(
                "Deleted {} playlist entries with tracks from other collections",
                deleted_playlist_entries
            );
        }
        Ok(())
    })?;

    // According to Richard Hipp himself executing VACUUM before ANALYZE is the
    // recommended order: https://sqlite.org/forum/forumpost/62fb63a29c5f7810?t=h
    if vacuum {
        log::info!("Rebuilding database storage");
        db.vacuum()?;
    }

    log::info!("Analyzing and optimizing database statistics");
    db.analyze_and_optimize_stats()?;
    Ok(())
}
