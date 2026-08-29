FROM docker.io/library/debian:bookworm-slim

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    ELAN_HOME=/usr/local/elan \
    PATH=/usr/local/cargo/bin:/usr/local/elan/bin:$PATH

RUN apt-get update \
    && apt-get install --no-install-recommends --yes \
        build-essential ca-certificates curl git libgmp10 \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --no-modify-path --profile minimal --default-toolchain 1.97.1 \
    && rustup component add rustfmt clippy

RUN curl https://elan.lean-lang.org/elan-init.sh -sSf \
    | sh -s -- -y --no-modify-path --default-toolchain none \
    && elan toolchain install leanprover/lean4:v4.32.1 \
    && elan default leanprover/lean4:v4.32.1

RUN cargo install cargo-deny --version 0.20.2 --locked

WORKDIR /work

