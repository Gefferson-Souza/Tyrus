# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/Gefferson-Souza/Tyrus/releases/tag/tyrus_orchestrator-v0.1.0) - 2026-02-14

### Added

- Improve generated Rust code quality by removing unnecessary `mut` keywords, update documentation with project name and grammar, and add a build step to CI.
- Add support for unary expressions, convert member properties to snake_case, inline AppError, fix a todo placeholder, and adjust DI order.
- Generate `serde(rename_all = "camelCase")` attributes for Rust types, enhance `AppError` with `Debug` and `Display` traits, and remove `test
- Remove all core compiler crates and related infrastructure, updating tests and CI configuration.

### Other

- rename `OxidizerError` to `TyrusError` and perform general project cleanup by removing temporary files and directories.
- reformat code, reorder imports, and adjust crate attributes for consistency.
