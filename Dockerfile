FROM rust:1.99-alpine@sha256:3ea887915f261f273d6a24efdfdb92665dee12199923e57870453644fe4ae5c9 AS build
WORKDIR /build
RUN apk add --no-cache musl-dev
RUN rustup target add x86_64-unknown-linux-musl
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --no-default-features --target x86_64-unknown-linux-musl

FROM alpine:3.24@sha256:71e9b7bae160ba167fa17414401b94cbcdaf3c32feeab9298bd794a2a0614c35
RUN apk add --no-cache ca-certificates && \
    adduser -D whois42d-ng
USER whois42d-ng
COPY --from=build /build/target/x86_64-unknown-linux-musl/release/whois42d-ng /whois42d-ng
CMD ["/whois42d-ng", "--registry", "/registry", "--address", "::", "--port", "4343"]
