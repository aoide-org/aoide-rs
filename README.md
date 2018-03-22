# aoide-rs

Backend services for curating and enjoying your music.

## Dependencies

## Tools

### Database Migrations

Link: [diesel.rs](https://diesel.rs)

#### Install Diesel CLI

```
cargo install diesel_cli --no-default-features --features "sqlite"
```

#### Create a new SQLite migration

Any modification of the database schema and contents requires new migration scripts:
```
diesel migration --migration-dir db/migrations/sqlite generate <MIGRATION_NAME>
```

#### Create a new SQLite database

A new database is created by applying all migrations:
```
diesel migration --migration-dir db/migrations/sqlite --database-url <SQLITE_DATABASE_FILE> run
```

Use the file extension *.sqlite* for SQLite database files. Executing this migration command on existing database files will run all pending migrations.

The migration commands *revert* and *redo* only affect the most recent migration.

## About

Aoide - the muse of voice and song.

> There are only two hard things in Computer Science: cache invalidation and naming things.
> -- Phil Karlton

See also: [TwoHardThings](https://martinfowler.com/bliki/TwoHardThings.html)
