FROM rust:1.98-alpine@sha256:c913be57168b9240b86f373f94060152a2e09ea16a72e0801a02ee3a262ca446 AS build
WORKDIR /build
RUN apk add --no-cache musl-dev
RUN rustup target add x86_64-unknown-linux-musl
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --no-default-features --target x86_64-unknown-linux-musl

FROM alpine:3.24@sha256:294b683cb724975bec92580e1e685676bd4b50bda910ddb8c51d4cabeaec77e6
RUN apk add --no-cache ca-certificates && \
    adduser -D whois42d-ng
USER whois42d-ng
COPY --from=build /build/target/x86_64-unknown-linux-musl/release/whois42d-ng /whois42d-ng
CMD ["/whois42d-ng", "--registry", "/registry", "--address", "::", "--port", "4343"]
