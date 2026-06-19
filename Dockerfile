FROM rust:1.85-bookworm AS build
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
USER 65534:65534
COPY --from=build /build/target/release/whois42d-ng /whois42d-ng
CMD ["/whois42d-ng", "--registry", "/registry", "--port", "4343"]
