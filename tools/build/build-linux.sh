#!/bin/sh
# ------------------------------------------------------------------------
# Build binary wheels for Linux
# ------------------------------------------------------------------------
# Copyright (C) 2022-25, Gufo Labs
# ------------------------------------------------------------------------

set -e

# Define all valid builds
VALID_IMAGES="
manylinux2014_x86_64
manylinux_2_28_x86_64
manylinux_2_28_aarch64
musllinux_1_2_x86_64
musllinux_1_2_aarch64
"

# Check if a given combination is valid
is_valid_image() {
    echo "$VALID_IMAGES" | grep -qE "^$1$"
}

# Build for platform
build() {
    local image="$1"

    case "$image" in
        *_x86_64) 
            local image_arch="linux/amd64"
            local rust_arch="x86_64-unknown-linux-gnu"
            ;;
        *_aarch64)
            local image_arch="linux/aarch64"
            local rust_arch="aarch64-unknown-linux-gnu"
            ;;
        *) return 1 ;; # Unknown platform
    esac
    docker run --rm\
        -e RUST_ARCH=${rust_arch}\
        -v $PWD:/workdir\
        -w /workdir\
        --user 0\
        --platform ${image_arch}\
        quay.io/pypa/${image}:latest\
        ./tools/build/build-many.sh 3.9 3.10 3.11 3.12 3.13 3.14
}

if [ "$#" -eq 0 ]; then
    # No arguments: Run all builds
    echo "$VALID_IMAGES" | while read -r image; do
        build "$image"
    done
elif [ "$#" -eq 1 ]; then
    # Two arguments: Check validity and run if valid
    if is_valid_image "$1"; then
        build "$1"
    else
        echo "Error: Invalid image '$1'"
        exit 1
    fi
else
    echo "Usage: $0 [image]"
    echo "Where [image] is one of: $VALID_IMAGES"
    exit 1
fi
