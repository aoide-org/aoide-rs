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
- The database can be rebuilt from scratch with the just content of the vault tables, i.e. only the vault storage needs to be considered for reading/writing/importing/exporting/synchronizing

## Dependencies

A list of projects which we depend on directly.

### Communication

[Actix (Web)](https://actix.rs) for the REST API and internal messaging/scheduling

### Serialization

[Serde](https://serde.rs) for serializing/deserializing the domain model and request/response parameters

### Persistent Storage

[Diesel](https://diesel.rs) for managing the database schema and building queries

[r2d2](https://github.com/sfackler/r2d2) for database connection pooling

[SQLite](https://www.sqlite.org/) as the database backend

## Development

### Executable

The server executable is built with the following command:

```bash
cargo build --bin aoide
```

During development it is handy to build and run the executable in a single step:

```bash
cargo run --bin aoide -- -vv --listen localhost:8081 /tmp/aoide.sqlite
```

In this example the following command line parameters are passed through to the executable:

| Parameter | Description |
| ----------|-------------|
|-vv        | Log level INFO |
|--listen localhost:8080 | Listen on localhost:8080 for incoming HTTP requests |
|/tmp/aoide.sqlite | Open or create the SQLite database file and perform any necessary maintenance tasks |

Use _--help_ for a list and description of all available command line parameters:

```bash
cargo run --bin aoide -- --help
```

### Tests

Build and run the unit tests with the following command:

```bash
cargo test --all --verbose -- --nocapture
```

## Deployment

### Native

Follow the instructions in _Development_ for building a dynamically linked executable for the host system.

### Docker

#### Build

A statically linked executable for the host architecture can be built with the help of [clux/muslrustclux/muslrust](https://github.com/clux/muslrust) and the corresponding Docker image.

```bash
make build
```

> On Fedora the `docker` command must be executed as _root_ and  you might need to add `sudo` for executing the `make` command. Since the build needs write access for the target directory you might also need to relocate that, e.g. by copying it recursively to _/tmp_ and starting the build with `sudo make build` there.

The resulting self-contained executable can be found in _target/x86_64-unknown-linux-musl/release/aoide_ and has been packaged into a slim runtime Docker image based on [Alpine Linux](https://hub.docker.com/_/alpine/).

#### Run

Various parameters for running the dockerized executable can be customized in the Makefile. A Docker container from this image is created and started with the following command:

```bash
make run
```

The `run` target uses the variables `RUN_HTTP_PORT` and `RUN_DATA_DIR` defined in the Makefile for configuring communication and persistent storage of the container. Use the corresponding `docker` command as a template and starting point for your custom startup configuration.

To stop and ultimately remove the Docker container use the following command:

```bash
make stop
```

#### Volumes

The Docker container is not supposed to store any persistent state. Instead the SQLite database file should be placed in a directory on the host that is mapped as a [Volume](https://docs.docker.com/storage/volumes) into the container at _/aoide/data_.

## API

### Command Line

The server is configured at startup with various command line parameters.

### REST

Once started the server will respond with a static HTML page when sending a GET request to the root path _/_ or _/index.html_. This page contains a link to the embedded [OpenAPI](https://www.openapis.org) specification implemented by the service.

Use the [Swagger Editor](https://editor.swagger.io) for exploring the API specification.

## Licensing

License: AGPL-3.0-or-later

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as
published by the Free Software Foundation, either version 3 of the
License, or (at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.

---

## One more thing...

*Ἀοιδή* - the muse of voice and song in Greek mythology.

> There are only two hard things in Computer Science: cache invalidation and naming things.
>
> -- Phil Karlton

See also: [TwoHardThings](https://martinfowler.com/bliki/TwoHardThings.html)

---

## Appendix

*Some (more or less) obsolete text snippets from an earlier version that should be rewritten or removed eventually.*

### Database Migrations

#### Install Diesel CLI

```bash
cargo install diesel_cli --no-default-features --features "sqlite"
```

#### Create or update an SQLite Database File

Database files are created or updated by applying all (pending) migrations:

```bash
diesel migration --migration-dir resources/migrations/sqlite --database-url <SQLITE_DATABASE_FILE> run
```

By convention use the file extension *.sqlite* for SQLite database files.

#### Add a new SQLite Database Migration

Modification of the database schema or its contents requires the creation of both *up* (forward) and *down* (backward) migration scripts:

```bash
diesel migration --migration-dir resources/migrations/sqlite generate <MIGRATION_NAME>
```

Test your scripts with the migration commands *run* followed by *revert* + *run* or *redo*! Undo the migration with the command *revert*

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
