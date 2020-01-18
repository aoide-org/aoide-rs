# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

### Removed

## [0.6.0] - 2020-01-18

**This release breaks backward compatibility with old versions! Existing databases need to be rebuilt.**

### Added

- Added support for playlist entities
- Added optional comment for track/collection relationships

### Changed

- Modified serialization format of track/collection relationships
- Renamed artwork background color field

### Removed

## [0.5.1] - 2020-01-05

### Changed

- Fixed serialization mapping of collection properties. The *name* and *description*
  of existing collections must be exchanged manually.

## [0.5.0] - 2019-12-19

**This release breaks backward compatibility with old versions! Existing databases need to be rebuilt.**

### Added

- Added auxiliary table for locating tracks by URI in collections
- Added constraints to prevent tracks with ambiguous/duplicate URIs
  in a single collection

## [0.4.0] - 2019-12-08

**This release breaks backward compatibility with old versions! Existing databases need to be rebuilt.**

### Changed

- Aggregated position/beat/key markers into a separate abstraction level with `state`
- Renamed various marker properties in serialization format

### Removed

- Removed `source` from position/beat/key markers
- Removed `state` from individual position/beat/key markers

## [0.3.3] - 2019-11-29

### Changed

- Improved and fixed validation

## [0.3.2] - 2019-11-08

### Changed

- Updated OpenAPI spec

## [0.3.1] - 2019-09-22

**This release introduces a backward-incompatible API change. The parameter *mediaUri* has been replaced by *mediaUriDecoded*.**

### Added

- New string field *mediaUri* for filtering canonical, percent-encoded URIs
- New string field *mediaUri* and *mediaUriDecoded* for ordering search results

### Changed

- Replaced string field *mediaUri* with *mediaUriDecoded*

## [0.3.0] - 2019-09-13

**This release breaks backward compatibility with old versions! Existing databases need to be rebuilt.**

### Added

- Added optional artwork to media sources

### Changed

- Changed the representation of media sources to allow adding optional metadata like artwork

### Removed

- Removed transparency (alpha channel) from RGB color codes

## [0.2.0] - 2019-09-04

**This release breaks backward compatibility with old versions! Existing databases need to be rebuilt.**

### Changed

- Log executable path and version on startup
- Support both simple release dates (YYYYMMDD) and full time stamps
- Store complete release dates (YYYYMMDD) instead of only the year for filtering, sorting, and grouping of album tracks

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

[Unreleased]: https://gitlab.com/uklotzde/aoide-rs/compare/v0.6.0...development
[0.6.0]: https://gitlab.com/uklotzde/aoide-rs/releases/v0.6.0
[0.5.1]: https://gitlab.com/uklotzde/aoide-rs/releases/v0.5.1
[0.5.0]: https://gitlab.com/uklotzde/aoide-rs/releases/v0.5.0
[0.4.0]: https://gitlab.com/uklotzde/aoide-rs/releases/v0.4.0
[0.3.3]: https://gitlab.com/uklotzde/aoide-rs/releases/v0.3.2
