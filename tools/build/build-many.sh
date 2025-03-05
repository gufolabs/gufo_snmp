#!/bin/sh
# ------------------------------------------------------------------------
# Build wheel in the quay.io/pypa/manylinux2014_x86_64:latest
# Usage:
# ./tools/build/build-many 3.9 3.10 3.11 3.11 3.12 3.13
# expects RUST_VERSION and RUST_ARCH
# ------------------------------------------------------------------------
# Copyright (C) 2022-25, Gufo Labs
# ------------------------------------------------------------------------

# set -x
set -e

empty_dir() {
    local path="$1"
    if [ -d "${path}" ]; then
        echo "Clearing ${path}..."
        rm -rf "${path}"/*
    else
        echo "Creating ${path}..."
        mkdir -p "${path}"
    fi
}

ensure_dir() {
    local path="$1"
    if [ ! -x "${path}" ]; then
        echo "Creating ${path}..."
        mkdir -p "${path}"
    fi
}

# Save base path
BASE_PATH=$PATH
# Rust settings
export RUSTUP_HOME=${RUSTUP_HOME:-/usr/local/rustup}
export CARGO_HOME=${CARGO_HOME:-/usr/local/cargo}
PATH=$CARGO_HOME/bin:$BASE_PATH

# Paths
BUILD="build"
DIST="dist"
TMP_WHEELHOUSE="/tmp/wheelhouse"
WHEELHOUSE="wheelhouse"
TARGET="target"

echo "##"
echo "## Installing rust"
echo "##"
empty_dir "${TARGET}"
./tools/build/setup-rust.sh

echo "##"
echo "## Installing snmpd"
echo "##"
./tools/build/setup-snmpd.sh

while [ $# -gt 0 ]
do
    echo "##"
    echo "## Building for Python $1"
    echo "##"
    # Convert version to ABI
    case "$1" in
        3.8) ABI=cp38-cp38 ;;
        3.9) ABI=cp39-cp39 ;;
        3.10) ABI=cp310-cp310 ;;
        3.11) ABI=cp311-cp311 ;;
        3.12) ABI=cp312-cp312 ;;
        3.13) ABI=cp313-cp313 ;;
        *)
            echo "Unknown Python version $1"
            exit 2
            ;;
    esac
    # Adjust path
    PATH=$CARGO_HOME/bin:/opt/python/$ABI/bin:$BASE_PATH
    # Check python version is supported in docker image
    export PYO3_PYTHON=/opt/python/$ABI/bin/python3
    if [ ! -f $PYO3_PYTHON ]; then
        echo "Python version $1 is not supported by image"
        exit 2
    fi
    # Check python
    PY_VER=$(python3 --version)
    echo "Python version: $PY_VER"
    echo "Upgrade pip..."
    pip install --upgrade pip
    echo "Setup build dependencies..."
    pip install -r ./.requirements/build.txt -r ./.requirements/test.txt
    echo "Building wheel..."
    empty_dir "${DIST}"
    empty_dir "${BUILD}"
    python3 -m build --wheel
    echo "Auditing wheel..."
    empty_dir "${TMP_WHEELHOUSE}"
    auditwheel repair --wheel-dir="${TMP_WHEELHOUSE}" "${DIST}"/*.whl
    echo "Installing wheel..."
    pip install "${TMP_WHEELHOUSE}"/*.whl
    echo "Testing..."
    pytest -vv
    echo "Saving..."
    ensure_dir "${WHEELHOUSE}"
    cp "${TMP_WHEELHOUSE}"/*.whl "${WHEELHOUSE}"/
    empty_dir "${DIST}"
    empty_dir "${TMP_WHEELHOUSE}"
    echo "... done"
    shift
done

echo "##"
echo "## Done"
echo "##"
ls -lh wheelhouse/
