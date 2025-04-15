#!/bin/sh
# ---------------------------------------------------------------------
# Gufo HTTP: Print components bin directory
# ---------------------------------------------------------------------

set +e

RUSTUP=`which rustup`
RUSTUP_HOME=`$RUSTUP show home`
DEFAULT_HOST=`$RUSTUP show | grep "Default host" | sed 's/Default host: //'`
TOOLCHAIN=`$RUSTUP show active-toolchain | cut -d" " -f1`

BIN="${RUSTUP_HOME}/toolchains/${TOOLCHAIN}/lib/rustlib/${DEFAULT_HOST}/bin"
echo $BIN