# Agent Notes -- gufo_snmp

## Project identity
- Name: gufo_snmp (PyPI package)
- Version: 0.12.0
- License: BSD-3-Clause
- Part of Gufo Stack — the collaborative effort led by Gufo Labs to extract reusable tech from NOC into separate packages.
- GitHub: https://github.com/gufolabs/gufo_snmp/

## What it does
Accelerated Python SNMP client library supporting async and sync modes. Rust core for BER parsing and socket I/O, exposed via PyO3 bindings as a native .so module (`gufo.snmp._fast`).

Supports SNMP v1, v2c, v3 (USM with HMAC-MD5/SHA-1/SHA-2-224/256/384/512 auth and DES/AES-128/192/256 privacy; 192/256 via Blumenthal or Cisco/Reeder key expansion). Query rate limiting. Zero-copy BER parsing. 100% Python typing support. CLI tool (`gufo-snmp`).

## Architecture overview
The project is a bilingual Rust+Python library:

```
src/  (Rust core, compiles to the native gufo.snmp._fast module)
  auth/          -- SNMPv3 USM auth: digest, noauth, blumenthal, cisco (key expansion)
  snmp/          -- PDU, get, getbulk, getmany, report, getresponse, value
    op/          -- Operation traits/impls: get, getbulk, getiter, getnext, getmany, refresh
    msg/         -- Message formats: v1, v2c, v3 (usm, scoped, data)
  ber/           -- X.690 BER encoder/decoder (minimalist SNMP subset)
  socket/        -- v1/v2c/v3 client sockets + SnmpSocket (Rust)
  privacy/       -- Encryption: des, aes (cfb-mode), nopriv
  buf/           -- Buffer primitives + pool
  lib.rs / util.rs / reqid.rs    -- crate entry, utilities, request IDs
src/gufo/snmp/   (Python API)
  __init__.py    -- Re-exports User, key types, SnmpVersion, errors, ValueType
  aio/           -- Async  SnmpSession + GetNextIter / GetBulkIter
  sync/          -- Sync  SnmpSession + getnext.py / getbulk.py iterators
  user.py        -- User + key types (Md5/Sha1/Sha*/Aes*/Des, KeyExpansion)
  policer.py     -- Rate limiter: BasePolicer, RPSPolicer
  utils.py, cli.py, snmpd.py, protocol.py, typing.py, version.py
tests/           -- async, sync, cli, user, policer, fast, project, docs, ci, utils
```

## Key file locations
| Concern | File |
| --- | --- |
| Async SnmpSession | `src/gufo/snmp/aio/client.py` |
| Sync SnmpSession | `src/gufo/snmp/sync/client.py` |
| Rust SNMP ops (get, getbulk) | `src/snmp/op/get.rs`, `src/snmp/op/getbulk.rs` |
| BER types | `src/ber/*.rs` (mod, objectid, sequence, int, etc.) |
| Socket impls | `src/socket/v1.rs`, `v2c.rs`, `v3.rs` |
| CLI tool | `src/gufo/snmp/cli.py` |
| Python types (User/Keys) | `src/gufo/snmp/user.py` |
| Config/lints | `Cargo.toml`, `pyproject.toml` |

## Cargo.toml specifics
- Edition: 2024
- Dependency: `pyo3 0.28` with `extension-module` feature
- Encryption deps: aes, cbc, cfb-mode, cipher, des, digest, md-5, sha1
- BER parsing: nom (parser combinator library)
- Release profile: LTO=full fat, strip=debuginfo
- Benches: iai_decode, iai_encode, iai_buf, iai_auth

## pyproject.toml specifics
- Python: >=3.9 (supports 3.9 through 3.14)
- Build deps: setuptools>=61.2, wheel, setuptools-rust>=1.9
- Lints: ruff (line_length=79, py39 target), mypy strict mode
- Test deps: pytest 8.4.2, pytest-cov 7, pytest-benchmark
- Docs mkdocs material + mkdocstrings[python]

## Async architecture (core hot paths)
Both sync and async SnmpSession share identical `__init__` logic. The key difference is I/O:

**Async** (`aio/client.py`):
- `_send()` — calls sender() in-place (hot path), falls back to loop.add_writer on BlockingIOError
- `_recv()` — event-loop reader polling, wait_for timeout
- `refresh()` — SNMPv3 engine discovery/refresh over async I/O
- Iterators `GetNextIter` / `GetBulkIter` await per-item

**Sync** (`sync/client.py`):
- Direct call to `self._sock.get(oid)` with `BlockingIOError` -> `TimeoutError` mapping
- `timeout` as nanoseconds passed to Rust socket (vs float seconds in async)
- Same structure but synchronous throughout

## API design
- Single entry point: `SnmpSession` class, dual import paths (`gufo.snmp` and `gufo.snmp.sync`)
- Identical constructor signatures for both async/sync variants
- Auto-version detection when `user` is None -> v2c; `user` set -> v3
- SNMPv3 deferred auth: engine_id discovery first session, then key localization via `set_keys()`
- Rate limiting built-in: `limit_rps` or manual `policer=` parameter
- Bulk fetch: `fetch()` delegates to `getbulk()` (SNMPv2+) or `getnext()` (v1)

## SNMPv3 specifics
- Auth algorithms: HMAC-MD5-96, HMAC-SHA-96 only (SHA2 family is future work)
- Privacy: DES, AES128 (AES256 is future work)
- Auth engines: `md-5` crate, `sha1` crate
- Private engines: `des`, `aes`+`cbc` for DES; `aes128` + `cfb-mode` for AES
- Engine ID discovery via REPORT message (automatic on first connection without engine_id)

## Testing approach
- pytest with coverage and benchmarking (`--benchmark-min-rounds=50`)
- SNMP daemon wrapper (`snmpd.Snmpd`) spins up local agent for integration tests
- Tests cover: async API, sync API, CLI, user/key types, rate limiter, native Rust FFI, docs, CI

## Build process (how to build)
```bash
pip install -e ".[build]"
# The Rust code is compiled via setuptools-rust automatically.
# For release builds: tools/build/build-many.sh or build-linux.sh
```

## Dev tooling — run-dev CLI

All lint and test commands **must** run inside the devcontainer:

```bash
./scripts/run-dev ruff check src/ tests/
./scripts/run-dev ruff format --check src/ tests/
./scripts/run-dev mypy src/
./scripts/run-dev pytest -v
./scripts/run-dev pytest --cov --cov-branch --cov-report=xml
```

Direct `pip install`, `pytest`, `ruff`, or `mypy` calls outside the container
**will fail** (missing dependencies, wrong Python version, no venv). Always
use `run-dev` as the wrapper.

## Future items (from README roadmap)
- SHA2 family of hashes
- AES256 encryption
- SNMP Trap and Inform collector
- Integration with NOC's Compiled MIB infrastructure

## CI/CD
- GitHub Actions workflows in `.github/workflows/`:
  - `tests.yml` — tests on push/PR
  - `package.yml` — PyPI package builds
  - `security.yml` / `codeql.yml` — security scanning
- Codecov integration for coverage

## Documentation
- MkDocs Material, at https://docs.gufolabs.com/gufo_snmp/
- Man pages: `docs/man/gufo-snmp.md`
- Dev types doc: `docs/dev/types.md`

## Conventions and rules
- Line length: 79 (ruff config)
- Python target: 3.9+
- Google-style docstrings (ruff lints pydocstyle.google)
- Both mypy strict mode and ruff type linting enabled
- Double-quote string convention (ruff flake8-quotes.docstring-quotes = "double")
- All modules use Gufo Labs copyright headers
- Rust edition: 2024

## Things to watch for (potential issues)
1. **Dual SnmpSession classes** — async and sync variants use identical constructor signatures but different I/O; they live in separate packages. Be careful not to mix imports.
2. **SNMPv3 deferred authentication** — if engine_id is None on init, a REPORT request runs automatically before first data operation. This means the first call to `get()`/`get_many()` etc. sends TWO packets (one for discovery, one for actual data). The `self._deferred_user` pattern handles this.
3. **Sync timeout** uses nanoseconds (`timeout_ns`) on Rust side while async uses float seconds — ensure consistency when modifying timeout logic.
4. **PDU tag constants in `src/snmp/mod.rs`** are plain const u8; SET_REQUEST, TRAP, and other PDU types are commented out (not implemented).
