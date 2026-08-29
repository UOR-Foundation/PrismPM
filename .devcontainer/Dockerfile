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

RUN curl -sSfL https://github.com/leanprover/elan/releases/download/v4.2.3/elan-x86_64-unknown-linux-gnu.tar.gz -o elan.tar.gz \
    && echo "df0b2b3a439961ffcbb3985214365ffe40f49bc871df04dff268c7d8e21ca8b2  elan.tar.gz" | sha256sum -c - \
    && mkdir -p /tmp/elan-bin && tar -xzf elan.tar.gz -C /tmp/elan-bin \
    && /tmp/elan-bin/elan-init -y --no-modify-path --default-toolchain none \
    && rm -rf elan.tar.gz /tmp/elan-bin \
    && elan toolchain install leanprover/lean4:v4.32.1 \
    && elan default leanprover/lean4:v4.32.1

RUN curl -sSfL https://github.com/casey/just/releases/download/1.57.0/just-1.57.0-x86_64-unknown-linux-musl.tar.gz -o just.tar.gz \
    && echo "45b548094283cb9739af8f13273b8cddeee869f5b4ef2bb631b1f311cb566155  just.tar.gz" | sha256sum -c - \
    && tar -xzf just.tar.gz -C /usr/local/bin just \
    && rm just.tar.gz \
    && chmod +x /usr/local/bin/just

RUN curl -sSfL https://github.com/EmbarkStudios/cargo-deny/releases/download/0.20.2/cargo-deny-0.20.2-x86_64-unknown-linux-musl.tar.gz -o cargo-deny.tar.gz \
    && echo "9f12ed4c49936e09b48bf862b595cde2fe64fcbd9d74dfacac6131ca824c8d5f  cargo-deny.tar.gz" | sha256sum -c - \
    && tar -xzf cargo-deny.tar.gz --strip-components=1 -C /usr/local/bin cargo-deny-0.20.2-x86_64-unknown-linux-musl/cargo-deny \
    && rm cargo-deny.tar.gz \
    && chmod +x /usr/local/bin/cargo-deny

WORKDIR /work
