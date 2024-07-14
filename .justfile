# SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
        just \
        cargo-edit \
        cargo-hack \
        trunk
    pip install -U pre-commit
    #pre-commit install --hook-type commit-msg --hook-type pre-commit

# Upgrade (and update) dependencies
upgrade: setup
    pre-commit autoupdate
    cargo upgrade --incompatible --pinned
    cargo update

# Format source code
fmt:
    cargo fmt --all

# Run pre-commit hooks
pre-commit:
    pre-commit run --all-files

# Check all lib/bin projects individually with selected features (takes a long time)
check-crates:
    cargo hack --each-feature --exclude-features json-api check --locked --all-targets -p aoide
    cargo hack --feature-powerset check --locked --all-targets -p aoide-backend-embedded
    cargo hack --feature-powerset check --locked --all-targets -p aoide-backend-webapi-json
    cargo hack --feature-powerset --exclude-features js check --locked --all-targets -p aoide-core
    cargo hack --feature-powerset check --locked --all-targets -p aoide-core-api
    cargo check --locked --all-targets --features frontend -p aoide-core-api-json
    cargo check --locked --all-targets --features backend -p aoide-core-api-json
    cargo check --locked --all-targets --features backend,frontend,json-schema -p aoide-core-api-json
    cargo hack --feature-powerset check --locked --all-targets -p aoide-core-json
    cargo hack --feature-powerset check --locked --all-targets -p aoide-demo-app
    cargo hack --feature-powerset check --locked --all-targets -p aoide-desktop-app
    cargo hack --feature-powerset check --locked --all-targets -p aoide-media-file
    cargo hack --feature-powerset check --locked --all-targets -p aoide-repo
    cargo hack --feature-powerset check --locked --all-targets -p aoide-repo-sqlite
    cargo hack --feature-powerset check --locked --all-targets -p aoide-search-index-tantivy
    cargo hack --feature-powerset check --locked --all-targets -p aoide-storage-sqlite
    cargo hack --feature-powerset check --locked --all-targets -p aoide-usecases
    cargo hack --feature-powerset check --locked --all-targets -p aoide-usecases-sqlite
    cargo hack --feature-powerset check --locked --all-targets -p aoide-websrv
    cargo hack --feature-powerset check --locked --all-targets -p aoide-websrv-warp-sqlite

check-crates-wasm:
    cargo check --locked --all-targets --features js --target wasm32-unknown-unknown -p aoide-core
    cargo check --locked --all-targets --features js,serde --target wasm32-unknown-unknown -p aoide-core
    cargo check --locked --all-targets --target wasm32-unknown-unknown -p aoide-core-json
    cargo check --locked --all-targets --target wasm32-unknown-unknown -p aoide-core-api
    cargo check --locked --all-targets --features frontend --target wasm32-unknown-unknown -p aoide-core-api-json
    cargo check --locked --all-targets --features backend --target wasm32-unknown-unknown -p aoide-core-api-json
    cargo check --locked --all-targets --features backend,frontend --target wasm32-unknown-unknown -p aoide-core-api-json
    cargo check --locked --all-targets --target wasm32-unknown-unknown -p aoide-repo
    cargo check --locked --all-targets --target wasm32-unknown-unknown -p aoide-usecases

# Run clippy on the workspace (both dev and release profile)
clippy:
    cargo clippy --locked --workspace --all-targets --no-deps --profile dev -- -D warnings --cap-lints warn
    cargo clippy --locked --workspace --all-targets --no-deps --profile release -- -D warnings --cap-lints warn

# Fix lint warnings
fix:
    cargo fix --locked --workspace --all-targets --all-features
    cargo clippy --locked --workspace --no-deps --all-targets --all-features --fix

# Run tests
test:
    RUST_BACKTRACE=1 cargo test --locked --workspace -- --nocapture
    RUST_BACKTRACE=1 cargo test --locked --workspace --no-default-features -- --nocapture
    RUST_BACKTRACE=1 cargo test --locked --workspace --all-features -- --nocapture

depgraph-svg:
    cargo depgraph --all-features --focus aoide-core | dot -T svg -o aoide-depgraph.svg

publish-dry-run:
    cargo publish --dry-run --allow-dirty -p aoide-core
    cargo publish --dry-run --allow-dirty -p aoide-core-api
    cargo publish --dry-run --allow-dirty -p aoide-core-api-json
    cargo publish --dry-run --allow-dirty -p aoide-core-json
    cargo publish --dry-run --allow-dirty -p aoide-media
    cargo publish --dry-run --allow-dirty -p aoide-repo
    cargo publish --dry-run --allow-dirty -p aoide-usecases
    cargo publish --dry-run --allow-dirty -p aoide-storage-sqlite
    cargo publish --dry-run --allow-dirty -p aoide-repo-sqlite
    cargo publish --dry-run --allow-dirty -p aoide-usecases-sqlite
    cargo publish --dry-run --allow-dirty -p aoide-search-index-tantivy
    cargo publish --dry-run --allow-dirty -p aoide-backend-embedded
    cargo publish --dry-run --allow-dirty -p aoide-desktop-app
    cargo publish --dry-run --allow-dirty -p aoide
