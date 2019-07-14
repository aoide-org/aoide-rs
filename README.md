# aoide-rs

A web service for managing and exploring music collections.

## Domain Concepts

### Track

The domain model is centered around the concept of individual *tracks* with detailed metadata:

- Multiple independent sources (URI) and data formats (MP3, AAC, FLAC, ...) per track
- Both track and album titles on different levels (main title, subtitle, classical work, classical movement)
- Both track and album actors with different roles (artist, composer, performer, ...)
- Release (date, label, ...) and licensing information
- Various audio und musical properties
- Position, beat, and key markers for live performance and mixing
- *Faceted and scored tags*
  - Custom properties (genre, mood, style, epoch, comment, rating, ...)
  - Feature analysis results (energy, valence, danceability, ...)
  - ...

### Collection

Tracks belong to one or more *collections*.

## Design Principles

### Interoperability

The universal track domain model with its detailed metadata and a customizable tagging scheme
should cover many use cases and existing data models, ranging from casual music players to
dedicated DJ apps.

### Synchronization

Both *tracks* and *collections* are referenced by globally unique identifiers for offline
usage and independent operation.

Track entities are revisioned to allow synchronization of file tags and synchronization
with external libraries. The synchronization algorithms are not part of this service
that is agnostic of any client software.

### Restrictions

Common music and DJ apps allow to arrange a selection of tracks into *playlists* or *crates*.
Playlists are manually ordered lists of tracks that may contain duplicate entries. Crates are
unordered sets of tracks without any duplicate entries.

Aoide currently supports none of those static track selections. Instead dynamic queries with
filter and ordering criteria are proposed for selecting a subset of tracks from a collection.

Crates can be simulated by assigning dedicated *tags*, e.g. with a dedicated *facet* named 'crate'.
This is actually a special case of *virtual crates* that are implemented by storing and executing
arbitrary *prepared queries*.

## Technology

- Cross-platform REST API (OpenAPI)
- Portable domain model (JSON)
- Hybrid relational/document-oriented database (SQLite)
- Standalone static executable with no dependencies (optional, see below)
- Safely written in *stable* Rust

## Development

### Executable

The server executable is built with the following command:

```bash
cargo build --bin aoide
```

During development it is handy to build and run the executable in a single step:

```bash
cargo run --bin aoide -- -vv --listen [::1]:0 /tmp/aoide.sqlite
```

In this example the following command line parameters are passed through to the executable:

| Parameter        | Description |
| -----------------|-------------|
|-vv               | Log level INFO |
|--listen [::1]:0  | Listen on IPv6 loopback device and bind to an ephemeral port for incoming HTTP requests |
|/tmp/aoide.sqlite | Open or create the SQLite database file and perform any necessary maintenance tasks |

The actual socket address with the bound (ephemeral) port will be printed on the first line to *stdout*
where the client can pick it up for connecting. You may also bind the service to some predefined port.

Logs messages are printed to *stderr*.

Use _--help_ for a list and description of all available command line parameters:

```bash
cargo run --bin aoide -- --help
```

> Use `cargo run --release ...` to build and run an optimized release build!

### Tests

Build and run the unit tests with the following command:

```bash
cargo test --all --verbose -- --nocapture
```

## Deployment

### Native

Follow the instructions in _Development_ for building a dynamically linked executable
for the host system.

### Docker

#### Build

```sh
docker build -t aoide:latest .
```

The final image is created `FROM scratch` and does not provide any user environment or shell.
It contains just the statically linked executable that can be extracted with `docker cp`.

#### Run

The container exposes the internal port 8080 for publishing to the host. The volume that
hosts the SQLite database is mounted at /data.

Example:

```sh
docker run --rm \
    -e RUST_LOG=info \
    -p 7878:8080 \
    -v .:/data:Z \
    aoide:latest
```

This will start the instance with the the database file stored in the current working directory.

## API

### Command Line

The server is configured at startup with various command line parameters.

### REST

Once started the server will respond with a static HTML page when sending a GET request
to the root path _/_ or _/index.html_. This page contains a link to the embedded
[OpenAPI](https://www.openapis.org) specification implemented by the service.

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
