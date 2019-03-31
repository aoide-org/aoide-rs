# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

### Removed

## [0.0.2] - 2019-03-31

### Added

- Added `count` POST request to report tracks per album

### Changed

- Splitted tags into `plain` and `faceted` tags
- The `label` of a faceted tag is now optional and may be missing
- Revised tag reporting for tracks by using `count` POST requests
- Fixed various documentation issues
- Changed the database schema. Existing SQLite databases need to be rebuilt from scratch!

### Removed

- Removed obsolete test executable

## [0.0.1] - 2019-03-24

### Added

- Initial public release

[Unreleased]: https://gitlab.com/uklotzde/aoide-rs/compare/v0.0.2...development
[0.0.2]: https://gitlab.com/uklotzde/aoide-rs/releases/v0.0.2
[0.0.1]: https://gitlab.com/uklotzde/aoide-rs/releases/v0.0.1
