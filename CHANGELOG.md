# Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

To see unreleased changes, please see the [CHANGELOG on the main branch guide](https://github.com/gufolabs/gufo_snmp/blob/main/CHANGELOG.md).

## [Unreleased]

### Infrastructure

* devcontainer: Move `settings` to the `customisations.vscode.settings`
* docs: mkdocs-material 9.2.3
* Rust 1.72.0

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