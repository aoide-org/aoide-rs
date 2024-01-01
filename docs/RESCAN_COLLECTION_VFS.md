<!-- SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al. -->
<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# Rescan collection CFS

_aoide_ is built to import metadata from audio files and store it in an internal database that could
be queried. _aoide_ is also able to detect modifications of files and re-import the affected
metadata on demand as needed.

## Use case

- You store your audio files (MP3/M4A/FLAC/OGG/Opus) on the local file system.
- All audio files are contained in a common root directory.
- The audio files contain metadata (ID3v2.4 tags/MP4 atoms/Vorbis comments).
- You consider metadata in the audio files as the _book of records_.
- You want to compose and execute search queries based metadata.

## Configuration

Both the initial import and subsequent re-import of updated metadata can be accomplished by running
the bundled shell script
[`synchronize_collection_vfs.sh`](../scripts/synchronize_collection_vfs.sh). It is recommended to
copy the contents and customize the settings according to your needs.

This sections only lists the most important configuration options that you might need to adjust.
Refer to the comments in the script for advanced options.

### Web Server

aoide includes a standalone web service that is supposed to be running in the background. The server
executable is started and stopped as part of the script for convenience.

The server listens for requests on `WEBSRV_URL`.

Log messages are redirected into the directory specified by `WEBSRV_LOG_DIR`.

### SQLite Database

The database is stored in a single SQLite database files specified by `DATABASE_URL`.

### Music Collection

Currently the only supported type of storage is a \*virtual file system (VFS) with a common root
directory. Media sources are then referenced by a relative path within this root directory.

The informational properties of a collection are defined by `COLLECTION_TITLE` and `COLLECTION_KIND`
(optional). If this collection does not exist it will be created on first run. All subsequent runs
will display an error message that the collection could not be created, ignore them.

The VFS root directory of the collection is specified by `COLLECTION_VFS_ROOT_URL`. Currently only
`file://` URLs are supported for this purpose.

## Ingest Data

Run the script whenever metadata in audio files has been modified:

```shell
scripts/synchronize_collection_vfs.sh
```

You can safely interrupt the script at any point or stop the web server manually at your will. The
database will catch up and resynchronize its contents on the next run. Easy and fool-proof.

## Query Data

Start the web server manually and access the web API at `<WEBSRV_URL>/api`. Please refer to the
OpenAPI documentation in [`openapi.yaml`](../websrv/res/openapi.yaml).
