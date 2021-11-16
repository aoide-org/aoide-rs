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

# Enable all features and targets for the individual project checks
ARG PROJECT_CHECK_ARGS="--locked --all-features --bins --examples --target ${BUILD_TARGET} --${BUILD_MODE}"

# Enable select features for the workspace build or leave empty
# for using the default features
# Example: "--features feature-foobar"
ARG WORKSPACE_BUILD_AND_TEST_ARGS="--locked --all-features --target ${BUILD_TARGET} --${BUILD_MODE}"

# Enable all features in the executable
ARG BUILD_BIN_ARGS="--locked --all-features --target ${BUILD_TARGET} --${BUILD_MODE}"

# Prepare for musl libc build target
# git and python3-pip are required for pre-commit
RUN apt update \
    && apt install --no-install-recommends -y \
        git \
        musl-tools \
        python3-pip \
        tree \
    && rm -rf /var/lib/apt/lists/* \
    && rustup target add \
        ${BUILD_TARGET} \
        wasm32-unknown-unknown \
    && rustup show \
    && rustup component add \
        rustfmt \
        clippy \
    && rustup component list --installed \
    && cargo install --locked trunk \
    && pip install pre-commit

# Docker build cache: Create and build an empty dummy workspace with all
# external dependencies to avoid redownloading them on subsequent builds
# if unchanged.

# Create workspace directory
RUN mkdir -p ${WORKDIR_ROOT}/${PROJECT_NAME}
WORKDIR ${WORKDIR_ROOT}/${PROJECT_NAME}

# Create all projects and crates in workspace
RUN USER=root cargo new --vcs none --lib ${PROJECT_NAME}-websrv && \
    mv ${PROJECT_NAME}-websrv websrv && \
    mkdir -p crates && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-client && \
    mv ${PROJECT_NAME}-client crates/client && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-core && \
    mv ${PROJECT_NAME}-core crates/core && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-core-serde && \
    mv ${PROJECT_NAME}-core-serde crates/core-serde && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-core-ext && \
    mv ${PROJECT_NAME}-core-ext crates/core-ext && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-core-ext-serde && \
    mv ${PROJECT_NAME}-core-ext-serde crates/core-ext-serde && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-jsonapi-sqlite && \
    mv ${PROJECT_NAME}-jsonapi-sqlite crates/jsonapi-sqlite && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-media && \
    mv ${PROJECT_NAME}-media crates/media && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-repo && \
    mv ${PROJECT_NAME}-repo crates/repo && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-repo-sqlite && \
    mv ${PROJECT_NAME}-repo-sqlite crates/repo-sqlite && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-usecases && \
    mv ${PROJECT_NAME}-usecases crates/usecases && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-usecases-sqlite && \
    mv ${PROJECT_NAME}-usecases-sqlite crates/usecases-sqlite && \
    tree -a

COPY [ \
    "Cargo.toml", \
    "Cargo.lock", \
    "./" ]
COPY [ \
    "websrv/Cargo.toml", \
    "./websrv/" ]
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
    "crates/usecases-sqlite/Cargo.toml", \
    "./crates/usecases-sqlite/" ]

# Build the workspace dependencies, then delete all build artefacts that must not(!) be cached
#
# - Note the special naming convention for all artefacts in deps/ that are referring
#   to the crate/project name: The character '-' must be substituted by '_',  i.e.
#   applying the same naming convention as for the corresponding imports in source
#   (.rs) files!
# - For each sub-project delete both the corresponding deps/ AND .fingerprint/
#   directories!
RUN tree -a && \
    CARGO_INCREMENTAL=0 cargo build --workspace ${WORKSPACE_BUILD_AND_TEST_ARGS} && \
    rm -f ./target/${BUILD_TARGET}/${BUILD_MODE}/${PROJECT_NAME}* && \
    rm -f ./target/${BUILD_TARGET}/${BUILD_MODE}/deps/${PROJECT_NAME}-* && \
    rm -f ./target/${BUILD_TARGET}/${BUILD_MODE}/deps/${PROJECT_NAME}_* && \
    rm -rf ./target/${BUILD_TARGET}/${BUILD_MODE}/.fingerprint/${PROJECT_NAME}-* && \
    tree -a

# Copy all project (re-)sources that are required for pre-commit and building
COPY [ \
    ".clippy.toml", \
    ".codespellignore", \
    ".gitignore", \
    ".markdownlint-cli2.yaml", \
    ".pre-commit-config.yaml", \
    ".rustfmt.toml", \
    "./" ]
COPY [ \
    "webapp", \
    "./webapp/" ]
COPY [ \
    "websrv/res", \
    "./websrv/res/" ]
COPY [ \
    "websrv/src", \
    "./websrv/src/" ]
COPY [ \
    "crates/client/src", \
    "./crates/client/src/" ]
COPY [ \
    "crates/client/examples", \
    "./crates/client/examples/" ]
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
    "crates/usecases-sqlite/src", \
    "./crates/usecases-sqlite/src/" ]

# 1. Run pre-commit
# 2. Check all sub-projects using their local manifest for an isolated, standalone build
# 3. Build workspace and run all unit tests
# 4. Build the target binary
# 5. Strip debug infos from the executable
RUN tree -a && \
    export CARGO_INCREMENTAL=0 && \
    cd webapp && trunk build && cd - && \
    git config --global user.email "pre-commit@example.com" && \
    git config --global user.name "pre-commit" && \
    git init && git add . && git commit -m "pre-commit" && \
    CARGO_BUILD_TARGET=${BUILD_TARGET} pre-commit run --all-files && \
    rm -rf .git && \
    cargo check -p aoide-client --manifest-path crates/client/Cargo.toml ${PROJECT_CHECK_ARGS} && \
    cargo check -p aoide-core --manifest-path crates/core/Cargo.toml ${PROJECT_CHECK_ARGS} && \
    cargo check -p aoide-core-serde --manifest-path crates/core-serde/Cargo.toml ${PROJECT_CHECK_ARGS} && \
    cargo check -p aoide-core-ext --manifest-path crates/core-ext/Cargo.toml ${PROJECT_CHECK_ARGS} && \
    cargo check -p aoide-core-ext-serde --manifest-path crates/core-ext-serde/Cargo.toml ${PROJECT_CHECK_ARGS} && \
    cargo check -p aoide-jsonapi-sqlite --manifest-path crates/jsonapi-sqlite/Cargo.toml ${PROJECT_CHECK_ARGS} && \
    cargo check -p aoide-media --manifest-path crates/media/Cargo.toml ${PROJECT_CHECK_ARGS} && \
    cargo check -p aoide-repo --manifest-path crates/repo/Cargo.toml ${PROJECT_CHECK_ARGS} && \
    cargo check -p aoide-repo-sqlite --manifest-path crates/repo-sqlite/Cargo.toml ${PROJECT_CHECK_ARGS} && \
    cargo check -p aoide-usecases --manifest-path crates/usecases/Cargo.toml ${PROJECT_CHECK_ARGS} && \
    cargo check -p aoide-usecases-sqlite --manifest-path crates/usecases-sqlite/Cargo.toml ${PROJECT_CHECK_ARGS} && \
    cargo check -p aoide-websrv --manifest-path websrv/Cargo.toml -${PROJECT_CHECK_ARGS} && \
    cargo test --workspace ${WORKSPACE_BUILD_AND_TEST_ARGS} --no-run && \
    cargo test --workspace ${WORKSPACE_BUILD_AND_TEST_ARGS} -- --nocapture --quiet && \
    cargo build -p aoide-websrv --manifest-path websrv/Cargo.toml --bin ${BUILD_BIN} ${BUILD_BIN_ARGS} && \
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
