# SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
# SPDX-License-Identifier: AGPL-3.0-or-later

# Dockerfile for creating a statically-linked Rust application using Docker's
# multi-stage build feature. This also leverages the docker build cache to
# avoid re-downloading dependencies if they have not changed between builds.


###############################################################################
# Define global ARGs for all stages

# rust:stable-slim, clux/muslrust: /usr/src
# ekidd/rust-musl-builder: /home/rust/src
ARG WORKDIR_ROOT=/usr/src

ARG WORKSPACE_NAME=aoide
ARG PACKAGE_NAME=aoide-websrv

ARG BUILD_TARGET=x86_64-unknown-linux-musl

# The corresponding target directory must match the profile name!
# Counterexample: For the `dev` profile the directory is named `debug`.
ARG BUILD_PROFILE=production

###############################################################################
# 1st Build Stage
# The Alpine-based image already contains the musl-libc toolchain
# that is needed for our BUILD_TARGET.
FROM rust:alpine AS build

# Import global ARGs
ARG WORKDIR_ROOT
ARG WORKSPACE_NAME
ARG PACKAGE_NAME
ARG BUILD_TARGET
ARG BUILD_PROFILE

ARG WORKSPACE_BUILD_AND_TEST_ARGS="--workspace --locked --all-targets --target ${BUILD_TARGET} --profile ${BUILD_PROFILE}"

# Prepare for musl libc build target
#
# Dependencies for pre-commit:
#  - git
#  - python3-pip
# Dependencies for egui:
#  - <https://github.com/emilk/egui/blob/master/.github/workflows/rust.yml>:
# Dependencies for building freetype-sys (needed by egui):
#  - make
#  - cmake
#  - g++
#  - libfontconfig-dev
# FIXME: Build of freetype-sys fails due to missing musl-g++ wrapper
# <https://bugs.debian.org/cgi-bin/bugreport.cgi?bug=988837>
# FIXME: Remove the symbolic link hack for usl-g++
RUN apk add --no-cache \
        tree \
        musl-dev \
        git py3-pip python3-dev nodejs npm bash \
        libxcb-dev libxkbcommon-dev \
        make cmake g++ fontconfig-dev \
    && rustup target add \
        ${BUILD_TARGET} \
    && rustup show \
    && rustup component add \
        rustfmt \
        clippy \
    && rustup component list --installed \
    && pip install pre-commit

# Docker build cache: Create and build an empty dummy workspace with all
# external dependencies to avoid redownloading them on subsequent builds
# if unchanged.

# Create workspace directory
RUN mkdir -p ${WORKDIR_ROOT}/${WORKSPACE_NAME}
WORKDIR ${WORKDIR_ROOT}/${WORKSPACE_NAME}

# Create all projects and crates in workspace
RUN USER=root \
    mkdir -p crates && \
    cargo new --vcs none --lib ${WORKSPACE_NAME}-backend-embedded && \
    mv ${WORKSPACE_NAME}-backend-embedded crates/backend-embedded && \
    cargo new --vcs none --lib ${WORKSPACE_NAME}-backend-webapi-json && \
    mv ${WORKSPACE_NAME}-backend-webapi-json crates/backend-webapi-json && \
    cargo new --vcs none --lib ${WORKSPACE_NAME}-client && \
    mv ${WORKSPACE_NAME}-client crates/client && \
    cargo new --vcs none --lib ${WORKSPACE_NAME}-core && \
    mv ${WORKSPACE_NAME}-core crates/core && \
    cargo new --vcs none --lib ${WORKSPACE_NAME}-core-json && \
    mv ${WORKSPACE_NAME}-core-json crates/core-json && \
    cargo new --vcs none --lib ${WORKSPACE_NAME}-core-api && \
    mv ${WORKSPACE_NAME}-core-api crates/core-api && \
    cargo new --vcs none --lib ${WORKSPACE_NAME}-core-api-json && \
    mv ${WORKSPACE_NAME}-core-api-json crates/core-api-json && \
    cargo new --vcs none --lib ${WORKSPACE_NAME}-desktop-app && \
    mv ${WORKSPACE_NAME}-desktop-app crates/desktop-app && \
    cargo new --vcs none --lib ${WORKSPACE_NAME}-media && \
    mv ${WORKSPACE_NAME}-media crates/media && \
    cargo new --vcs none --lib ${WORKSPACE_NAME}-repo && \
    mv ${WORKSPACE_NAME}-repo crates/repo && \
    cargo new --vcs none --lib ${WORKSPACE_NAME}-repo-sqlite && \
    mv ${WORKSPACE_NAME}-repo-sqlite crates/repo-sqlite && \
    cargo new --vcs none --lib ${WORKSPACE_NAME}-search-index-tantivy && \
    mv ${WORKSPACE_NAME}-search-index-tantivy crates/search-index-tantivy && \
    cargo new --vcs none --lib ${WORKSPACE_NAME}-storage-sqlite && \
    mv ${WORKSPACE_NAME}-storage-sqlite crates/storage-sqlite && \
    cargo new --vcs none --lib ${WORKSPACE_NAME}-usecases && \
    mv ${WORKSPACE_NAME}-usecases crates/usecases && \
    cargo new --vcs none --lib ${WORKSPACE_NAME}-usecases-sqlite && \
    mv ${WORKSPACE_NAME}-usecases-sqlite crates/usecases-sqlite && \
    cargo new --vcs none --lib ${WORKSPACE_NAME}-websrv-warp-sqlite && \
    mv ${WORKSPACE_NAME}-websrv-warp-sqlite crates/websrv-warp-sqlite && \
    cargo new --vcs none --bin ${WORKSPACE_NAME}-webcli && \
    mv ${WORKSPACE_NAME}-webcli webcli && \
    cargo new --vcs none --bin ${WORKSPACE_NAME}-websrv && \
    mv ${WORKSPACE_NAME}-websrv websrv && \
    tree -a

COPY [ \
    "Cargo.toml", \
    "Cargo.lock", \
    "./" ]
COPY [ \
    "crates/backend-embedded/Cargo.toml", \
    "./crates/backend-embedded/" ]
COPY [ \
    "crates/backend-webapi-json/Cargo.toml", \
    "./crates/backend-webapi-json/" ]
COPY [ \
    "crates/client/Cargo.toml", \
    "./crates/client/" ]
COPY [ \
    "crates/core/Cargo.toml", \
    "./crates/core/" ]
COPY [ \
    "crates/core-json/Cargo.toml", \
    "./crates/core-json/" ]
COPY [ \
    "crates/core-api/Cargo.toml", \
    "./crates/core-api/" ]
COPY [ \
    "crates/core-api-json/Cargo.toml", \
    "./crates/core-api-json/" ]
COPY [ \
    "crates/media/Cargo.toml", \
    "./crates/media/" ]
COPY [ \
    "crates/desktop-app/Cargo.toml", \
    "./crates/desktop-app/" ]
COPY [ \
    "crates/repo/Cargo.toml", \
    "./crates/repo/" ]
COPY [ \
    "crates/repo-sqlite/Cargo.toml", \
    "./crates/repo-sqlite/" ]
COPY [ \
    "crates/search-index-tantivy/Cargo.toml", \
    "./crates/search-index-tantivy/" ]
COPY [ \
    "crates/storage-sqlite/Cargo.toml", \
    "./crates/storage-sqlite/" ]
COPY [ \
    "crates/usecases/Cargo.toml", \
    "./crates/usecases/" ]
COPY [ \
    "crates/usecases-sqlite/Cargo.toml", \
    "./crates/usecases-sqlite/" ]
COPY [ \
    "crates/websrv-warp-sqlite/Cargo.toml", \
    "./crates/websrv-warp-sqlite/" ]
COPY [ \
    "webcli/Cargo.toml", \
    "./webcli/" ]
COPY [ \
    "websrv/Cargo.toml", \
    "./websrv/" ]

# Build the workspace dependencies, then delete all build artefacts that must not(!) be cached
#
# - Note the special naming convention for all artefacts in deps/ that are referring
#   to the crate/project name: The character '-' must be substituted by '_',  i.e.
#   applying the same naming convention as for the corresponding imports in source
#   (.rs) files!
# - For each sub-project delete both the corresponding deps/ AND .fingerprint/
#   directories!
RUN tree -a && \
    CARGO_INCREMENTAL=0 cargo build ${WORKSPACE_BUILD_AND_TEST_ARGS} && \
    rm -f ./target/${BUILD_PROFILE}/${WORKSPACE_NAME}* && \
    rm -f ./target/${BUILD_PROFILE}/deps/${WORKSPACE_NAME}-* && \
    rm -f ./target/${BUILD_PROFILE}/deps/${WORKSPACE_NAME}_* && \
    rm -rf ./target/${BUILD_PROFILE}/.fingerprint/${WORKSPACE_NAME}-* && \
    tree -a

# Copy all project (re-)sources that are required for pre-commit and building
COPY [ \
    "Cargo.lock.license", \
    ".codespellignore", \
    ".commitlintrc.json", \
    ".commitlintrc.json.license", \
    ".gitignore", \
    ".markdownlint-cli2.yaml", \
    ".pre-commit-config.yaml", \
    ".prettierrc.yaml", \
    ".rustfmt.toml", \
    "./" ]
COPY [ \
    "LICENSES", \
    "./LICENSES/" ]
COPY [ \
    "crates/backend-embedded/src", \
    "./crates/backend-embedded/src/" ]
COPY [ \
    "crates/backend-webapi-json/src", \
    "./crates/backend-webapi-json/src/" ]
COPY [ \
    "crates/client/src", \
    "./crates/client/src/" ]
COPY [ \
    "crates/core/src", \
    "./crates/core/src/" ]
COPY [ \
    "crates/core-json/src", \
    "./crates/core-json/src/" ]
COPY [ \
    "crates/core-api/src", \
    "./crates/core-api/src/" ]
COPY [ \
    "crates/core-api-json/src", \
    "./crates/core-api-json/src/" ]
COPY [ \
    "crates/media/src", \
    "./crates/media/src/" ]
COPY [ \
    "crates/desktop-app/src", \
    "./crates/desktop-app/src/" ]
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
    "crates/search-index-tantivy/src", \
    "./crates/search-index-tantivy/src/" ]
COPY [ \
    "crates/storage-sqlite/src", \
    "./crates/storage-sqlite/src/" ]
COPY [ \
    "crates/usecases/src", \
    "./crates/usecases/src/" ]
COPY [ \
    "crates/usecases-sqlite/src", \
    "./crates/usecases-sqlite/src/" ]
COPY [ \
    "crates/websrv-warp-sqlite/src", \
    "./crates/websrv-warp-sqlite/src/" ]
COPY [ \
    "webcli/src", \
    "./webcli/src/" ]
COPY [ \
    "websrv/res", \
    "./websrv/res/" ]
COPY [ \
    "websrv/src", \
    "./websrv/src/" ]

# Print theresulting file system structure
RUN tree -a

# Run pre-commit (requires a temporary Git repo)
RUN sed -i 's|id: prettier|id: prettier\n        language_version: system|g' .pre-commit-config.yaml && \
    sed -i 's|id: markdownlint-cli2|id: markdownlint-cli2\n        language_version: system|g' .pre-commit-config.yaml && \
    git config --global user.email "pre-commit@example.com" && \
    git config --global user.name "pre-commit" && \
    git config --global init.defaultBranch main && \
    git init && git add . && git commit -m "pre-commit" && \
    SKIP=no-commit-to-branch pre-commit run --all-files && \
    rm -rf .git

# Build workspace and unit tests
RUN export CARGO_INCREMENTAL=0 && \
    cargo test ${WORKSPACE_BUILD_AND_TEST_ARGS} --no-run

# Run unit tests
RUN export CARGO_INCREMENTAL=0 && \
    cargo test ${WORKSPACE_BUILD_AND_TEST_ARGS} -- --nocapture --quiet

# Build the target binary with default features and strip debug infos from the executable
RUN export CARGO_INCREMENTAL=0 && \
    cargo build --locked --target ${BUILD_TARGET} --profile ${BUILD_PROFILE} --package ${PACKAGE_NAME} --manifest-path websrv/Cargo.toml && \
    strip ./target/${BUILD_TARGET}/${BUILD_PROFILE}/${PACKAGE_NAME}


###############################################################################
# 2nd Build Stage
FROM scratch

# Import global ARGs
ARG WORKDIR_ROOT
ARG WORKSPACE_NAME
ARG PACKAGE_NAME
ARG BUILD_TARGET
ARG BUILD_PROFILE

ARG DATA_VOLUME="/data"
VOLUME [ ${DATA_VOLUME} ]

# Copy the statically-linked executable into the minimal scratch image
COPY --from=build [ \
    "${WORKDIR_ROOT}/${WORKSPACE_NAME}/target/${BUILD_TARGET}/${BUILD_PROFILE}/${PACKAGE_NAME}", \
    "./entrypoint" ]

ENTRYPOINT [ "./entrypoint" ]
