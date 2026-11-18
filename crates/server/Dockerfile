FROM rust:1.98.1-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates crates
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked -p abstractions-server \
    && cp target/release/abstractions-server /abstractions-server

FROM --platform=$BUILDPLATFORM busybox:1.37 AS data
RUN mkdir /data && chown 10001:10001 /data

FROM scratch AS base
COPY --from=data --chown=10001:10001 /data /data
ENV ABSTRACTIONS_DATA=/data
VOLUME /data
EXPOSE 24650/udp
USER 10001
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --start-interval=2s \
    CMD ["/abstractions-server", "ping"]
ENTRYPOINT ["/abstractions-server"]

FROM base AS prebuilt
ARG TARGETARCH
COPY --chmod=755 dist/${TARGETARCH}/abstractions-server /abstractions-server

FROM base AS runtime
COPY --from=builder /abstractions-server /abstractions-server
