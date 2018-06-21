FROM alpine:latest
RUN apk --no-cache add ca-certificates

COPY [ \
    "./target/x86_64-unknown-linux-musl/release/aoide", \
    "./docker-entrypoint.sh", \
    "/" ]

COPY [ \
    "./resources", \
    "/resources" ]

VOLUME [ \
    "/data" ]

EXPOSE 8080/tcp

# A shell script is needed to evaluate arguments in form of
# environment variables at runtime. This is the reason why
# we cannot use "FROM scratch" for this image.
ENTRYPOINT [ "/docker-entrypoint.sh" ]
