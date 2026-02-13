# syntax = docker/dockerfile:1
FROM rust:1.93.1-trixie AS base
WORKDIR /usr/src
RUN --mount=type=cache,id=api:/var/cache/apt,target=/var/cache/apt \
    --mount=type=cache,id=api:/var/lib/apt/lists,target=/var/lib/apt/lists \
    apt-get -y update && \
    apt-get -y install \
        libjemalloc-dev
COPY . ./

FROM base AS debug
RUN --mount=type=cache,id=api:/usr/local/cargo/registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=api:/usr/src/target,target=/usr/src/target \
    JEMALLOC_OVERRIDE=/usr/lib/$(dpkg-architecture --query DEB_BUILD_MULTIARCH)/libjemalloc.so \
        cargo build && \
    cp target/debug/hoarder /usr/local/bin/hoarder

FROM base AS release
RUN --mount=type=cache,id=api:/usr/local/cargo/registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=api:/usr/src/target,target=/usr/src/target \
    JEMALLOC_OVERRIDE=/usr/lib/$(dpkg-architecture --query DEB_BUILD_MULTIARCH)/libjemalloc.so \
        cargo build --release && \
    cp target/release/hoarder /usr/local/bin/hoarder

FROM scratch AS production-linux-amd64
ARG DEB_BUILD_MULTIARCH=x86_64-linux-gnu
ARG LD=/lib64/ld-linux-x86-64.so.2

FROM scratch AS production-linux-arm64
ARG DEB_BUILD_MULTIARCH=aarch64-linux-gnu
ARG LD=/lib/ld-linux-aarch64.so.1

FROM production-${TARGETPLATFORM//\//-} AS production
ARG PORT=80
ENV PORT=$PORT
ENV PATH=/
COPY --link --from=release $LD $LD
COPY --link --from=release /lib/$DEB_BUILD_MULTIARCH/libc.so.* /lib/$DEB_BUILD_MULTIARCH/
COPY --link --from=release /lib/$DEB_BUILD_MULTIARCH/libcrypto.so.* /lib/$DEB_BUILD_MULTIARCH/
COPY --link --from=release /lib/$DEB_BUILD_MULTIARCH/libgcc_s.so.* /lib/$DEB_BUILD_MULTIARCH/
COPY --link --from=release /lib/$DEB_BUILD_MULTIARCH/libjemalloc.so.* /lib/$DEB_BUILD_MULTIARCH/
COPY --link --from=release /lib/$DEB_BUILD_MULTIARCH/libm.so.* /lib/$DEB_BUILD_MULTIARCH/
COPY --link --from=release /lib/$DEB_BUILD_MULTIARCH/libssl.so.* /lib/$DEB_BUILD_MULTIARCH/
COPY --link --from=release /lib/$DEB_BUILD_MULTIARCH/libstdc++.so.* /lib/$DEB_BUILD_MULTIARCH/
COPY --link --from=release /lib/$DEB_BUILD_MULTIARCH/libz.so.* /lib/$DEB_BUILD_MULTIARCH/
COPY --link --from=release /lib/$DEB_BUILD_MULTIARCH/libzstd.so.* /lib/$DEB_BUILD_MULTIARCH/
COPY --link --from=release /usr/local/bin/hoarder /hoarder
COPY --link --from=release /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --link --from=release /usr/share/zoneinfo /usr/share/zoneinfo
RUN --mount=from=release,src=/bin,dst=/bin \
    --mount=from=release,src=/lib,dst=/lib \
    --mount=from=release,src=/usr,dst=/usr \
    /bin/mkdir --mode=1777 /tmp
EXPOSE $PORT
RUN ["hoarder", "--version"]
ENTRYPOINT ["hoarder"]
