FROM rust:alpine3.20 AS build
LABEL authors="filipponik"

WORKDIR app

RUN apk add \
    musl-dev \
    openssl \
    pkgconfig \
    libressl-dev \
    upx \
    curl \
    unzip

COPY src/ ./src
COPY Cargo.lock Cargo.toml ./

RUN cargo build --release && upx --best --lzma target/release/telegraph

FROM zenika/alpine-chrome:124
WORKDIR app
COPY --from=build /app/target/release/telegraph .
