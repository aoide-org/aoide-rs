# just manual: https://github.com/casey/just/#readme

# Ignore the .env file that is only used by the web service
set dotenv-load := false

_default:
    @just --list

# Set up (and update) tooling
setup:
    rustup self update
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

# Upgrade (and update) depenencies
upgrade:
    cargo upgrade --workspace \
        --exclude aoide-backend-embedded \
        --exclude aoide-backend-webapi-json \
        --exclude aoide-client \
        --exclude aoide-core \
        --exclude aoide-core-api \
        --exclude aoide-core-api-json \
        --exclude aoide-core-json \
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
    cd webapp && cargo update
    #cargo minimal-versions check --workspace

# Check all crates individually (takes a long time)
check:
    cargo check --locked --all-targets --all-features -p aoide-backend-embedded
    cargo check --locked --all-targets --no-default-features -p aoide-backend-embedded
    cargo check --locked --all-targets --all-features -p aoide-backend-webapi-json
    cargo check --locked --all-targets --all-features -p aoide-client
    cargo check --locked --all-targets -p aoide-core
    cargo check --locked --all-targets --target wasm32-unknown-unknown -p aoide-core
    cargo check --locked --all-targets -p aoide-core-api
    cargo check --locked --all-targets --target wasm32-unknown-unknown -p aoide-core-api
    cargo check --locked --all-targets --features backend -p aoide-core-api-json
    cargo check --locked --all-targets --features frontend -p aoide-core-api-json
    cargo check --locked --all-targets --all-features -p aoide-core-api-json
    cargo check --locked --all-targets --all-features --target wasm32-unknown-unknown -p aoide-core-api-json
    cargo check --locked --all-targets -p aoide-core-json
    cargo check --locked --all-targets --all-features -p aoide-core-json
    cargo check --locked --all-targets --target wasm32-unknown-unknown -p aoide-core-json
    cargo check --locked --all-targets --all-features -p aoide-media
    cargo check --locked --all-targets --no-default-features -p aoide-media
    cargo check --locked --all-targets --all-features -p aoide-repo
    cargo check --locked --all-targets --all-features -p aoide-repo-sqlite
    cargo check --locked --all-targets --all-features -p aoide-search-index-tantivy
    cargo check --locked --all-targets --all-features -p aoide-storage-sqlite
    cargo check --locked --all-targets --all-features -p aoide-usecases
    cargo check --locked --all-targets --all-features -p aoide-usecases-sqlite
    cargo check --locked --all-targets --all-features -p aoide-webcli
    cargo check --locked --all-targets --all-features -p aoide-websrv
    cargo check --locked --all-targets --all-features -p aoide-websrv-warp-sqlite

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
