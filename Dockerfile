#FROM scratch
FROM alpine:latest
RUN apk --no-cache add ca-certificates

COPY ./target/x86_64-unknown-linux-musl/release/aoide /aoide
COPY ./resources /resources
COPY ./docker-start.sh /docker-start.sh

VOLUME /data

EXPOSE 8080

ENTRYPOINT [ "/bin/sh", "-c", "/docker-start.sh" ]
