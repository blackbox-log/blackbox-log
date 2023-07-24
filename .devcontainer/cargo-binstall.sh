#!/bin/sh

# Adapted from https://github.com/cargo-bins/cargo-binstall/blob/129331410058e5d66b3955201c53e76ff8b61386/install-from-binstall-release.sh

set -euxo pipefail

cd "$(mktemp -d)"

base_url="https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-"

os="$(uname -s)"
if [ "$os" = "Darwin" ]; then
    url="${base_url}universal-apple-darwin.zip"
    curl -LO --proto '=https' --tlsv1.2 -sSf "$url"
    unzip cargo-binstall-universal-apple-darwin.zip
elif [ "$os" = "Linux" ]; then
    machine="$(uname -m)"
    target="${machine}-unknown-linux-musl"
    if [ "$machine" = "armv7" ]; then
        target="${target}eabihf"
    fi

    url="${base_url}${target}.tgz"
    curl -L --proto '=https' --tlsv1.2 -sSf "$url" | tar -xvzf -
else
    echo "Unupporteed OS ${os}"
    exit 1
fi

./cargo-binstall $@
