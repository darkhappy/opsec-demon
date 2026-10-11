VERSION 0.8
verify:
    FROM rust:1.93.1-bookworm
    WORKDIR /app
    RUN apt-get update && apt-get install -y python3
    COPY . .
    RUN sh scripts/ci.sh
    SAVE ARTIFACT target/release/opsec-demon AS LOCAL target/earthly/opsec-demon
