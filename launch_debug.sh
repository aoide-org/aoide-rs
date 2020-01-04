#!/usr/bin/env bash
set -euo pipefail

# Change into directory where this shell script is located
SCRIPT_ROOT=$(cd -P -- "$(dirname -- "$0")" && pwd -P)
cd "${SCRIPT_ROOT}"

reset && cargo build --bin openfairdb
#cp openfair_without_subscriptions.db /tmp/openfair.db
ROCKET_PORT=6767 ROCKET_ADDRESS=127.0.0.1 RUST_LOG=debug target/debug/openfairdb --db-url /tmp/openfair.db
