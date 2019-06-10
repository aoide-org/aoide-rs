# aoide-rs

A web service for managing and exploring music collections.

## Domain Concepts

### Track

The domain model is centered around the concept of individual *tracks* with detailed metadata:

- Multiple independent sources (URI) and data formats (MP3, AAC, FLAC, ...) per track
- Track and album titles on different levels (main title, subtitle, classical work, classical movement)
- Track and album actors with different roles (artist, composer, performer, ...)
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
cargo run --bin aoide -- -vv --listen [::1]:7878 /tmp/aoide.sqlite
```

In this example the following command line parameters are passed through to the executable:

| Parameter | Description |
| ----------|-------------|
|-vv        | Log level INFO |
|--listen [::1]:7878 | Listen on IPv6 loopback device at port 7878 for incoming HTTP requests |
|/tmp/aoide.sqlite | Open or create the SQLite database file and perform any necessary maintenance tasks |

Use _--help_ for a list and description of all available command line parameters:

```bash
cargo run --bin aoide -- --help
```

> Use `cargo run --release ...` to build and run an optimized release build!

#### ICYW

On a 3x4 numeric key pad 7878 = RUST.

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

A statically linked executable for the host architecture can be built with the help of
[clux/muslrustclux/muslrust](https://github.com/clux/muslrust) and the corresponding
Docker image.

##### Update the Docker image

```bash
make -f Makefile.clux-muslrust pull
```

##### Build the application

```bash
make -f Makefile.clux-muslrust build
```

The resulting self-contained executable can be found in _bin/x86_64-unknown-linux-musl/_.

#### Run

Various parameters for running the dockerized executable can be customized in the Makefile.
A Docker container from this image is created and started with the following command:

```bash
make -f Makefile.clux-muslrust run
```

The `run` target uses the variables `RUN_HTTP_PORT` and `RUN_DATA_DIR` defined in the Makefile
for configuring communication and persistent storage of the container. Use the corresponding
`docker` command as a template and starting point for your custom startup configuration.

To stop and ultimately remove the Docker container use the following command:

```bash
make -f Makefile.clux-muslrust stop
```

#### Volumes

The Docker container is not supposed to store any persistent state. Instead the SQLite
database file should be placed in a directory on the host that is mapped as a
[Volume](https://docs.docker.com/storage/volumes) into the container at _/aoide/data_.

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
