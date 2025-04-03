FROM alpine as cargo-binstall

RUN apk add --no-cache curl

COPY cargo-binstall.sh /
RUN mkdir /binstall && \
    /cargo-binstall.sh -y --locked --install-path /binstall cargo-llvm-cov cargo-nextest flamegraph


# cargo-criterion does not provide pre-built binaries
FROM rust:slim as cargo-criterion
RUN mkdir /out && \
    cargo install --root /out cargo-criterion


FROM mcr.microsoft.com/devcontainers/base:alpine

ARG msrv

COPY --from=cargo-binstall /binstall/* /usr/local/bin/
COPY --from=cargo-criterion /out/* /usr/local/bin/

RUN apk upgrade && \
    apk add --no-cache just rustup foot-extra-terminfo wezterm-extra-terminfo

USER vscode

RUN rustup-init -y --default-toolchain $msrv --component llvm-tools-preview --target thumbv7em-none-eabihf

RUN $HOME/.cargo/bin/rustup toolchain install nightly --profile minimal --component rustfmt
