FROM rust:alpine3.19 as build
LABEL authors="filipponik"

WORKDIR app

RUN apk add \
    musl-dev \
    openssl \
    pkgconfig \
    libressl-dev \
    upx

COPY src/ ./src
COPY Cargo.lock Cargo.toml ./

RUN cargo build --release \
    && upx --best --lzma target/release/telegraph
FROM alpine:3.19
WORKDIR app
COPY --from=build /app/target/release/telegraph .
