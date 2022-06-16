# aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

# The corresponding target directory must match the profile name!
# Counterexample: For the `dev` profile the directory is named `debug`.
ARG BUILD_PROFILE=production

ARG BUILD_BIN=aoide-websrv

###############################################################################
# 1st Build Stage
FROM rust:slim AS build

# Import global ARGs
ARG WORKDIR_ROOT
ARG PROJECT_NAME
ARG BUILD_TARGET
ARG BUILD_PROFILE
ARG BUILD_BIN

ARG WORKSPACE_BUILD_AND_TEST_ARGS="--workspace --locked --all-targets --profile ${BUILD_PROFILE}"

# Prepare for musl libc build target
# git and python3-pip are required for pre-commit
RUN apt update \
    && apt install --no-install-recommends -y \
        git \
        musl-tools \
        python3-pip \
        tree \
        libxcb-render0-dev \
        libxcb-shape0-dev \
        libxcb-xfixes0-dev \
        libspeechd-dev \
        libxkbcommon-dev \
        libssl-dev \
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
RUN USER=root mkdir .cargo && \
    echo "[build]" > .cargo/config.toml && \
    echo "target = \"${BUILD_TARGET}\"" >> .cargo/config.toml && \
    cat .cargo/config.toml && \
    cargo new --vcs none --bin ${PROJECT_NAME}-websrv && \
    mv ${PROJECT_NAME}-websrv websrv && \
    cargo new --vcs none --bin ${PROJECT_NAME}-webcli && \
    mv ${PROJECT_NAME}-webcli webcli && \
    mkdir -p crates && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-backend-embedded && \
    mv ${PROJECT_NAME}-backend-embedded crates/backend-embedded && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-backend-webapi-json && \
    mv ${PROJECT_NAME}-backend-webapi-json crates/backend-webapi-json && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-client && \
    mv ${PROJECT_NAME}-client crates/client && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-core && \
    mv ${PROJECT_NAME}-core crates/core && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-core-json && \
    mv ${PROJECT_NAME}-core-json crates/core-json && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-core-api && \
    mv ${PROJECT_NAME}-core-api crates/core-api && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-core-api-json && \
    mv ${PROJECT_NAME}-core-api-json crates/core-api-json && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-media && \
    mv ${PROJECT_NAME}-media crates/media && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-repo && \
    mv ${PROJECT_NAME}-repo crates/repo && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-repo-sqlite && \
    mv ${PROJECT_NAME}-repo-sqlite crates/repo-sqlite && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-search-index-tantivy && \
    mv ${PROJECT_NAME}-search-index-tantivy crates/search-index-tantivy && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-storage-sqlite && \
    mv ${PROJECT_NAME}-storage-sqlite crates/storage-sqlite && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-usecases && \
    mv ${PROJECT_NAME}-usecases crates/usecases && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-usecases-sqlite && \
    mv ${PROJECT_NAME}-usecases-sqlite crates/usecases-sqlite && \
    USER=root cargo new --vcs none --lib ${PROJECT_NAME}-websrv-warp-sqlite && \
    mv ${PROJECT_NAME}-websrv-warp-sqlite crates/websrv-warp-sqlite && \
    tree -a

COPY [ \
    ".cargo", \
    "./.cargo/" ]
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
    rm -f ./target/${BUILD_PROFILE}/${PROJECT_NAME}* && \
    rm -f ./target/${BUILD_PROFILE}/deps/${PROJECT_NAME}-* && \
    rm -f ./target/${BUILD_PROFILE}/deps/${PROJECT_NAME}_* && \
    rm -rf ./target/${BUILD_PROFILE}/.fingerprint/${PROJECT_NAME}-* && \
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
    "webapp", \
    "./webapp/" ]
COPY [ \
    "webcli/src", \
    "./webcli/src/" ]
COPY [ \
    "websrv/res", \
    "./websrv/res/" ]
COPY [ \
    "websrv/src", \
    "./websrv/src/" ]

# 1. Run pre-commit
# 2. Build workspace and run all unit tests
# 3. Build the target binary
# 4. Strip debug infos from the executable
RUN tree -a && \
    export CARGO_INCREMENTAL=0 && \
    cd webapp && trunk build && cd - && \
    git config --global user.email "pre-commit@example.com" && \
    git config --global user.name "pre-commit" && \
    git config --global init.defaultBranch main && \
    git init && git add . && git commit -m "pre-commit" && \
    SKIP=no-commit-to-branch pre-commit run --all-files && \
    rm -rf .git && \
    cargo test ${WORKSPACE_BUILD_AND_TEST_ARGS} --no-run && \
    cargo test ${WORKSPACE_BUILD_AND_TEST_ARGS} -- --nocapture --quiet && \
    cargo build -p ${BUILD_BIN} --manifest-path websrv/Cargo.toml --locked --all-features --profile ${BUILD_PROFILE} && \
    strip ./target/${BUILD_PROFILE}/${BUILD_BIN}


###############################################################################
# 2nd Build Stage
FROM scratch

# Import global ARGs
ARG WORKDIR_ROOT
ARG PROJECT_NAME
ARG BUILD_PROFILE
ARG BUILD_BIN

ARG DATA_VOLUME="/data"

ARG EXPOSE_PORT=8080

# Copy the statically-linked executable into the minimal scratch image
COPY --from=build [ \
    "${WORKDIR_ROOT}/${PROJECT_NAME}/target/${BUILD_PROFILE}/${BUILD_BIN}", \
    "./entrypoint" ]

VOLUME [ ${DATA_VOLUME} ]

EXPOSE ${EXPOSE_PORT}

# Wire the exposed port
ENV ENDPOINT_PORT ${EXPOSE_PORT}

ENTRYPOINT [ "./entrypoint" ]
