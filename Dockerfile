# aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

# rust:stable-slim, clux/muslrust: /usr/src
# ekidd/rust-musl-builder: /home/rust/src
ARG WORKDIR_ROOT=/usr/src

ARG PROJECT_NAME=aoide

ARG BUILD_TARGET=x86_64-unknown-linux-musl

ARG BUILD_MODE=release

ARG BUILD_BIN=aoide


###############################################################################
# 1st Build Stage
FROM rust:slim AS build

# Import global ARGs
ARG WORKDIR_ROOT
ARG PROJECT_NAME
ARG BUILD_TARGET
ARG BUILD_MODE
ARG BUILD_BIN

# Enable select features for the workspace build or leave empty
# for using the default features
# Example: "--features feature-foobar"
ARG WORKSPACE_BUILD_AND_TEST_ARGS="--all-features"

# Enable all features and targets for the individual project checks
ARG PROJECT_CHECK_ARGS="--all-targets --all-features"

# Enable all features in the executable
ARG BUILD_BIN_ARGS="--all-features"

# Prepare for musl libc build target
RUN apt update \
    && apt install --no-install-recommends -y \
        musl-tools \
        tree \
    && rm -rf /var/lib/apt/lists/* \
    && rustup target add ${BUILD_TARGET}

# Docker build cache: Create and build an empty dummy workspace with all
# external dependencies to avoid redownloading them on subsequent builds
# if unchanged.

# Create workspace directory
RUN mkdir -p ${WORKDIR_ROOT}/${PROJECT_NAME}
WORKDIR ${WORKDIR_ROOT}/${PROJECT_NAME}

# Create all sub-crates in workspace
RUN mkdir -p crates && \
    USER=root cargo new --lib ${PROJECT_NAME}-client && \
    mv ${PROJECT_NAME}-client crates/client && \
    USER=root cargo new --lib ${PROJECT_NAME}-core && \
    mv ${PROJECT_NAME}-core crates/core && \
    USER=root cargo new --lib ${PROJECT_NAME}-core-serde && \
    mv ${PROJECT_NAME}-core-serde crates/core-serde && \
    USER=root cargo new --lib ${PROJECT_NAME}-core-ext && \
    mv ${PROJECT_NAME}-core-ext crates/core-ext && \
    USER=root cargo new --lib ${PROJECT_NAME}-core-ext-serde && \
    mv ${PROJECT_NAME}-core-ext-serde crates/core-ext-serde && \
    USER=root cargo new --lib ${PROJECT_NAME}-jsonapi-sqlite && \
    mv ${PROJECT_NAME}-jsonapi-sqlite crates/jsonapi-sqlite && \
    USER=root cargo new --lib ${PROJECT_NAME}-media && \
    mv ${PROJECT_NAME}-media crates/media && \
    USER=root cargo new --lib ${PROJECT_NAME}-repo && \
    mv ${PROJECT_NAME}-repo crates/repo && \
    USER=root cargo new --lib ${PROJECT_NAME}-repo-sqlite && \
    mv ${PROJECT_NAME}-repo-sqlite crates/repo-sqlite && \
    USER=root cargo new --lib ${PROJECT_NAME}-usecases && \
    mv ${PROJECT_NAME}-usecases crates/usecases && \
    USER=root cargo new --lib ${PROJECT_NAME}-websrv && \
    mv ${PROJECT_NAME}-websrv crates/websrv && \
    tree

COPY [ \
    "Cargo.toml", \
    "Cargo.lock", \
    "./" ]
COPY [ \
    "crates/client/Cargo.toml", \
    "./crates/client/" ]
COPY [ \
    "crates/core/Cargo.toml", \
    "./crates/core/" ]
COPY [ \
    "crates/core/benches", \
    "./crates/core/benches/" ]
COPY [ \
    "crates/core-serde/Cargo.toml", \
    "./crates/core-serde/" ]
COPY [ \
    "crates/core-ext/Cargo.toml", \
    "./crates/core-ext/" ]
COPY [ \
    "crates/core-ext-serde/Cargo.toml", \
    "./crates/core-ext-serde/" ]
COPY [ \
    "crates/jsonapi-sqlite/Cargo.toml", \
    "./crates/jsonapi-sqlite/" ]
COPY [ \
    "crates/media/Cargo.toml", \
    "./crates/media/" ]
COPY [ \
    "crates/repo/Cargo.toml", \
    "./crates/repo/" ]
COPY [ \
    "crates/repo-sqlite/Cargo.toml", \
    "./crates/repo-sqlite/" ]
COPY [ \
    "crates/usecases/Cargo.toml", \
    "./crates/usecases/" ]
COPY [ \
    "crates/websrv/Cargo.toml", \
    "./crates/websrv/" ]

# Build the workspace, then delete all build artefacts that must not(!) be cached
#
# - Note the special naming convention for all artefacts in deps/ that are referring
#   to the crate/project name: The character '-' must be substituted by '_',  i.e.
#   applying the same naming convention as for the corresponding imports in source
#   (.rs) files!
# - For each sub-project delete both the corresponding deps/ AND .fingerprint/
#   directories!
RUN tree && \
    cargo build --workspace ${WORKSPACE_BUILD_AND_TEST_ARGS} --${BUILD_MODE} --target ${BUILD_TARGET} && \
    rm -f ./target/${BUILD_TARGET}/${BUILD_MODE}/${PROJECT_NAME}* && \
    rm -f ./target/${BUILD_TARGET}/${BUILD_MODE}/deps/${PROJECT_NAME}-* && \
    rm -f ./target/${BUILD_TARGET}/${BUILD_MODE}/deps/${PROJECT_NAME}_* && \
    rm -rf ./target/${BUILD_TARGET}/${BUILD_MODE}/.fingerprint/${PROJECT_NAME}-* && \
    tree

# Copy all project (re-)sources that are required for building
COPY [ \
    "crates/client/src", \
    "./crates/client/src/" ]
COPY [ \
    "crates/core/src", \
    "./crates/core/src/" ]
COPY [ \
    "crates/core-serde/src", \
    "./crates/core-serde/src/" ]
COPY [ \
    "crates/core-ext/src", \
    "./crates/core-ext/src/" ]
COPY [ \
    "crates/core-ext-serde/src", \
    "./crates/core-ext-serde/src/" ]
COPY [ \
    "crates/jsonapi-sqlite/src", \
    "./crates/jsonapi-sqlite/src/" ]
COPY [ \
    "crates/media/src", \
    "./crates/media/src/" ]
COPY [ \
    "crates/repo/src", \
    "./crates/repo/src/" ]
COPY [ \
    "crates/repo-sqlite/src", \
    "./crates/repo-sqlite/src/" ]
COPY [ \
    "crates/repo-sqlite/migrations", \
    "./crates/repo-sqlite/migrations/" ]
COPY [ \
    "crates/usecases/src", \
    "./crates/usecases/src/" ]
COPY [ \
    "crates/websrv/res", \
    "./crates/websrv/res/" ]
COPY [ \
    "crates/websrv/src", \
    "./crates/websrv/src/" ]

# 1. Check all sub-projects using their local manifest for an isolated, standalone build
# 2. Build workspace and run all unit tests
# 3. Build the target binary
# 4. Strip debug infos from the executable
RUN tree && \
    cargo check -p aoide-client --manifest-path crates/client/Cargo.toml ${PROJECT_CHECK_ARGS} --${BUILD_MODE} && \
    cargo check -p aoide-core --manifest-path crates/core/Cargo.toml ${PROJECT_CHECK_ARGS} --${BUILD_MODE} && \
    cargo check -p aoide-core-serde --manifest-path crates/core-serde/Cargo.toml ${PROJECT_CHECK_ARGS} --${BUILD_MODE} && \
    cargo check -p aoide-core-ext --manifest-path crates/core-ext/Cargo.toml ${PROJECT_CHECK_ARGS} --${BUILD_MODE} && \
    cargo check -p aoide-core-ext-serde --manifest-path crates/core-ext-serde/Cargo.toml ${PROJECT_CHECK_ARGS} --${BUILD_MODE} && \
    cargo check -p aoide-jsonapi-sqlite --manifest-path crates/jsonapi-sqlite/Cargo.toml ${PROJECT_CHECK_ARGS} --${BUILD_MODE} && \
    cargo check -p aoide-media --manifest-path crates/media/Cargo.toml ${PROJECT_CHECK_ARGS} --${BUILD_MODE} && \
    cargo check -p aoide-repo --manifest-path crates/repo/Cargo.toml ${PROJECT_CHECK_ARGS} --${BUILD_MODE} && \
    cargo check -p aoide-repo-sqlite --manifest-path crates/repo-sqlite/Cargo.toml ${PROJECT_CHECK_ARGS} --${BUILD_MODE} && \
    cargo check -p aoide-usecases --manifest-path crates/usecases/Cargo.toml ${PROJECT_CHECK_ARGS} --${BUILD_MODE} && \
    cargo check -p aoide-websrv --manifest-path crates/websrv/Cargo.toml ${PROJECT_CHECK_ARGS} --${BUILD_MODE} && \
    cargo test --workspace ${WORKSPACE_BUILD_AND_TEST_ARGS} --${BUILD_MODE} --target ${BUILD_TARGET} && \
    cargo build -p aoide-websrv --manifest-path crates/websrv/Cargo.toml --bin ${BUILD_BIN} ${BUILD_BIN_ARGS} --${BUILD_MODE} --target ${BUILD_TARGET} && \
    strip ./target/${BUILD_TARGET}/${BUILD_MODE}/${BUILD_BIN}


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
