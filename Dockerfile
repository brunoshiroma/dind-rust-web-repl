FROM rust:alpine AS builder

WORKDIR /rust

COPY src src
COPY Cargo.toml .
COPY Cargo.lock .
RUN cargo build --release


FROM docker:dind AS runner

WORKDIR /server

COPY --from=builder /rust/target/release/dind-rust-web-repl .