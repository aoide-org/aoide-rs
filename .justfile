# SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
# SPDX-License-Identifier: CC0-1.0

# just manual: https://github.com/casey/just/#readme

# Ignore the .env file that is only used by the web service
set dotenv-load := false

_default:
    @just --list

# Set up (and update) tooling
setup:
    # Ignore rustup failures, because not everyone might use it
    rustup self update || true
    # cargo-edit is needed for `cargo upgrade`
    cargo install \
        cargo-edit \
        trunk
    pip install -U pre-commit
    pre-commit autoupdate
    #pre-commit install --hook-type commit-msg --hook-type pre-commit

# Format source code
fmt:
    cargo fmt --all
    cd webapp && cargo fmt

# Run pre-commit hooks
pre-commit:
    pre-commit run --all-files

# Upgrade (and update) dependencies
upgrade:
    cargo update
    cargo upgrade --workspace \
        --exclude aoide-backend-embedded \
        --exclude aoide-backend-webapi-json \
        --exclude aoide-client \
        --exclude aoide-core \
        --exclude aoide-core-api \
        --exclude aoide-core-api-json \
        --exclude aoide-core-json \
        --exclude aoide-desktop-app \
        --exclude aoide-media \
        --exclude aoide-repo \
        --exclude aoide-repo-sqlite \
        --exclude aoide-search-index-tantivy \
        --exclude aoide-storage-sqlite \
        --exclude aoide-usecases \
        --exclude aoide-usecases-sqlite \
        --exclude aoide-websrv-warp-sqlite \
        --exclude libsqlite3-sys
    cargo update
    cd webapp \
        && cargo update \
        && cargo upgrade \
            --exclude aoide-core \
            --exclude aoide-core-api \
            --exclude aoide-core-api-json \
            --exclude aoide-core-json \
        && cargo update
    #cargo minimal-versions check --workspace

# Check all lib/bin projects individually with selected features (takes a long time)
check:
    cargo check --locked --all-targets -p aoide-backend-embedded
    cargo check --locked --all-targets -p aoide-backend-embedded --all-features
    cargo check --locked --all-targets -p aoide-backend-webapi-json
    cargo check --locked --all-targets -p aoide-backend-webapi-json --all-features
    cargo check --locked --all-targets -p aoide-client
    cargo check --locked --all-targets -p aoide-client --all-features
    cargo check --locked --all-targets -p aoide-core
    cargo check --locked --all-targets -p aoide-core-api-json --features backend
    cargo check --locked --all-targets -p aoide-core-api-json --features frontend
    cargo check --locked --all-targets -p aoide-core-api-json --all-features
    cargo check --locked --all-targets -p aoide-core-json
    cargo check --locked --all-targets -p aoide-core-json --all-features
    cargo check --locked --all-targets -p aoide-desktop-app
    cargo check --locked --all-targets -p aoide-desktop-app --all-features
    cargo check --locked --all-targets -p aoide-media --all-features
    cargo check --locked --all-targets -p aoide-media --no-default-features
    cargo check --locked --all-targets -p aoide-repo --all-features
    cargo check --locked --all-targets -p aoide-repo-sqlite --all-features
    cargo check --locked --all-targets -p aoide-search-index-tantivy --all-features
    cargo check --locked --all-targets -p aoide-storage-sqlite --all-features
    cargo check --locked --all-targets -p aoide-usecases
    cargo check --locked --all-targets -p aoide-usecases --features media
    cargo check --locked --all-targets -p aoide-usecases-sqlite --all-features
    cargo check --locked --all-targets -p aoide-websrv-warp-sqlite --all-features
    cargo check --locked --all-targets -p aoide-webcli --all-features
    cargo check --locked --all-targets -p aoide-websrv --all-features

check-wasm:
    cargo check --locked --all-targets --target wasm32-unknown-unknown --features js -p aoide-core
    cargo check --locked --all-targets --target wasm32-unknown-unknown --features js -p aoide-core-api
    cargo check --locked --all-targets --target wasm32-unknown-unknown --features js,backend -p aoide-core-api-json
    cargo check --locked --all-targets --target wasm32-unknown-unknown --features js,frontend -p aoide-core-api-json
    cargo check --locked --all-targets --target wasm32-unknown-unknown --features js -p aoide-core-json
    cargo check --locked --all-targets --target wasm32-unknown-unknown --features js -p aoide-repo
    cargo check --locked --all-targets --target wasm32-unknown-unknown --features js -p aoide-usecases

# Run clippy on the workspace (both dev and release profile)
clippy:
    cargo clippy --locked --workspace --all-targets --no-deps --profile dev -- -D warnings --cap-lints warn
    cargo clippy --locked --workspace --all-targets --no-deps --profile release -- -D warnings --cap-lints warn
    cd webapp && cargo clippy --locked --no-deps --target wasm32-unknown-unknown --all-targets --all-features --profile dev -- -D warnings --cap-lints warn
    cd webapp && cargo clippy --locked --no-deps --target wasm32-unknown-unknown --all-targets --all-features --profile release -- -D warnings --cap-lints warn

# Fix lint warnings
fix:
    cargo fix --locked --workspace --all-targets --all-features
    cargo clippy --locked --workspace --no-deps --all-targets --all-features --fix
    cd webapp && cargo fix --locked --target wasm32-unknown-unknown --all-targets --all-features
    cd webapp && cargo clippy --locked --no-deps --target wasm32-unknown-unknown --all-targets --all-features --fix

# Run tests
test:
    RUST_BACKTRACE=1 cargo test --locked --workspace -- --nocapture
    RUST_BACKTRACE=1 cargo test --locked --workspace --no-default-features -- --nocapture
    RUST_BACKTRACE=1 cargo test --locked --workspace --all-features -- --nocapture
    cd webapp && RUST_BACKTRACE=1 cargo test --locked --all-features -- --nocapture

# Launch a debug build of the web service with the webapp enabled
debug-webapp:
    cd webapp && trunk build
    RUST_LOG=debug cargo run --package aoide-websrv --all-features

depgraph-svg:
    cargo depgraph --all-features --focus aoide-core | dot -T svg -o aoide-depgraph.svg
