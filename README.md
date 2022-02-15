# aoide

![aoide banner](assets/aoide_banner_1280x340.png)

[![GitLab CI](https://gitlab.com/uklotzde/aoide-rs/badges/dev/pipeline.svg?style=flat-square)](https://gitlab.com/uklotzde/aoide-rs/pipelines?scope=branches)
[![GitHub CI](https://github.com/aoide-org/aoide-rs/actions/workflows/continuous-integration.yaml/badge.svg?branch=dev&style=flat-square)](https://github.com/aoide-org/aoide-rs/actions/workflows/continuous-integration.yaml)
[![Security audit](https://github.com/aoide-org/aoide-rs/actions/workflows/security-audit.yaml/badge.svg?branch=dev&style=flat-square)](https://github.com/aoide-org/aoide-rs/actions/workflows/security-audit.yaml)
[![License](https://img.shields.io/badge/license-AGPLv3-blue.svg?style=flat-square)](https://gitlab.com/uklotzde/aoide-rs/blob/main/LICENSE.md)

A local HTTP/REST service for managing and exploring music collections. Independent and portable. Written in Rust.

## The idea

The basic ideas and domain model of aoide in a few sentences:

- All **_media sources_** (e.g. audio files) are managed in the context of isolated **_collections_**.
- Each **_media source_** represents and contains a single **_track_**.
- The **_media tracker_** component is responsible for keeping track of all external modifications on _media sources_, e.g. when modifying the metadata of files.
- Tracks can be organized manually in **_playlists_** resulting in _static track lists_.
- Tracks can be **_queried_** by composing **_filtering and sorting criteria_** resulting in _dynamic track lists_ aka _virtual playlists_.

Currently the focus is on local, file-based storage. The domain model is flexible enough to also include online media sources that could be referenced by arbitrary URIs.

## Quickstart

### Use Case

- You store your audio files (mp3/m4a/flac/ogg/opus) on the local file system
- All audio files are contained in a common root directory
- The audio files contain metadata (ID3v2.4 tags/MP4 atoms/Vorbis comments)
- You consider this metadata in the audio files as the _book of records_
- You want to compose and execute search queries on these metadata properties

aoide is able to ingest the metadata from the audio files and store it in a database that could be queried. aoide is also able to detect modifications of files and re-import the affected metadata on demand as needed.

### Setup

Both the initial ingestion and subsequent re-import of updated metadata can be accomplished by running the bundled shell script [`ingest_vfs_collection.sh`](./ingest_vfs_collection.sh). It is recommended to copy the contents and customize the settings to your needs.

This sections only lists the most important configuration options that you might need to adjust. Refer to the comments in the script for advanced options.

#### Web Server

aoide implements a client/server architecture. The web server executable is started and stopped as part of the script for convenience.

The server listens for requests on `WEBSRV_URL`.

Log messages are redirected into the directory specified by `WEBSRV_LOG_DIR`.

#### SQLite Database

The database is stored in a single SQLite database files specified by `DATABASE_URL`.

#### Music Collection

Currently the only supported type of storage is a *virtual file system (VFS) with a common root directory. Media sources are then referenced by a relative path within this root directory.

The informational properties of a collection are defined by `COLLECTION_TITLE` and `COLLECTION_KIND` (optional). If this collection does not exist it will be created on first run. All subsequent runs will display an error message that the collection could not be created, ignore them.

The VFS root directory of the collection is specified by `COLLECTION_VFS_ROOT_URL`. Currently only `file://` URLs are supported for this purpose.

### Ingest Data

Run the script whenever metadata in audio files has been modified:

```shell
./ingest_vfs_collection.sh
```

You can safely interrupt the script at any point or stop the web server manually at your will. The database will catch up and resynchronize its contents on the next run. Easy and fool-proof.

### Query Data

Start the web server manually and access the web API at `<WEBSRV_URL>/api`. Please refer to the OpenAPI documentation in [`openapi.yaml`](./websrv/res/openapi.yaml).

## Behind the scenes

Pronounced /eɪˈiːdiː/ or _ay-ee-dee_ in English.

### Overview

A music _collection_ is an aggregation of both _tracks_ (aka songs) and _playlists_:

```plantuml
@startuml

skinparam backgroundColor transparent

class Collection <<entity>> {
    uid
    rev
}

class MediaSource {
    path
}
MediaSource "0..*" -up-> "1" Collection

class Track <<entity>> {
    uid
    rev
}
Track "1" -up-> "1" MediaSource

class Playlist <<entity>> {
    uid
    rev
}
Playlist "0..*" -up-> "1" Collection

@enduml
```

The top-level entities _collection_, _track_, and _playlist_ are identified by a _**u**nique **id**entifier_ or short _uid_. This identifier is generated and guaranteed to be globally unique. Modifications are tracked by a revision number _rev_.

_Media sources_ are the glue objects between _tracks_ and their _collection_. They are identified
by a (case-sensitive) _path_ that is unique within a collection. The path could contain either
a URL/URI or a relative path. The collection defines the path scheme and how to locate media
sources by their path, e.g. local files addressed by a relative path in a common root directory.

### Playlists

Traditionally music collections are organized into subsets of tracks, namely
_playlists_ (ordered) or _crates_ (unordered). Both playlists and crates are
_static_, i.e. the tracks are assigned to them independent of their metadata.
Modifying the metadata will not change the membership to playlists and crates.

Playlists are an ordered collection of _entries_. Most entries reference a track.
Entries without a track reference act as separators to partition the playlist
into sections. All tracks in a playlist must be contained in the same collection.

```plantuml
@startuml

skinparam backgroundColor transparent

class Playlist
class PlaylistEntry
class Track

Playlist "1" *-- "0..* {ordered}" PlaylistEntry
PlaylistEntry --> "0..1" Track

@enduml
```

Unordered sets of tracks aka _crates_ are currently not supported.

### Tracks

Tracks and their media sources are characterized by metadata like _artist_
, _title_, or _duration_. This metadata is stored in the database.

Some or all of this metadata might have been imported from the media source's URI,
e.g. from an _MP3_ file with _ID3v2_ tags or an _MP4_ container with _atoms_.
If the media source is not stored locally but provided by a streaming service
then this metadata might have been obtained separately by API calls.

### Tags

Track metadata is usually restricted to predefined properties that are not
extensible. Aoide allows assigning custom metadata through _tags_.

Only a basic set of track properties is predefined, everything else can be
covered by tags. Even the common property _genre_ is encoded as a tag
and allows one to assign multiple values. We will revisit this example in
a moment.

#### Plain tags

The public value of a tag is stored in the _label_. Each tag may be
assigned a _score_ between 1.0 (= maximum/default) and 0.0 (= minimum).
Tags with only a _label_ and _score_ are called _plain tags_.

#### Faceted tags

_Faceted tags_ allow storing multiple, different _labels_ for the same
_facet_. Tags with the same facet can be prioritized among each other by
assigning a _score_ like for _plain tags_.

A typical use case for multi-valued, faceted tags is the _musical genre_.
The musical genre of a track is manifold and ambiguous with a varying
degree of association. It is (by convention) represented by the facet
"genre". The different labels might contain values like "Pop", "Rock",
"R&B/Soul", "Hip-Hop/Rap", ... each with a _score_ that represents the
perceived relationship to this genre. Ordering the tag labels by their
descending score value reveals the main genre(s).

Faceted tags and their score could also be used to store feature analysis
results of the audio data like energy, valence, or danceability for
encoding a huge amount of musical knowledge about the collected tracks
and to perform clustering on this data.

### Queries

Powerful queries with filtering and sorting criteria can be defined by clients
and executed on the database. The criteria of those queries can refer to both
predefined textual or numerical properties as well as all custom tags with their
facet, label, and score.

Some applications allow defining _dynamic_ playlists or crates. In this
case, the membership is defined by a _selection criteria_. Their internal
ordering (in case of playlists) is defined by a _sort criteria_. In aoide
queries are used for this purpose.

## Build & run

[build & run]: #build-and-run

### Prerequisites

#### Trunk

Use `cargo install trunk` once to install the
[Trunk](https://github.com/thedodd/trunk)
web application builder/bundler.

The web application is embedded in the server and enabled by default.

#### just (optional)

Install [just](https://github.com/casey/just) to automate various development
tasks. Prepared recipes can be found in [`.justfile`](./.justfile).

### Executable

The server executable is built with the following commands:

```bash
cd webapp && trunk build && cd -
cargo build --all-features --package aoide-websrv
```

The _webapp_ itself is **not** part of the workspace and needs to be built separately
before building the server.

> Use `cargo build --profile production ...` for a fully optimized release build instead
of a debug build.

During development it is handy to build and run the server in a single step:

```bash
cargo run --all-features --package aoide-websrv
```

The configuration is controlled by environment variables. Please refer to the
[`.env`](./.env) file in the project folder for an example configuration.

#### Configuration examples

| Configuration                    | Description                                               |
| -------------------------------- | --------------------------------------------------------- |
| `RUST_LOG=debug`                 | Log/tracing level `debug` (trace/debug/info/warn/error)   |
| `ENDPOINT_IP=::`                 | Listen on IPv6 loopback device                            |
| `ENDPOINT_PORT=0`                | Bind to an ephemeral port for incoming HTTP requests      |
| `ENDPOINT_PORT=8080`             | Bind to port 8080 for incoming HTTP requests              |
| `DATABASE_URL=:memory:`          | Use an in-memory database for testing purposes            |
| `DATABASE_URL=/tmp/aoide.sqlite` | Open or create the corresponding SQLite database file     |
| `LAUNCH_HEADLESS=true`           | Start the web server immediately and hide the launcher UI |
| `DEFAULT_CONFIG=true`            | Start with a fresh default config instead of loading a previously stored one |

The actual socket address with the bound (ephemeral) port will be printed on the first line to _stdout_
where the client can pick it up for connecting. You may also bind the service to some predefined port.

Log/tracing messages are printed to _stderr_.

### Tests

Build and run the unit tests with the following command:

```bash
cargo test --workspace --all-features --verbose -- --nocapture
```

## Deploy

### Native

Follow the instructions in [Build & run](#build-and-run) for building a dynamically
linked executable for the host system.

### Docker

#### Docker build

```sh
docker build -t aoide:latest .
```

The final image is created `FROM scratch` and does not provide any user environment or shell.
It contains just the statically linked executable that can be extracted with `docker cp`.

#### Docker run

The container exposes the internal port 8080 for publishing to the host. The volume that
hosts the SQLite database is mounted at /data.

Example:

```sh
docker run --rm \
    -e RUST_LOG=info \
    -e DATABASE_URL=aoide.sqlite \
    -p 7878:8080 \
    -v .:/data:Z \
    aoide:latest
```

This will start the instance with the database file _aoide.sqlite_ in the current working directory.

## REST API

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
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.

---

## One more thing...

_Ἀοιδή_ - the muse of voice and song in Greek mythology.

> There are only two hard things in Computer Science: cache invalidation and naming things.
>
> -- Phil Karlton

See also: [TwoHardThings](https://martinfowler.com/bliki/TwoHardThings.html)
