---
hide:
    - navigation
---
# Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

To see unreleased changes, please see the [CHANGELOG on the main branch guide](https://github.com/gufolabs/gufo_snmp/blob/main/CHANGELOG.md).

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