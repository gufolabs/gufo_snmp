FROM python:3.13-slim-bullseye AS dev
COPY .requirements tools/build/setup-rust.sh /tmp
ENV \
    PATH=/usr/local/cargo/bin:$PATH\
    RUSTUP_HOME=/usr/local/rustup\
    CARGO_HOME=/usr/local/cargo
RUN \
    set -x \
    && apt-get clean \
    && apt-get update \
    && apt-get install -y --no-install-recommends\
    git\
    ca-certificates\
    gcc\
    libc6-dev\
    curl\
    snmpd\
    && /tmp/setup-rust.sh \
    && rustup component add\
    rust-analysis\
    rust-src \
    rls\
    clippy\
    rustfmt\
    && pip install --upgrade pip\
    && pip install --upgrade build\
    && pip install\
    -r /tmp/build.txt\
    -r /tmp/docs.txt\
    -r /tmp/ipython.txt\
    -r /tmp/lint.txt\
    -r /tmp/test.txt