# Building

## Pre-requisites

Some build tools require _Python_.

## Setup

### Install the _Rust_ toolchain

Install the appropriate _Rust_ toolchain for your native OS/platform
by following these [instructions](https://www.rust-lang.org/tools/install).

We are using the `stable` toolchain only.

### Install `just`

Many development tasks have been automated with _recipes_ for
[`just`](https://github.com/casey/just) that can be found in
[`.justfile`](.justfile).

Install `just` through `cargo`:

```sh
cargo install just
```

### Install build tooling

The remaining setup of all build tooling is accomplished by running the following command:

```sh
just setup
```

## Build and run the web service

Use the following command to build the web service as a standalone executable

```sh
cargo run --profile dev --package aoide-websrv
```

### Build profiles

The following build profiles are available.

#### `dev`

- Default
- Slow at runtime, only for development and debugging
- Executable: `target/debug/aoide-websrv`

#### `release`

- Reasonably optimized and fast, both building and running
- Executable: `target/release/aoide-websrv`

#### `production`

- Optimized for maximum speed at runtime
- Slow to build
- Executable: `target/production/aoide-websrv`

### Run configuration

The last settings are stored in a local configuration file and could
be overridden by both environment variables and a _dotenv_ (`.env`) file.
Watch the log messages for details.

When started without any environment variables set the _Launcher UI_
will appear. Set `LAUNCH_HEADLESS=true` to suppress the _Launcher UI_.
