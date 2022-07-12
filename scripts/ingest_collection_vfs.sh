#!/usr/bin/env bash

# SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
# SPDX-License-Identifier: AGPL-3.0-or-later

set -euo pipefail

SCRIPT_NAME=$(basename "$0" .sh)

# Change into directory where this shell script is located
SCRIPT_ROOT=$(cd -P -- "$(dirname -- "$0")" && pwd -P)

DATE_STAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
cd "${SCRIPT_ROOT}"

#######################################################################
### Settings & Configuration                                        ###
#######################################################################

# Common definitions
IPV6_LOCALHOST="::1"
XDG_MUSIC_DIR=${XDG_MUSIC_DIR:-${HOME}/Music}

# Server endpoint
WEBSRV_PROTOCOL="http"
WEBSRV_HOST=${IPV6_LOCALHOST}
WEBSRV_PORT=8080
WEBSRV_URL="${WEBSRV_PROTOCOL}://[${WEBSRV_HOST}]:${WEBSRV_PORT}"

# Redirection of server logs
WEBSRV_LOG_DIR=/tmp/

# Database URL (SQLite)
DATABASE_URL="file:///tmp/aoide.sqlite"
#DATABASE_URL="file://${HOME}/.mixxx/aoide.sqlite"

# Collection properties
COLLECTION_TITLE="1st Collection"
#COLLECTION_KIND="mixxx.org"

# The music directory as a `file://` URL
COLLECTION_VFS_ROOT_URL="file://${XDG_MUSIC_DIR}/"
#COLLECTION_VFS_ROOT_URL="file://${XDG_MUSIC_DIR}/Collections/1st"

# The subdirectory or folder within the music directory that the
# media tracker should actually process. Only needs to be adjusted
# if processing should be restricted to a subdirectory.
# For most tasks this is an optional parameter and defaults to the
# collection's VFS root path.
#MEDIA_TASK_ROOT_URL=${COLLECTION_VFS_ROOT_URL}

# Optional file name for exporting all tracks from the database
# into a single JSON file. Note: This must be a simple file system
# path and not a `file://...` URL!
#EXPORT_TRACKS_JSON_FILE="/tmp/aoide_tracks_${DATE_STAMP}.json"

#######################################################################
### No need to change anything below (only for debugging)           ###
#######################################################################

# Build profile(s)
BUILD_PROFILE="production"
#BUILD_PROFILE="release"
#BUILD_PROFILE="dev"

# Log levels
WEBCLI_LOG=info
WEBSRV_LOG=info

RUST_BACKTRACE=1

# Selected features
FEATURES="--all-features"

# Clear screen
reset

# Ensure that all optional variables are bound
COLLECTION_KIND=${COLLECTION_KIND:-""}
EXPORT_TRACKS_JSON_FILE=${EXPORT_TRACKS_JSON_FILE:-""}
MEDIA_TASK_ROOT_URL=${MEDIA_TASK_ROOT_URL:-${COLLECTION_VFS_ROOT_URL}}

echo "BUILD_PROFILE           : ${BUILD_PROFILE}"
echo "WEBSRV_URL              : ${WEBSRV_URL}"
echo "WEBSRV_LOG_DIR          : ${WEBSRV_LOG_DIR}"
echo "DATABASE_URL            : ${DATABASE_URL}"
echo "COLLECTION_TITLE        : ${COLLECTION_TITLE}"
echo "COLLECTION_KIND         : ${COLLECTION_KIND}"
echo "COLLECTION_VFS_ROOT_URL : ${COLLECTION_VFS_ROOT_URL}"
echo "MEDIA_TASK_ROOT_URL     : ${MEDIA_TASK_ROOT_URL}"
if [ -n "${EXPORT_TRACKS_JSON_FILE}" ]
then
echo "EXPORT_TRACKS_JSON_FILE : ${EXPORT_TRACKS_JSON_FILE}"
fi

# Kill the server if it is still running
pkill --signal SIGINT -x aoide-websrv || true

sleep 1

LOG_FILE=${WEBSRV_LOG_DIR}${SCRIPT_NAME}@${DATE_STAMP}.log
PERF_FILE=${WEBSRV_LOG_DIR}${SCRIPT_NAME}@${DATE_STAMP}.perf

#perf record -F 99 -g -o "${PERF_FILE}" --

# Build the server executable (blocking)
cargo build --package aoide-websrv --profile ${BUILD_PROFILE} ${FEATURES}

# After the server executable has been built run it (in the background)
RUST_BACKTRACE=${RUST_BACKTRACE} \
RUST_LOG=${WEBSRV_LOG} \
DATABASE_URL=${DATABASE_URL} \
LAUNCH_HEADLESS=true \
cargo run --package aoide-websrv --profile ${BUILD_PROFILE} ${FEATURES} &>"${LOG_FILE}" &

sleep 2

export RUST_BACKTRACE=${RUST_BACKTRACE}
export RUST_LOG=${WEBCLI_LOG}

# Ignore failures when trying to recreate existing collections
cargo run --package aoide-webcli --profile ${BUILD_PROFILE} ${FEATURES} -- --websrv-url "${WEBSRV_URL}" create-collection --title "${COLLECTION_TITLE}" --kind "${COLLECTION_KIND}" --vfs-root-url "${COLLECTION_VFS_ROOT_URL}" || true

# Synchronize the collection with the file system
cargo run --package aoide-webcli --profile ${BUILD_PROFILE} ${FEATURES} -- --websrv-url "${WEBSRV_URL}" media-tracker scan-directories --collection-title "${COLLECTION_TITLE}" "${MEDIA_TASK_ROOT_URL}"
cargo run --package aoide-webcli --profile ${BUILD_PROFILE} ${FEATURES} -- --websrv-url "${WEBSRV_URL}" media-tracker untrack-orphaned-directories --collection-title "${COLLECTION_TITLE}" "${MEDIA_TASK_ROOT_URL}"
cargo run --package aoide-webcli --profile ${BUILD_PROFILE} ${FEATURES} -- --websrv-url "${WEBSRV_URL}" media-tracker import-files --collection-title "${COLLECTION_TITLE}" "${MEDIA_TASK_ROOT_URL}"
cargo run --package aoide-webcli --profile ${BUILD_PROFILE} ${FEATURES} -- --websrv-url "${WEBSRV_URL}" media-sources purge-untracked --collection-title "${COLLECTION_TITLE}" "${MEDIA_TASK_ROOT_URL}"
cargo run --package aoide-webcli --profile ${BUILD_PROFILE} ${FEATURES} -- --websrv-url "${WEBSRV_URL}" media-sources purge-orphaned --collection-title "${COLLECTION_TITLE}" "${MEDIA_TASK_ROOT_URL}"
cargo run --package aoide-webcli --profile ${BUILD_PROFILE} ${FEATURES} -- --websrv-url "${WEBSRV_URL}" media-tracker find-untracked-files --collection-title "${COLLECTION_TITLE}" "${MEDIA_TASK_ROOT_URL}"
cargo run --package aoide-webcli --profile ${BUILD_PROFILE} ${FEATURES} -- --websrv-url "${WEBSRV_URL}" tracks find-unsynchronized --collection-title "${COLLECTION_TITLE}"

# Exports all tracks
if [ -n "${EXPORT_TRACKS_JSON_FILE}" ]
then
    echo "Exporting tracks into JSON file ${EXPORT_TRACKS_JSON_FILE}"
    cargo run --package aoide-webcli --profile ${BUILD_PROFILE} ${FEATURES} -- --websrv-url "${WEBSRV_URL}" tracks export-all-into-file --collection-title "${COLLECTION_TITLE}" "${EXPORT_TRACKS_JSON_FILE}"
fi

sleep 2

pkill --signal SIGINT -x aoide-websrv || true

sleep 1

if [ -f "${PERF_FILE}" ]
then
    perf report -i "${PERF_FILE}"
fi
