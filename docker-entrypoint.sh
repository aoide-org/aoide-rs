#!/bin/sh -x

# Number of 'v' characters controls the verbosity
# aka log level. The default log level is INFO.
AOIDE_VERBOSITY="${AOIDE_VERBOSITY:-vv}"

# Default: IPv6 wildcard address "[::]" (IPv4: "0.0.0.0")
AOIDE_HOST=${AOIDE_HOST:-[::]}

# The default port is EXPOSEDd in the Dockerfile and should
# should not be changed.
AOIDE_PORT=${AOIDE_PORT:-7878}

# This path is defined as a VOLUME in the Dockerfile
AOIDE_DATA="${HOME}/data"

AOIDE_DB_URL="${AOIDE_DB_URL:-file://${AOIDE_DATA}/aoide.sqlite}"

exec "${HOME}/aoide" \
    -${AOIDE_VERBOSITY} \
    --listen ${AOIDE_HOST}:${AOIDE_PORT} \
    "${AOIDE_DB_URL}"
