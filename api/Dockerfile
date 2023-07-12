# syntax = docker/dockerfile:1
FROM rust:1.70.0 AS build
WORKDIR /usr/src
COPY . ./
RUN --mount=type=cache,id=api:/usr/local/cargo/registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=api:/usr/src/target,target=/usr/src/target \
    cargo build && \
    cp ./target/*/hoarder /usr/local/bin/hoarder

FROM scratch AS production
ARG PORT=80
ENV PORT=$PORT
COPY --from=build /usr/local/bin/hoarder /hoarder
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=build /usr/share/zoneinfo /usr/share/zoneinfo
EXPOSE 80
CMD ["/hoarder"]
