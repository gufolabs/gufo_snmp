#!/bin/sh
# ---------------------------------------------------------------------
# Gufo SNMP: Docker entrypoint for benchmarks
# ---------------------------------------------------------------------
# Copyright (C) 2024-25, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

set -e

export PATH=/usr/local/cargo/bin:$PATH
export RUSTUP_HOME=/usr/local/rustup
export CARGO_HOME=/usr/local/cargo
export PYTHONPATH=src

echo "Installing system dependencies..."
apt-get clean
apt-get update
apt-get install -y --no-install-recommends\
    ca-certificates\
    curl\
    gcc\
    libc6-dev\
    snmpd\
    libsnmp-dev
echo "Updating pip..."
pip install --upgrade pip
echo "Installing requirements..."
pip install -r .requirements/test.txt -r .requirements/bench.txt
echo "Setting up rust..."
./tools/build/setup-rust.sh
rustup component add llvm-tools-preview
echo "Benchmarking"
./tools/docs/update-benchmarks.sh
