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
    cargo clippy --locked --workspace --all-features --bins --examples --tests -- -D warnings
    cd webapp && cargo clippy --locked --all-features --bins --examples --tests -- -D warnings

# Fix lint warnings
fix:
    cargo fix --workspace --all-features --bins --examples --tests
    cargo clippy --workspace --all-features --bins --examples --tests --fix
    cd webapp && cargo fix --all-features --examples --tests
    cd webapp && cargo clippy --all-features --examples --tests

# Run unit tests
test:
    cargo test --locked --workspace --all-features
    cd webapp && cargo test --locked --all-features

# Update depenencies and pre-commit hooks
update:
    rustup self update
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