# syntax = docker/dockerfile:1
FROM node:24.10.0-trixie AS base
WORKDIR /usr/src
COPY package.json yarn.lock .yarnrc.yml /usr/src/

FROM base AS debug
RUN --mount=type=tmpfs,target=/tmp \
    --mount=type=cache,id=ui:/usr/local/share/.cache/yarn,target=/usr/local/share/.cache/yarn \
    corepack enable && \
    yarn --immutable

FROM base AS release
ENV NODE_ENV=production
RUN --mount=type=tmpfs,target=/tmp \
    --mount=type=cache,id=ui:/usr/local/share/.cache/yarn,target=/usr/local/share/.cache/yarn \
    corepack enable && \
    yarn --immutable
COPY . ./
RUN --mount=type=bind,from=schema,target=/usr/schema \
    yarn codegen
ENV API_URL=https://__HOARDER_API_URL__
RUN --mount=type=tmpfs,target=/tmp \
    yarn build

FROM scratch AS production
WORKDIR /app
ARG DEB_BUILD_MULTIARCH=x86_64-linux-gnu
ARG LD=/lib64/ld-linux-x86-64.so.2
ARG PORT=3000
ENV PORT=$PORT
ENV NODE_ENV=production
ENV HOSTNAME=
COPY --link --from=release $LD $LD
COPY --link --from=release /lib/$DEB_BUILD_MULTIARCH/libc.so* /lib/$DEB_BUILD_MULTIARCH/
COPY --link --from=release /lib/$DEB_BUILD_MULTIARCH/libcrypto.so* /lib/$DEB_BUILD_MULTIARCH/
COPY --link --from=release /lib/$DEB_BUILD_MULTIARCH/libdl.so.* /lib/$DEB_BUILD_MULTIARCH/
COPY --link --from=release /lib/$DEB_BUILD_MULTIARCH/libgcc_s.so* /lib/$DEB_BUILD_MULTIARCH/
COPY --link --from=release /lib/$DEB_BUILD_MULTIARCH/libm.so* /lib/$DEB_BUILD_MULTIARCH/
COPY --link --from=release /lib/$DEB_BUILD_MULTIARCH/libpthread.so* /lib/$DEB_BUILD_MULTIARCH/
COPY --link --from=release /lib/$DEB_BUILD_MULTIARCH/libresolv.so* /lib/$DEB_BUILD_MULTIARCH/
COPY --link --from=release /lib/$DEB_BUILD_MULTIARCH/libssl.so* /lib/$DEB_BUILD_MULTIARCH/
COPY --link --from=release /lib/$DEB_BUILD_MULTIARCH/libstdc++.so.* /lib/$DEB_BUILD_MULTIARCH/
COPY --link --from=release /lib/$DEB_BUILD_MULTIARCH/libz.so.* /lib/$DEB_BUILD_MULTIARCH/
COPY --link --from=release /lib/$DEB_BUILD_MULTIARCH/libzstd.so.* /lib/$DEB_BUILD_MULTIARCH/
COPY --link --from=release /usr/local/bin/node /node
COPY --link --from=release /etc/group /etc/group
COPY --link --from=release /etc/passwd /etc/passwd
COPY --link --from=release /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --link --from=release /usr/share/zoneinfo /usr/share/zoneinfo
COPY --chown=nobody:nogroup --from=release /usr/src/entrypoint.mjs /app/
COPY --chown=nobody:nogroup --from=release /usr/src/.next/standalone /app/
COPY --chown=nobody:nogroup --from=release /usr/src/.next/static /app/.next/static
RUN --mount=from=release,src=/bin,dst=/bin \
    --mount=from=release,src=/lib,dst=/lib \
    --mount=from=release,src=/usr,dst=/usr \
    /bin/mkdir --mode=1777 /tmp
USER nobody
EXPOSE $PORT
ENTRYPOINT ["/node", "entrypoint.mjs"]
