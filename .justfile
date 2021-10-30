# just manual: https://github.com/casey/just/#readme

# Ignore the .env file that is only used by the web service
set dotenv-load := false

_default:
    @just --list

# Run clippy
check:
    cargo clippy --locked --workspace --all-features -- -D warnings

# Run unit tests
test:
    cargo test --locked --workspace --all-features

# Fix lint warnings
fix:
    cargo fix --workspace --all-features
    cargo clippy --workspace --all-features --fix

# Update depenencies and pre-commit hooks
update:
    cargo update --aggressive
    pip install -U pre-commit
    pre-commit autoupdate

# Run pre-commit hooks
pre-commit:
    pre-commit run --all-files

# Launch a debug build of the web service with the webapp enabled
debug-webapp:
    cd webapp && trunk build
    RUST_LOG=debug cargo run --package aoide-websrv --all-features
