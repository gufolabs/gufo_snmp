#!/bin/sh
# ------------------------------------------------------------------------
# Gufo Labs: Install and setup rust
# ------------------------------------------------------------------------
# Copyright (C) 2023-24, Gufo Labs
# ------------------------------------------------------------------------

set -x
set -e

if [ -z "${RUST_ARCH}" ]; then
    echo "RUST_ARCH is not set"
    exit 2
fi

RUST_VERSION=${RUST_VERSION:-1.80.0}

# @todo: Allow override
export RUSTUP_HOME=${RUSTUP_HOME:-/usr/local/rustup}
export CARGO_HOME=${CARGO_HOME:-/usr/local/cargo}
export PATH=${CARGO_HOME}/bin:${PATH}

echo "Install Rust ${RUST_ARCH}"
echo "PATH        = ${PATH}"
echo "RUSTUP_HOME = ${RUSTUP_HOME}"
echo "CARGO_HOME  = ${CARGO_HOME}"

# Install rust
# rustup-init tries to check /proc/self/exe
# which is not accessible during Docker build
# on aarch64, so we will patch it
curl -s --tlsv1.2 https://sh.rustup.rs \
    | sed 's#/proc/self/exe#/bin/sh#g' \
    | sh -s -- \
        -y --no-modify-path --profile minimal \
        --default-toolchain ${RUST_VERSION} \
        --default-host ${RUST_ARCH}
# Check
cargo --version
rustc --version
