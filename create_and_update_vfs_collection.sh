#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME=$(basename "$0" .sh)

# Change into directory where this shell script is located
SCRIPT_ROOT=$(cd -P -- "$(dirname -- "$0")" && pwd -P)

DATE_STAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
cd "${SCRIPT_ROOT}"

# Build profile(s)
BUILD_PROFILE="production"
#BUILD_PROFILE="dev"

# Log levels and backtrace options
WEBCLI_LOG=info
WEBSRV_LOG=info
RUST_BACKTRACE=1

# Selected features
FEATURES="--all-features"

# Server endpoint
SERVICE_URL="http://[::1]:8080"

# Redirection of server logs
OUT_DIR=/tmp/

# Database URL
DATABASE_URL="file:///tmp/aoide.sqlite"
#DATABASE_URL="file://${HOME}/.mixxx/aoide.sqlite"

# Collection properties
COLLECTION_KIND="mixxx.org"
COLLECTION_TITLE="My Collection"
#COLLECTION_TITLE="1st Collection"

# Music directory
XDG_MUSIC_DIR=${XDG_MUSIC_DIR:-${HOME}/Music}
COLLECTION_VFS_ROOT_URL="file://${XDG_MUSIC_DIR}/"
#COLLECTION_VFS_ROOT_URL="file://${XDG_MUSIC_DIR}/Collections/1st"

# Optional file name for exporting all tracks from the database
# into a single JSON file.
#EXPORT_TRACKS_JSON_FILE="/tmp/aoide_tracks_${DATE_STAMP}.json"

#######################################################################
### No need to change anything below!                               ###
#######################################################################

# Clear screen
reset

# Ensure that EXPORT_TRACKS_JSON_FILE is not unbound
EXPORT_TRACKS_JSON_FILE=${EXPORT_TRACKS_JSON_FILE:-""}

echo "BUILD_PROFILE          : ${BUILD_PROFILE}"
echo "OUT_DIR                : ${OUT_DIR}"
echo "SERVICE_URL            : ${SERVICE_URL}"
echo "DATABASE_URL           : ${DATABASE_URL}"
echo "COLLECTION_KIND        : ${COLLECTION_KIND}"
echo "COLLECTION_TITLE       : ${COLLECTION_TITLE}"
echo "COLLECTION_VFS_ROOT_URL: ${COLLECTION_VFS_ROOT_URL}"
echo "EXPORT_TRACKS_JSON_FILE: ${EXPORT_TRACKS_JSON_FILE}"

# Kill the server if it is still running
pkill --signal SIGINT -x aoide-websrv || true

sleep 1

cargo build --workspace --profile ${BUILD_PROFILE} ${FEATURES}

LOG_FILE=${OUT_DIR}${SCRIPT_NAME}@${DATE_STAMP}.log
PERF_FILE=${OUT_DIR}${SCRIPT_NAME}@${DATE_STAMP}.perf

#perf record -F 99 -g -o "${PERF_FILE}" --

RUST_BACKTRACE=${RUST_BACKTRACE} \
RUST_LOG=${WEBSRV_LOG} \
DATABASE_URL=${DATABASE_URL} \
cargo run --package aoide-websrv --profile ${BUILD_PROFILE} ${FEATURES} &>"${LOG_FILE}" &

sleep 1

cd crates/client
export RUST_BACKTRACE=${RUST_BACKTRACE}
export RUST_LOG=${WEBCLI_LOG}

# Ignore failures when trying to recreate existing collections
cargo run --package aoide-webcli --profile ${BUILD_PROFILE} ${FEATURES} -- --service-url "${SERVICE_URL}" collections create-mixxx "${COLLECTION_TITLE}" "${COLLECTION_VFS_ROOT_URL}" || true

# Synchronize the collection with the file system
cargo run --package aoide-webcli --profile ${BUILD_PROFILE} ${FEATURES} -- --service-url "${SERVICE_URL}" media-tracker scan-directories "${COLLECTION_VFS_ROOT_URL}"
cargo run --package aoide-webcli --profile ${BUILD_PROFILE} ${FEATURES} -- --service-url "${SERVICE_URL}" media-tracker untrack-orphaned-directories "${COLLECTION_VFS_ROOT_URL}"
cargo run --package aoide-webcli --profile ${BUILD_PROFILE} ${FEATURES} -- --service-url "${SERVICE_URL}" media-tracker import-files "${COLLECTION_VFS_ROOT_URL}"
cargo run --package aoide-webcli --profile ${BUILD_PROFILE} ${FEATURES} -- --service-url "${SERVICE_URL}" media-sources purge-untracked "${COLLECTION_VFS_ROOT_URL}"
cargo run --package aoide-webcli --profile ${BUILD_PROFILE} ${FEATURES} -- --service-url "${SERVICE_URL}" media-sources purge-orphaned "${COLLECTION_VFS_ROOT_URL}"
cargo run --package aoide-webcli --profile ${BUILD_PROFILE} ${FEATURES} -- --service-url "${SERVICE_URL}" media-tracker find-untracked-files "${COLLECTION_VFS_ROOT_URL}"

# Exports all tracks
if [ -n "${EXPORT_TRACKS_JSON_FILE}" ]
then
    echo "Exporting tracks into JSON file ${EXPORT_TRACKS_JSON_FILE}"
    cargo run --package aoide-webcli --profile ${BUILD_PROFILE} ${FEATURES} -- --service-url "${SERVICE_URL}" tracks export-all-into-file -o "${EXPORT_TRACKS_JSON_FILE}"
fi

sleep 2

pkill --signal SIGINT -x aoide-websrv || true

sleep 1

if [ -f "${PERF_FILE}" ]
then
    perf report -i "${PERF_FILE}"
fi
