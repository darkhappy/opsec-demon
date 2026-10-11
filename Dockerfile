FROM rust:1.99.0-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY assets ./assets
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime
RUN useradd --uid 10001 --create-home demon
COPY --from=build /app/target/release/opsec-demon /usr/local/bin/opsec-demon
USER 10001:10001
EXPOSE 9090
ENTRYPOINT ["opsec-demon"]
CMD ["--metrics-server", "0.0.0.0:9090"]
