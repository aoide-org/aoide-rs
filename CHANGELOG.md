# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

- Log executable path and version on startup

### Removed

## [0.1.1] - 2019-08-31

### Added

- New sort field `musicKey` (Circle of fifth / Open Key)

### Changed

- Fixed OpenAPI spec

## [0.1.0] - 2019-08-30

**This release breaks backward compatibility with existing versions! Existing databases need to be rebuilt.**

### Added

- Support track/disc number and total for filtering and sorting
- Evaluate environment variables if corresponding command-line arguments are missing: LOG_LEVEL, LISTEN_ADDR, DATABASE_URL
- New multi-stage Docker build image

### Changed

- Re-engineered the JSON schema for improving space efficiency
- Improved semantic validation and reporting
- Separate routes for calculating album and tag statistics
- Allow to sort tag count results by various fields

### Removed

- No more language specific titles

## [0.0.9] - 2019-06-20

### Changed

- Support both prefix and exact match URI predicates for relocating tracks

## [0.0.8] - 2019-06-16

### Added

- Added /tracks/relocate (POST) to relocate track sources by their URI prefix

### Changed

- Delay advertising of endpoint address until server is listening

## [0.0.7] - 2019-06-14

### Changed

- Fix clux-muslrust build
- Update version numbers

## [0.0.6] - 2019-06-14

### Added

- Added missing documentation for /shutdown request

### Changed

- Purge tracks by either exact source URI or by prefix

## [0.0.5] - 2019-06-12

### Added

- Added *AutoCrop* position marker type
- Added /about (GET) for health checks and monitoring
- Added /shutdown (POST) for graceful shutdown
- Added /tracks/purge (POST) to purge track sources and tracks by URI
- Print socket address to *stdout* for connecting clients through an ephemeral port

### Changed

- Renamed *LoadCue* position marker to just *Cue*

## [0.0.4] - 2019-05-11

### Added

- Web framework [Warp](https://github.com/seanmonstar/warp)

### Changed

- Embedded all static resources in executable
- Changed default port of Docker image from 8080 to 7878
- Fixed IPv6 wildcard address in Docker entrypoint script
- Fixed inconsistent version numbers across projects and documents

### Removed

- Web framework [Actix Web](https://github.com/actix/actix-web)

## [0.0.3] - 2019-04-24

### Added

- Added missing database indexes to improve serach performance
- Added support for marking custom positions (points/sections) in a track
- Added filter for searching tracks by the labels of their position markers
- Added beat markers
- Added key markers

### Changed

- Fixed GreaterOrEqual numeric filtering
- Track: Renamed "markers" as "positionMarkers"

### Removed

- Removed support for assigning tags to track/position markers that
  were not searchable. The single label that can be assigned to a position
  marker should be sufficient and is supported for searching.

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

[Unreleased]: https://gitlab.com/uklotzde/aoide-rs/compare/v0.1.1...development
[0.1.1]: https://gitlab.com/uklotzde/aoide-rs/releases/v0.1.1
[0.1.0]: https://gitlab.com/uklotzde/aoide-rs/releases/v0.1.0
[0.0.9]: https://gitlab.com/uklotzde/aoide-rs/releases/v0.0.9
[0.0.8]: https://gitlab.com/uklotzde/aoide-rs/releases/v0.0.8
[0.0.7]: https://gitlab.com/uklotzde/aoide-rs/releases/v0.0.7
