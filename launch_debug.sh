#!/usr/bin/env bash
set -euo pipefail

# Change into directory where this shell script is located
SCRIPT_ROOT=$(cd -P -- "$(dirname -- "$0")" && pwd -P)
cd "${SCRIPT_ROOT}"

FEATURES=""

reset && cargo build ${FEATURES}
LOG_LEVEL=debug DATABASE_URL="file:///home/uk/.mixxx/aoide.sqlite" cargo run ${FEATURES}
