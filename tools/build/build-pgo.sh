#!/bin/sh
# ------------------------------------------------------------------------
# Build version which collects PGO data
# ------------------------------------------------------------------------
# Copyright (C) 2022-25, Gufo Labs
# ------------------------------------------------------------------------

set -e

PGO_DATA_DIR=$1
if [ "$PGO_DATA_DIR" = "" ]; then
    echo "PGO data dir must be set"
    exit 1
fi

# Collect PGO
echo "Building profiling version"
rm -rf target/
RUSTFLAGS="-Cprofile-generate=$PGO_DATA_DIR -Cllvm-args=-pgo-warn-missing-function" python3 -m pip install --editable .
echo "Collecting PGO data"
PYTHONPATH=src/:$PYTHONPATH python3 ./tools/build/pgo-runner.py
rm -f src/gufo/snmp/*.so
echo "Merging profdata"
$(./tools/build/get-rustup-bin.sh)/llvm-profdata merge -o $PGO_DATA_DIR/merged.profdata $PGO_DATA_DIR
echo "PGO profile is written into $PGO_DATA_DIR/merged.profdata"
