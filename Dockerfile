FROM ekidd/rust-musl-builder:latest as base-toolchain


ADD --chown=rust:rust Cargo.toml Cargo.toml
RUN mkdir src && touch src/lib.rs
RUN cargo build --release
RUN rm src/lib.rs

ADD --chown=rust:rust . .
RUN cargo build --release --bin robbot