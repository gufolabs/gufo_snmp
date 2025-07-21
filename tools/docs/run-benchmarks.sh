#!/bin/sh
# ---------------------------------------------------------------------
# Gufo SNMP: Run benchmarks in docker
# ---------------------------------------------------------------------
# Copyright (C) 2024-25, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

docker run --rm\
    -w /workspaces/gufo_snmp\
    -v $PWD:/workspaces/gufo_snmp\
    python:3.13-slim-bullseye\
    /workspaces/gufo_snmp/tools/docs/entrypoint.sh
