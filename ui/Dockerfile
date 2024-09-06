# syntax = docker/dockerfile:1
FROM node:22.8.0-bookworm AS base
WORKDIR /usr/src
COPY package.json yarn.lock .yarnrc.yml /usr/src/

FROM base AS development
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
ENV PUBLIC_URL=https://__HOARDER_PUBLIC_URL__
RUN --mount=type=tmpfs,target=/tmp \
    yarn build

FROM scratch AS production
WORKDIR /app
ARG PORT=3000
ENV PORT=$PORT
ENV NODE_ENV=production
ENV HOSTNAME=
COPY --link --from=release /lib/x86_64-linux-gnu/ld-linux-x86-64.* /lib/x86_64-linux-gnu/
COPY --link --from=release /lib/x86_64-linux-gnu/libc.so* /lib/x86_64-linux-gnu/
COPY --link --from=release /lib/x86_64-linux-gnu/libcrypto.so* /lib/x86_64-linux-gnu/
COPY --link --from=release /lib/x86_64-linux-gnu/libdl.so.* /lib/x86_64-linux-gnu/
COPY --link --from=release /lib/x86_64-linux-gnu/libgcc_s.so* /lib/x86_64-linux-gnu/
COPY --link --from=release /lib/x86_64-linux-gnu/libm.so* /lib/x86_64-linux-gnu/
COPY --link --from=release /lib/x86_64-linux-gnu/libpthread.so* /lib/x86_64-linux-gnu/
COPY --link --from=release /lib/x86_64-linux-gnu/libresolv.so* /lib/x86_64-linux-gnu/
COPY --link --from=release /lib/x86_64-linux-gnu/libssl.so* /lib/x86_64-linux-gnu/
COPY --link --from=release /lib/x86_64-linux-gnu/libstdc++.so.* /lib/x86_64-linux-gnu/
COPY --link --from=release /lib64 /lib64
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
