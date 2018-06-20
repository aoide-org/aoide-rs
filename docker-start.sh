#!/bin/sh -x

AOIDE_HOST=${AOIDE_HOST:-localhost}

AOIDE_DB_FILE="${AOIDE_DB_FILE:-aoide.sqlite}"

# Number of 'v' characters controls the verbosity
# aka log level. The default log level is INFO.
AOIDE_VERBOSITY="${AOIDE_VERBOSITY:-vv}"

# The default port is EXPOSEDd in the Dockerfile
AOIDE_PORT=${AOIDE_PORT:-8080}

# This path is defined as a VOLUME in the Dockerfile
AOIDE_DATA=/data

/aoide -${AOIDE_VERBOSITY} --listen ${AOIDE_HOST}:${AOIDE_PORT} "${AOIDE_DATA}/${AOIDE_DB_FILE}"
