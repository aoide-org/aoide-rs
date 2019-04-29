FROM alpine:latest

RUN apk --no-cache add \
    ca-certificates

ARG APP_USER=aoide
ARG APP_GROUP=aoide

# Both UID and GID should match with their corresponding
# twins on the host system for read/write access of files
# in the data volume, e.g. the SQLite database.
ARG APP_UID=1000
ARG APP_GID=1000

ARG APP_HOME=/aoide

RUN addgroup -S $APP_GROUP -g $APP_GID && \
    adduser  -S $APP_USER -G aoide -u $APP_UID -h $APP_HOME

WORKDIR $APP_HOME

# TODO (if available): Add flag --chown=$APP_USER:$APP_GROUP
# TODO: Remove hard-coded target "x86_64-unknown-linux-musl"
COPY [ \
    "bin/x86_64-unknown-linux-musl/aoide", \
    "docker-entrypoint.sh", \
    "./" ]

VOLUME [ \
    "./data" ]

EXPOSE 8080

USER $APP_USER

# A shell script is needed to evaluate arguments in form of
# environment variables at runtime. This is the reason why
# we cannot use "FROM scratch" for this image.
ENTRYPOINT [ "./docker-entrypoint.sh" ]
