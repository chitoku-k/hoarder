# syntax = docker/dockerfile:1
FROM rust:1.85.1-bookworm AS base
WORKDIR /usr/src
RUN --mount=type=cache,id=api:/var/cache/apt,target=/var/cache/apt \
    --mount=type=cache,id=api:/var/lib/apt/lists,target=/var/lib/apt/lists \
    rustup component add clippy && \
    apt-get -y update && \
    apt-get -y install \
        libjemalloc-dev
COPY . ./

FROM base AS debug
ENV JEMALLOC_OVERRIDE=/usr/lib/x86_64-linux-gnu/libjemalloc.so
RUN --mount=type=cache,id=api:/usr/local/cargo/registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=api:/usr/src/target,target=/usr/src/target \
    cargo build && \
    cp -r target/debug/hoarder /usr/local/bin/hoarder

FROM base AS release
ENV JEMALLOC_OVERRIDE=/usr/lib/x86_64-linux-gnu/libjemalloc.so
RUN --mount=type=cache,id=api:/usr/local/cargo/registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=api:/usr/src/target,target=/usr/src/target \
    cargo build --release && \
    cp -r target/release/hoarder /usr/local/bin/hoarder

FROM scratch AS production
ARG PORT=80
ENV PORT=$PORT
ENV PATH=/
COPY --link --from=release /lib/x86_64-linux-gnu/ld-linux-x86-64.so.* /lib/x86_64-linux-gnu/
COPY --link --from=release /lib/x86_64-linux-gnu/libc.so.* /lib/x86_64-linux-gnu/
COPY --link --from=release /lib/x86_64-linux-gnu/libcrypto.so.* /lib/x86_64-linux-gnu/
COPY --link --from=release /lib/x86_64-linux-gnu/libgcc_s.so.* /lib/x86_64-linux-gnu/
COPY --link --from=release /lib/x86_64-linux-gnu/libjemalloc.so.* /lib/x86_64-linux-gnu/
COPY --link --from=release /lib/x86_64-linux-gnu/libm.so.* /lib/x86_64-linux-gnu/
COPY --link --from=release /lib/x86_64-linux-gnu/libssl.so.* /lib/x86_64-linux-gnu/
COPY --link --from=release /lib/x86_64-linux-gnu/libstdc++.so.* /lib/x86_64-linux-gnu/
COPY --link --from=release /lib64 /lib64
COPY --link --from=release /usr/local/bin/hoarder /hoarder
COPY --link --from=release /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --link --from=release /usr/share/zoneinfo /usr/share/zoneinfo
RUN --mount=from=release,src=/bin,dst=/bin \
    --mount=from=release,src=/lib,dst=/lib \
    --mount=from=release,src=/usr,dst=/usr \
    /bin/mkdir --mode=1777 /tmp
EXPOSE $PORT
ENTRYPOINT ["hoarder"]
