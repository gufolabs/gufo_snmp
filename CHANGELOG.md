---
hide:
    - navigation
---
# Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

To see unreleased changes, please see the [CHANGELOG on the main branch guide](https://github.com/gufolabs/gufo_snmp/blob/main/CHANGELOG.md).

## 0.9.0 - 2025-09-23

### Added

* MacOS ARM64 binary wheels.

### Security

* Install security updates during devcontainer build.
* Use python:3.13-slim-trixie as base for devcontainer.

### Infrastructure

* Rust 1.90.0
* PyO3 0.26
* Codecov integration.

## 0.8.4 - 2025-08-29

### Infrastructure

* Move dependencies to pyproject.toml

## 0.8.3 - 2025-07-21

### Changed

* SNMPv3: Always set REPORT flag for improved compatibility with wierd implementations.

## 0.8.2 - 2025-07-21

### Fixed

* Bug-per-bug compatibility with Net-SNMP: Fill encryption padding with padding length.

## 0.8.1 - 2025-07-18

### Fixed

* Fixed DES privacy padding in SNMP v3.

## 0.8.0 - 2025-04-15

### Added

* Benchmarks.
* Enable PGO on productive builds.

### Changed

* Change OID internal format, reduce memory allocations.
* Optimized internal ObjectId processing.

### Infrastructure

* Ruff 0.11.2
* Rust 1.86.0
* PyO3 0.24
* mkdocs-material 9.6.11

## 0.7.1 - 2025-03-23

### Added

* LTO is switched on.

### Fixed

* #17: Fix premature end of getbulk.

## 0.7.0 - 2025-03-05

### Added

* Musl and ARM64 binary wheels

### Changed

* Massive internals refactoring caused by moving to a new PyO3 API.

### Infrastructure

* Rust 1.85.0
* Rust edition 2024
* PyO3 0.23

## 0.6.0 - 2024-11-06

### Fixed

* Fix [#16][#16]: SNMPv3 requests not works with Mikrotik RouterOS devices.
* Fix [#15][#15]: sync client not releasing Python GIL on blocking operations.

### Added

* Python 3.13 binary wheels.

### Changed

* Dropping support of Python 3.8
* Massive refactoring of internals to support PyO3 0.22 and to remove duplicated code.
* Use reusable buffers pool insead of allocating buffer along with each socket.

### Infrastructure

* Rust 1.82.0
* mypy 1.13.0
* Ruff 0.7.2
* pytest 8.3.3
* coverage 7.6.4
* mkdocs-material 9.5.44
* Black formatter replaced by Ruff.

## 0.5.2 - 2024-07-30

### Added
* `SnmpAuthError` is exposed to `gufo.snmp` root module.

### Infrastructure

* Rust 1.80.0

## 0.5.1 - 2024-02-28

### Added
* Pack rust benchmarks into sdist.

### Infrastructure
* Rust 1.76.0
* Remove `setup.py` file.

## 0.5.0 - 2024-02-02

### Added
* Synchronous SNMP client.
* `Snmpd`: `verbose` option.

### Changed
* Improved password to master key translation performance: ~35%.
* Optimized SNMPv3 message signing.
* Examples split to sync and async versions.

### Fixed
* Default key type for auth keys set to Password.

## 0.4.0 - 2024-01-29

### Added
* SNMP v3 support.
* Snmpd.engine_id property.
* Tests for all SNMP versions.

## 0.3.0 - 2024-01-10

### Added

* Python 3.12 builds
* Optimized performance (measured L.A. reduction in 1.5 times)

### Changed

* docs: Fancy front page
* Build on GLibc 2.28 rather than on 2.24

### Infrastructure

* devcontainer: Move `settings` to the `customisations.vscode.settings`
* docs: mkdocs-material 9.2.3
* Rust 1.75.0
* PyO3 0.20
* socket2 0.5
* devcontainer: Use Python 3.12

## 0.2.0 - 2023-02-27

### Added

* docs: Benchmarks
* SnmpSession `policer` and `limit_rps` parameters
  for query rate limiting.

### Changed

* Improve BER decoder performance:

  * BER Header: ~18%
  * Typical Get Response: ~25%
  * Object Identifier: ~46%

### Infrastructure

* Iai benchmarks
* `fmt-iai.py` tool

## 0.1.1 - 2023-02-17

### Fixed

* Fix SnmpSession allow_bulk handling.
* Fix [#1][#1]: Getting FileNotFoundError exception if multiple instances
  of SnmpSession were previously used.

### Infrastructure

* Rust 1.67.1
* Ruff 0.0.246
* Criterion Benchmarks

## 0.1.0 - 2023-02-02

### Added

* Initial implementation

[#1]: https://github.com/gufolabs/gufo_snmp/issues/1
[#15]: https://github.com/gufolabs/gufo_snmp/issues/16
[#16]: https://github.com/gufolabs/gufo_snmp/issues/16