# syntax = docker/dockerfile:1
FROM rust:1.75.0-bookworm AS development
WORKDIR /usr/src
COPY . ./
RUN --mount=type=cache,id=api:/usr/local/cargo/registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=api:/usr/src/target,target=/usr/src/target \
    cargo build && \
    cp -r target/debug/hoarder /usr/local/bin/hoarder

FROM development AS release
RUN --mount=type=cache,id=api:/usr/local/cargo/registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=api:/usr/src/target,target=/usr/src/target \
    cargo build --release && \
    cp -r target/release/hoarder /usr/local/bin/hoarder

FROM scratch AS production
ARG PORT=80
ENV PORT=$PORT
COPY --link --from=release /lib/x86_64-linux-gnu/ld-linux-x86-64.* /lib/x86_64-linux-gnu/
COPY --link --from=release /lib/x86_64-linux-gnu/libc.so* /lib/x86_64-linux-gnu/
COPY --link --from=release /lib/x86_64-linux-gnu/libgcc_s.so* /lib/x86_64-linux-gnu/
COPY --link --from=release /lib/x86_64-linux-gnu/libm.so* /lib/x86_64-linux-gnu/
COPY --link --from=release /lib64 /lib64
COPY --link --from=release /usr/local/bin/hoarder /hoarder
COPY --link --from=release /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --link --from=release /usr/share/zoneinfo /usr/share/zoneinfo
EXPOSE 80
CMD ["/hoarder"]
