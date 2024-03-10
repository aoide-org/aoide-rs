<!-- SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al. -->
<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# aoide

[![GitLab CI](https://gitlab.com/uklotzde/aoide-rs/badges/dev/pipeline.svg)](https://gitlab.com/uklotzde/aoide-rs/pipelines?scope=branches)
[![GitHub CI](https://github.com/aoide-org/aoide-rs/actions/workflows/test.yaml/badge.svg?branch=dev)](https://github.com/aoide-org/aoide-rs/actions/workflows/test.yaml)
[![Dependency status](https://deps.rs/repo/gitlab/uklotzde/aoide-rs/status.svg)](https://deps.rs/repo/gitlab/uklotzde/aoide-rs)
[![Dependency audit](https://github.com/aoide-org/aoide-rs/actions/workflows/dependency-audit.yaml/badge.svg?branch=dev)](https://github.com/aoide-org/aoide-rs/actions/workflows/dependency-audit.yaml)
[![License: GNU Affero General Public License version 3](https://img.shields.io/badge/license-AGPLv3-blue.svg)](https://opensource.org/license/agpl-v3-or-later)

Pronounced /eɪˈiːdiː/ or _ay-ee-dee_ in English.

Aimed at DJs who need to organize and search their music collections.

## Quickstart

Populate a database from your local music files and submit search queries.

- [Install Rust](https://www.rust-lang.org/tools/install)

- Run the `aoide-demo-app`:

  ```sh
  RUST_LOG=info cargo run -p aoide-demo-app
  ```

The application creates one collection for each selected music directory and sets the title
accordingly.

Both the default SQLite database and the application settings can be found in the corresponding
config directory of your platform. See the log messages for the file path.

### Metadata, tags, and searching

Click on the artwork thumbnail for opening the file URL.

#### BPM filter

Tracks can be filtered by BPM using `+<digits>` (_greater-or-equal_) and `-<digits>`
(_less-or-equal_).

`+80 -120`

This filter matches all tracks with a BPM between 80.0 and 120.0 (inclusive).

#### Hashtag filter

Supports _hashtags_ in the
[Content Group](https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html#grouping-3)
field, URL-encoded and delimited by whitespace:

`#Afterhour #Main%20Floor`

The simple search input tokenizer of the app doesn't support spaces in tag labels yet, i.e. prefer
`#MainFloor` instead of `#Main%20Floor`.

Searching is case-insensitive and for tag labels only the prefix is matched, e.g. searching `#main`
finds `#MainFloor`.

Search tokens that don't start with `#` will match various other fields like _title_, _artist_, or
_album_.

#### Multiple genres

Supports multiple [genres](https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html#id11)
separated by semicolons as a single field:

`Pop;Hip-Hop/Rap;Nu Metal`

No multi-valued file tags needed for this purpose.

## Features

- Designed for large _collections_ of _tracks_ and _playlists_
  - 100k and more tracks
- Extended _metadata_ support for DJ applications
  - Multi-valued fields
  - Analysis results and performance data
  - Custom, faceted tags
- Bidirectional synchronization of metadata with file tags
  - MP3/ID3v2, MP4/M4A/ALAC, FLAC, Ogg Vorbis, Opus
  - Mapping (largely) follows
    [MusicBrainz Picard](https://picard-docs.musicbrainz.org/appendices/tag_mapping.html)
- Rich _filtering and sorting_ capabilities
  - Query DSL
  - For creating _dynamic/smart playlists_
- Embedded storage backend
  - Primary: Relational database using [SQLite](https://www.sqlite.org/)
  - Secondary (optional): Full-text search engine using
    [Tantivy](https://github.com/quickwit-oss/tantivy)
- Modular architecture
  - Standalone (web) server executables as backend
  - Desktop applications with embedded backend
  - Web apps (WASM)
- Multi-platform
  - Linux/Windows/macOS
  - WASM (partially, only `core`/`-api` components)
- Written in pure _Rust_

## Axioms

### Globally unique identifiers

All entities in _aoide_, namely _collections_, _tracks_, and _playlists_ are identified by a
globally unique `UID` that is independent of any database backend. We use
[`ULID`s](https://github.com/ulid/spec) with the default base32 string encoding for this purpose.

### Source-centric

Internally metadata is stored and indexed in one or more databases (relational/index). This metadata
is imported from media sources, which are considered the actual _book of records_.

Media sources should primarily carry all the precious information that has been collected and
curated over the years. This includes analyzed audio properties like _perceived loudness_, musical
metadata like _bpm_ or _key_, the position of _cue points_ and _loops_, as well as any kind of
custom metadata.

If the database with the metadata becomes inaccessible, at least the information stored in the audio
files should be recoverable.

Backing up only the media sources should preserve as much metadata as possible. This is also helpful
when switching from _aoide_ to some other platform or when using different applications in parallel.

### Always synchronized

Any modification of external metadata in media sources should be reflected in the database and vice
versa. Changes are prioritized according to time stamps, i.e. newer metadata replaces older
metadata.

Switching between different applications that modify the metadata of the underlying media sources
should work seamlessly. At least when reducing conflicting, non-synchronized modifications to a
minimum.

#### Limitation (temporary)

Export of modified metadata into file tags is implemented, but is untested and has not been enabled
yet.

### Prepared queries

Traditionally, tracks have been organized into (static) playlists. This feature is also provided by
_aoide_.

That works fine as long as individual tracks are not deleted or replaced. Moreover the contents of
these playlists are usually only stored in the database.

A more flexible approach is to dynamically compose playlists through filtering and sorting criteria
based on track metadata. This feature is often referred to as _smart playlists_. If metadata
changes, the contents of all smart playlists updates automatically. There is no predefined list of
tracks. Only a _prepared query_ that, when executed on demand, returns a list of tracks.

By providing a simple _domain-specific language_ (DSL) for denoting _prepared queries_ applications
can manage and organize _smart playlists_ on their own.

## Limitations

### No (audio) streaming

The service only manages metadata. It is not supposed to provide audio streams for playing the music
that is contained in the media sources.

### No _folder-like_ organizational structures

Grouping any kind of entities (collections/tracks/playlists) into custom, hierarchical structures
like _folders_ is not supported. Client applications are responsible for providing such features as
needed. They can leverage stable, globally unique, entity identifiers (UID) for this purpose.

Various file formats (JSON/YAML/TOML/...) are much more suitable for storing hierarchical structures
than a (relational) database.

### Tested for local files

Currently the focus is on local, file-based storage. The domain model is flexible enough to also
include online media sources that could be referenced by arbitrary URIs.

Exemplary use case for testing this ability: Ingest the playlists and corresponding tracks from a
user's Spotify account as a separate collection.

### Single root directory for relative, local file paths (VFS)

The ability for referencing media sources through a relative path by using a _virtual file system_
(VFS) is restricted to a single, local root directory.

This is probably the most common and flexible use case for local files. By either permanently or
temporarily switching the root directory media sources could be accessed and loaded from a different
location as long as their relative paths remain unchanged.

## Quickstart

- [Build & run the web service](docs/BUILDING.md)
- [Ingest your music files](docs/INGEST_COLLECTION.md)
- [Contributing](docs/CONTRIBUTING.md)

## License

License: AGPL-3.0-or-later

This program is free software: you can redistribute it and/or modify it under the terms of the GNU
Affero General Public License as published by the Free Software Foundation, either version 3 of the
License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without
even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero
General Public License for more details.

You should have received a copy of the GNU Affero General Public License along with this program. If
not, see <https://www.gnu.org/licenses/>.
