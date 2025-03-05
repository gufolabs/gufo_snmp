#!/bin/sh
# ------------------------------------------------------------------------
# Build binary wheels for Linux
# ------------------------------------------------------------------------
# Copyright (C) 2022-25, Gufo Labs
# ------------------------------------------------------------------------

set -e

build() {
    local image_arch="$1"
    local image="$2"

    case "${image_arch}" in
        linux/amd64) rust_arch="x86_64-unknown-linux-gnu";;
        linux/aarch64) rust_arch="aarch64-unknown-linux-gnu";;
        *)
            echo "Invalid arch: ${image_arch}"
            exit 1
            ;;
    esac
    docker run --rm\
        -e RUST_ARCH=${rust_arch}\
        -v $PWD:/workdir\
        -w /workdir\
        --user 0\
        --platform ${image_arch}\
        quay.io/pypa/${image}:latest\
        ./tools/build/build-many.sh 3.9 3.10 3.11 3.12 3.13
}

build linux/amd64 manylinux2014_x86_64
build linux/amd64 manylinux_2_28_x86_64
build linux/aarch64 manylinux_2_28_aarch64
build linux/amd64 musllinux_1_2_x86_64
build linux/aarch64 musllinux_1_2_aarch64