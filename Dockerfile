FROM rust:1.98-alpine@sha256:1716b3aa042d735f4566d14dc54e8037de9d69556e2d5dd58131d93a613d173d AS build
WORKDIR /build
RUN apk add --no-cache musl-dev
RUN rustup target add x86_64-unknown-linux-musl
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --no-default-features --target x86_64-unknown-linux-musl

FROM alpine:3.24@sha256:e7c4abb69531cb09e2a2bbb56fad3367ab694865c49df898c1c683185cc4376c
RUN apk add --no-cache ca-certificates && \
    adduser -D whois42d-ng
USER whois42d-ng
COPY --from=build /build/target/x86_64-unknown-linux-musl/release/whois42d-ng /whois42d-ng
CMD ["/whois42d-ng", "--registry", "/registry", "--address", "::", "--port", "4343"]
