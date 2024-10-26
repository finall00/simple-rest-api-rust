FROM rust:1.72 as builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

ENV CARGO_HOME=/usr/local/cargo

# Instale dependências e compile o projeto sem interações
RUN cargo install --locked cargo-chef && \
    cargo chef prepare && \
    cargo chef cook --release

RUN cargo fetch

COPY . . 

RUN cargo build --release

FROM debian:bullseye-slim

WORKDIR /usr/local/bin

COPY --from=builder /app/target/release/rust-crud-api .



CMD ["./rust-crud-api"]
