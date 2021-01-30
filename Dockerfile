FROM bitnami/minideb:latest as base-toolchain

ENV DEBIAN_FRONTEND="noninteractive"

RUN apt-get update && \
    apt-get install ca-certificates libssl-dev pkg-config -y && \
    update-ca-certificates

FROM base-toolchain as toolchain

RUN install_packages curl build-essential

RUN useradd -m robbot -d /robbot
RUN usermod -p '!!' root # Disable all passwords for root
USER robbot
ENV USER=robbot
ENV PATH=/robbot/.cargo/bin:$PATH
WORKDIR /robbot

# Ensure that we are using the latest stable version of rustup and the
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal --default-toolchain "stable"

# Fetch all the crate source files

FROM toolchain as bare-sources

WORKDIR /robbot

# Build the sources (without the actual code).  This will mean we get
# a cargo cache as long as we don't change our dependencies.
RUN mkdir src
ADD --chown=robbot Cargo.toml /robbot/Cargo.toml
RUN touch /robbot/src/lib.rs
RUN cargo build --release

# Compiler and sources
FROM bare-sources as sources

RUN rm src/*.rs
ADD --chown=robbot src /robbot/src
RUN cargo build --release

# Just the application now
FROM base-toolchain as runenv
WORKDIR /app
COPY --from=sources /robbot/target/release/robbot /app/robbot

CMD ["/app/robbot"]