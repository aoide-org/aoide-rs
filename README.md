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

#### Create or update an SQLite database file

Database files are created or updated by applying all (pending) migrations:
```
diesel migration --migration-dir db/migrations/sqlite --database-url <SQLITE_DATABASE_FILE> run
```

By convention use the file extension *.sqlite* for SQLite database files.

#### Add a new SQLite database migration

Modification of the database schema or its contents requires the creation of both *up* (forward) and *down* (backward) migration scripts:
```
diesel migration --migration-dir db/migrations/sqlite generate <MIGRATION_NAME>
```

Test your scripts with the migration commands *run* followed by *revert* + *run* or *redo*! Undo the migration with the command *revert*

## About

Aoide - the muse of voice and song.

> There are only two hard things in Computer Science: cache invalidation and naming things.
>
> -- Phil Karlton

See also: [TwoHardThings](https://martinfowler.com/bliki/TwoHardThings.html)
