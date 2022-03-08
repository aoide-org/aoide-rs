# just manual: https://github.com/casey/just/#readme

# Ignore the .env file that is only used by the web service
set dotenv-load := false

_default:
    @just --list

# Format source code
fmt:
    cargo fmt --all
    cd webapp && cargo fmt

# Run clippy
check:
    cargo clippy --locked --workspace --bins --examples --tests -- -D warnings
    cargo clippy --locked --workspace --no-default-features --bins --examples --tests -- -D warnings
    cargo clippy --locked --workspace --all-features --bins --examples --tests -- -D warnings
    cd webapp && cargo clippy --target wasm32-unknown-unknown --locked --all-features --bins --examples --tests -- -D warnings

# Fix lint warnings
fix:
    cargo fix --workspace --all-features --bins --examples --tests
    cargo clippy --workspace --all-features --bins --examples --tests --fix
    cd webapp && cargo fix --target wasm32-unknown-unknown --all-features --examples --tests
    cd webapp && cargo clippy --target wasm32-unknown-unknown --all-features --examples --tests

# Run unit tests
test:
    RUST_BACKTRACE=1 cargo test --locked --workspace -- --nocapture
    RUST_BACKTRACE=1 cargo test --locked --workspace --no-default-features -- --nocapture
    RUST_BACKTRACE=1 cargo test --locked --workspace --all-features -- --nocapture
    cd webapp && RUST_BACKTRACE=1 cargo test --locked --all-features -- --nocapture

# Update depenencies and pre-commit hooks
update:
    rustup self update
    cargo install \
        cargo-edit \
        trunk
    cargo upgrade --workspace --exclude \
        aoide-client \
        aoide-core \
        aoide-core-api \
        aoide-core-api-json \
        aoide-core-json \
        aoide-media \
        aoide-repo \
        aoide-repo-sqlite \
        aoide-index-tantivy \
        aoide-storage-sqlite \
        aoide-usecases \
        aoide-usecases-sqlite \
        aoide-backend-webapi-json \
        aoide-websrv-api \
        libsqlite3-sys \
        triseratops
    #cargo minimal-versions check --workspace
    cargo update
    cd webapp && cargo update
    pip install -U pre-commit
    pre-commit autoupdate

# Run pre-commit hooks
pre-commit:
    pre-commit run --all-files

# Launch a debug build of the web service with the webapp enabled
debug-webapp:
    cd webapp && trunk build
    RUST_LOG=debug cargo run --package aoide-websrv --all-features
