![aoide banner](resources/aoide_banner_1280x340.png)

# aoide - All about music

A local HTTP/REST service for managing and exploring music collections. Independent and portable. Written in Rust.

## Fundamentals

### Collections and tracks

A music _collection_ is an aggregation of _tracks_ or songs. Tracks can
belong to multiple collections.

### Tracks

The nature of tracks is twofold. The actual content of a track is the
audio stream, encoded with some _audio codec_. The other part is metadata
about this content, like an _artist_ or a _title_.

Some or all of this metadata could be stored together with the audio stream
in a file format like _MP3_ with _ID3v2_ tags or an _MP4_ container with
_atoms_. If the audio stream is not stored locally but provided by a streaming
service then this metadata might be obtained separately through an API
request.

### Playlists and crates

Traditionally music collections are organized into _playlists_ (ordered) or
_crates_ (unordered). Both playlists and crates are _static_, i.e. tracks
are assigned independent of their metadata. Modifying the metadata will not
change the membership to playlists and crates.

Some applications allow defining _dynamic_ playlists or crates. In this
case, the membership is defined by a _selection criteria_. Their internal
ordering (in case of playlists) is defined by a _sort criteria_.

### Tags

Metadata is usually restricted to predefined properties that are not
extensible. Aoide allows assigning custom metadata through _tags_.

Only a basic set of properties is predefined, everything else can be
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

Aoide does neither support playlists nor crates (yet). Instead, powerful
queries with filtering and sorting criteria can be defined by clients.
The criteria of those queries can refer to both predefined textual or
numerical properties as well as all custom tags with their facet, label,
and score.

## Integrations

### Mixxx (experimental)

A proof-of-concept for integrating _aoide_ with [Mixxx](https://www.mixxx.org)
is available in this [PR #2282](https://github.com/mixxxdj/mixxx/pull/2282)
for testing.

## Build & run
[Build & run]: #build-and-run

### Executable

The server executable is built with the following command:

```bash
cargo build --bin aoide
```

During development it is handy to build and run the executable in a single step:

```bash
cargo run --bin aoide -- -vv --listen "[::1]:0" /tmp/aoide.sqlite
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
    -p 7878:8080 \
    -v .:/data:Z \
    aoide:latest
```

This will start the instance with the database file stored in the current working directory.

## API

### CLI

The server is configured at startup with various command line parameters. Use the command-line argument `--help` for an overview.

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
