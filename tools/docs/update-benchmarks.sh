#!/bin/sh
# ---------------------------------------------------------------------
# Gufo SNMP: Run benchmarks suit and update docs results
# ---------------------------------------------------------------------
# Copyright (C) 2024-25, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

PYTEST=pytest
PYTHON=python3

# Run benchmarks
for f in benchmarks/test_*.py; do
    # Extract parts from filename
    base=$(basename "$f" .py)  # e.g., test_v2c_getbulk
    proto=$(echo "$base" | cut -d_ -f2)  # v2c or v3
    outfile="docs/benchmarks/$proto/${base}.txt"
    echo "Running $f..."
    $PYTEST "$f" > "$outfile"
done

# Update charts
$PYTHON tools/docs/update-bench-charts.py
