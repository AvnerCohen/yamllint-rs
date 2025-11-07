# Changelog

All notable changes to this project will be documented in this file.

## [0.1.4] - 2025-01-XX

### Fixed
- Exit code now correctly returns 1 when linting issues are found (previously always returned 0)
- Progress tracking shows accurate file counts during directory processing

### Added
- `--no-progress` flag to disable progress updates for quieter output
- Progress updates every 1000 files processed (shows ~10-30 updates for large directories)

## [0.1.3] - 2025-01-XX

### Added
- Top-level `ignore` and `ignore-from-file` configuration options for file-level exclusion
- Support for gitwildmatch-style patterns in ignore configuration (matching Python yamllint behavior)
- Files matching ignore patterns are now completely skipped from processing (not just rule-level ignored)

## [0.1.2] - 2025-11-06

### Added
- Multi-platform Docker build and push command (`docker-multi-push`) supporting both linux/amd64 and linux/arm64 architectures

## [0.1.1] - 2024-12-19

### Changed
- Made the `-r` (recursive) flag optional. Directories are now automatically detected and processed recursively without requiring the `-r` flag. The flag still works if explicitly provided.

## [0.0.1] - Initial Release

### Overview
yamllint-rs is a Rust implementation of yamllint, a linter for YAML files. This project aims to provide a fast, native alternative to the Python-based yamllint while maintaining compatibility with yamllint's rule set and configuration format.

### Features
- Support for yamllint-compatible rules
- Configuration file support (yamllint-compatible format)
- Recursive directory processing
- Auto-fix capability for fixable issues
- Colored and standard output formats
- Gitignore support for file discovery

