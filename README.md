# aoide-rs

APIs and backend services for curating and enjoying your music.

## Requirements & Features

### System Context

- Utilizes standard serialization formats for communication between frontend and backend components
- Frontend components might be written in any language for various platforms (desktop, mobile, embedded, ...)
- Backend components are written primarily in Rust
- Frontend components that are also written in Rust might reuse storage-independent backend code, e.g. the domain model

### Domain Model

- Applies [*Domain-driven Design (DDD)*](www.domaindrivendesign.org) principles
- Domain objects are out-of-the box serializable with support for various formats  (JSON, BSON, CBOR, Bincode, ...)
- Incorporates features from public APIs and standards (Spotify/EchoNest, MusicBrainz, ID3v2/MP4/VorbisComment/APE2v2,...)
- Supports multi-valued attributes for selected fields
- Supports custom and extensible tagging schemes

### Persistent Storage

- Applies a hybrid approach between SQL and NoSQL document storage (JSON, BSON, CBOR, Bincode, ...)
- Single *vault table* per *aggregate root* (= top-level domain entity) that stores essential identity and metadata together with a serialized representation
- Multiple *join* or *view* tables that provide viewing/seraching/filtering/ordering capabilities for one or more aggregate roots
- The database can be rebuild from scratch with the just content of the vault tables, i.e. only the vault storage needs to be considered for reading/writing/importing/exporting/synchronizing

## Dependencies

A list of projects which we depend on directly.

### Networking

[Tokio](https://tokio.rs)

### Serialization

[Serde](https://serde.rs)

### Data Mapping & Schema Migration

[Diesel](https://diesel.rs)

### Media Import/Export

[GStreamer](https://gstreamer.freedesktop.org) and [GStreamer bindings for Rust](https://github.com/sdroege/gstreamer-rs)

## Build

### Docker

```bash
make build
```

On Fedora `docker` must be executed as root and you might need to add `sudo` before this command. Since the build needs write access for the target directory you might also need to relocate that.

## Tools

### Database Migrations

#### Install Diesel CLI

```bash
cargo install diesel_cli --no-default-features --features "sqlite"
```

#### Create or update an SQLite database file

Database files are created or updated by applying all (pending) migrations:

```bash
diesel migration --migration-dir resources/migrations/sqlite --database-url <SQLITE_DATABASE_FILE> run
```

By convention use the file extension *.sqlite* for SQLite database files.

#### Add a new SQLite database migration

Modification of the database schema or its contents requires the creation of both *up* (forward) and *down* (backward) migration scripts:

```bash
diesel migration --migration-dir resources/migrations/sqlite generate <MIGRATION_NAME>
```

Test your scripts with the migration commands *run* followed by *revert* + *run* or *redo*! Undo the migration with the command *revert*

## Running

Print help information:

```bash
cargo run --bin aoide -- --help
```

Starting the service without specifying an SQLite database file will create an in-memory database:

```bash
cargo run --bin aoide -- -vv
```

To connect to an existing database or to create a new database start the service with the path to the file as an additional argument:

```bash
cargo run --bin aoide -- -vv /tmp/aoide.sqlite
```

## Examples

### Testing

Run all tests with verbose console output:

```bash
cargo test --all --verbose -- --nocapture
```

### JSON Import/Export

Read and parse (no import yet) JSON example files into domain model objects.

A file with a single track that demonstrates most of the capabilities of the domain model:

```bash
cargo run --bin parse_json examples/json/tracks_single.json
```

A file with a single track and only the minimum set of fields:

```bash
cargo run --bin parse_json examples/json/tracks_minimum.json
```

## One more thing

*Aoide* - the muse of voice and song.

Not to forget dancing, but *Terpsichord* was such an awkward long name. It's up to you to bring this to life.

> There are only two hard things in Computer Science: cache invalidation and naming things.
>
> -- Phil Karlton

See also: [TwoHardThings](https://martinfowler.com/bliki/TwoHardThings.html)
