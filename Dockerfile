# aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License as
# published by the Free Software Foundation, either version 3 of the
# License, or (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License
# along with this program.  If not, see <https:#www.gnu.org/licenses/>.

# Dockerfile for creating a statically-linked Rust application using Docker's
# multi-stage build feature. This also leverages the docker build cache to
# avoid re-downloading dependencies if they have not changed between builds.


###############################################################################
# Define global ARGs for all stages

ARG WORKDIR_ROOT=/usr/src

ARG PROJECT_NAME=aoide

ARG BUILD_TARGET=x86_64-unknown-linux-musl

ARG BUILD_MODE=release

ARG BUILD_BIN=aoide


###############################################################################
# 1st Build Stage
FROM clux/muslrust:stable AS build

# Import global ARGs
ARG WORKDIR_ROOT
ARG PROJECT_NAME
ARG BUILD_TARGET
ARG BUILD_MODE
ARG BUILD_BIN

WORKDIR ${WORKDIR_ROOT}

# Docker build cache: Create and build an empty dummy project with all
# external dependencies to avoid redownloading them on subsequent builds
# if unchanged.
RUN USER=root cargo new --bin ${PROJECT_NAME}
WORKDIR ${WORKDIR_ROOT}/${PROJECT_NAME}

RUN mkdir -p "./src/bin/${BUILD_BIN}" \
    && \
    mv ./src/main.rs "./src/bin/${BUILD_BIN}" \
    && \
    USER=root cargo new --lib ${PROJECT_NAME}-core \
    && \
    mv ${PROJECT_NAME}-core core \
    && \
    USER=root cargo new --lib ${PROJECT_NAME}-core-serde \
    && \
    mv ${PROJECT_NAME}-core-serde core-serde \
    && \
    USER=root cargo new --lib ${PROJECT_NAME}-repo \
    && \
    mv ${PROJECT_NAME}-repo repo \
    && \
    USER=root cargo new --lib ${PROJECT_NAME}-repo-sqlite \
    && \
    mv ${PROJECT_NAME}-repo-sqlite repo-sqlite

COPY [ \
    "Cargo.toml", \
    "Cargo.lock", \
    "./" ]
COPY [ \
    "core/Cargo.toml", \
    "./core/" ]
COPY [ \
    "core/benches", \
    "./core/benches/" ]
COPY [ \
    "core-serde/Cargo.toml", \
    "./core-serde/" ]
COPY [ \
    "repo/Cargo.toml", \
    "./repo/" ]
COPY [ \
    "repo-sqlite/Cargo.toml", \
    "./repo-sqlite/" ]

# Build the dummy project(s), then delete all build artefacts that must(!) not be cached
RUN cargo build --${BUILD_MODE} --target ${BUILD_TARGET} --workspace \
    && \
    rm -f ./target/${BUILD_TARGET}/${BUILD_MODE}/${PROJECT_NAME}* \
    && \
    rm -f ./target/${BUILD_TARGET}/${BUILD_MODE}/deps/${PROJECT_NAME}-* \
    && \
    rm -rf ./target/${BUILD_TARGET}/${BUILD_MODE}/.fingerprint/${PROJECT_NAME}-*

# Copy all project (re-)sources that are required for building
COPY [ \
    "src", \
    "./src/" ]
COPY [ \
    "resources", \
    "./resources/" ]
COPY [ \
    "core/src", \
    "./core/src/" ]
COPY [ \
    "core-serde/src", \
    "./core-serde/src/" ]
COPY [ \
    "repo/src", \
    "./repo/src/" ]
COPY [ \
    "repo-sqlite/src", \
    "./repo-sqlite/src/" ]
COPY [ \
    "repo-sqlite/migrations", \
    "./repo-sqlite/migrations/" ]

# Test and build the actual project
RUN cargo test --${BUILD_MODE} --target ${BUILD_TARGET} --workspace \
    && \
    cargo build --${BUILD_MODE} --target ${BUILD_TARGET} --bin ${BUILD_BIN} \
    && \
    strip ./target/${BUILD_TARGET}/${BUILD_MODE}/${BUILD_BIN}

# Switch back to the root directory
#
# NOTE(2019-08-30, uklotzde): Otherwise copying from the build image fails
# during all subsequent builds of the 2nd stage with an unchanged 1st stage
# image. Tested with podman 1.5.x on Fedora 30.
WORKDIR /


###############################################################################
# 2nd Build Stage
FROM scratch

# Import global ARGs
ARG WORKDIR_ROOT
ARG PROJECT_NAME
ARG BUILD_TARGET
ARG BUILD_MODE
ARG BUILD_BIN

ARG DATA_VOLUME="/data"

ARG EXPOSE_PORT=8080

# Copy the statically-linked executable into the minimal scratch image
COPY --from=build [ \
    "${WORKDIR_ROOT}/${PROJECT_NAME}/target/${BUILD_TARGET}/${BUILD_MODE}/${BUILD_BIN}", \
    "./entrypoint" ]

VOLUME [ ${DATA_VOLUME} ]

EXPOSE ${EXPOSE_PORT}

# Wire the exposed port
ENV ENDPOINT_PORT ${EXPOSE_PORT}

ENTRYPOINT [ "./entrypoint" ]
