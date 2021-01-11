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
ARG WORKSPACE_BUILD_FEATURES=""

# Enable all features and targets for the individual project checks
ARG PROJECT_CHECK_FEATURES="--all-targets --all-features"

# Prepare for musl libc build target
RUN apt update \
    && apt install --no-install-recommends -y musl-tools tree \
    && rm -rf /var/lib/apt/lists/* \
    && rustup target add ${BUILD_TARGET}

WORKDIR ${WORKDIR_ROOT}

# Docker build cache: Create and build an empty dummy workspace with all
# external dependencies to avoid redownloading them on subsequent builds
# if unchanged.

# Create workspace with main project
WORKDIR ${WORKDIR_ROOT}
RUN USER=root cargo new --bin ${PROJECT_NAME}

# Create all sub-projects in workspace
WORKDIR ${WORKDIR_ROOT}/${PROJECT_NAME}
RUN mkdir -p "./src/bin/${BUILD_BIN}" && \
    mv ./src/main.rs "./src/bin/${BUILD_BIN}" && \
    USER=root cargo new --lib ${PROJECT_NAME}-core && \
    mv ${PROJECT_NAME}-core core && \
    USER=root cargo new --lib ${PROJECT_NAME}-core-serde && \
    mv ${PROJECT_NAME}-core-serde core-serde && \
    USER=root cargo new --lib ${PROJECT_NAME}-media && \
    mv ${PROJECT_NAME}-media media && \
    USER=root cargo new --lib ${PROJECT_NAME}-repo && \
    mv ${PROJECT_NAME}-repo repo && \
    USER=root cargo new --lib ${PROJECT_NAME}-repo-sqlite && \
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
    "media/Cargo.toml", \
    "./media/" ]
COPY [ \
    "repo/Cargo.toml", \
    "./repo/" ]
COPY [ \
    "repo-sqlite/Cargo.toml", \
    "./repo-sqlite/" ]

# Build the dummy project, then delete all build artefacts that must not(!) be cached
#
# - Note the special naming convention for all artefacts in deps/ that are referring
#   to the crate/project name: The character '-' must be substituted by '_',  i.e.
#   applying the same naming convention as for the corresponding imports in source
#   (.rs) files!
# - For each sub-project delete both the corresponding deps/ AND .fingerprint/
#   directories!
RUN tree && \
    cargo build --${BUILD_MODE} --target ${BUILD_TARGET} --workspace && \
    rm -f ./target/${BUILD_TARGET}/${BUILD_MODE}/${PROJECT_NAME}* && \
    rm -f ./target/${BUILD_TARGET}/${BUILD_MODE}/deps/${PROJECT_NAME}-* && \
    rm -f ./target/${BUILD_TARGET}/${BUILD_MODE}/deps/${PROJECT_NAME}-* && \
    rm -rf ./target/${BUILD_TARGET}/${BUILD_MODE}/.fingerprint/${PROJECT_NAME}-* && \
    rm -f ./target/${BUILD_TARGET}/${BUILD_MODE}/deps/aoide_core-* && \
    rm -rf ./target/${BUILD_TARGET}/${BUILD_MODE}/.fingerprint/aoide-core-* && \
    rm -f ./target/${BUILD_TARGET}/${BUILD_MODE}/deps/aoide_core_serde-* && \
    rm -rf ./target/${BUILD_TARGET}/${BUILD_MODE}/.fingerprint/aoide-core-serde-* && \
    rm -f ./target/${BUILD_TARGET}/${BUILD_MODE}/deps/aoide_media-* && \
    rm -rf ./target/${BUILD_TARGET}/${BUILD_MODE}/.fingerprint/aoide-media-* && \
    rm -f ./target/${BUILD_TARGET}/${BUILD_MODE}/deps/aoide_repo-* && \
    rm -rf ./target/${BUILD_TARGET}/${BUILD_MODE}/.fingerprint/aoide-repo-* && \
    rm -f ./target/${BUILD_TARGET}/${BUILD_MODE}/deps/aoide_repo_sqlite-* && \
    rm -rf ./target/${BUILD_TARGET}/${BUILD_MODE}/.fingerprint/aoide-repo-sqlite-* && \
    tree

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
    "media/src", \
    "./media/src/" ]
COPY [ \
    "repo/src", \
    "./repo/src/" ]
COPY [ \
    "repo-sqlite/src", \
    "./repo-sqlite/src/" ]
COPY [ \
    "repo-sqlite/migrations", \
    "./repo-sqlite/migrations/" ]

# 1. Check all sub-projects using their local manifest for an isolated, standalone build
# 2. Build workspace and run all unit tests
# 3. Build the target binary
# 4. Strip debug infos from the executable
RUN tree && \
    cargo check -p aoide-core --manifest-path core/Cargo.toml --${BUILD_MODE} ${PROJECT_CHECK_FEATURES} && \
    cargo check -p aoide-core-serde --manifest-path core-serde/Cargo.toml --${BUILD_MODE} ${PROJECT_CHECK_FEATURES} && \
    cargo check -p aoide-media --manifest-path media/Cargo.toml --${BUILD_MODE} ${PROJECT_CHECK_FEATURES} && \
    cargo check -p aoide-repo --manifest-path repo/Cargo.toml --${BUILD_MODE} ${PROJECT_CHECK_FEATURES} && \
    cargo check -p aoide-repo-sqlite --manifest-path repo-sqlite/Cargo.toml --${BUILD_MODE} ${PROJECT_CHECK_FEATURES} && \
    cargo test --workspace --${BUILD_MODE} --target ${BUILD_TARGET} ${WORKSPACE_BUILD_FEATURES} && \
    cargo build --bin ${BUILD_BIN} --${BUILD_MODE} --target ${BUILD_TARGET} ${WORKSPACE_BUILD_FEATURES} && \
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
