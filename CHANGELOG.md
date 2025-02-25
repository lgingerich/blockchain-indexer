# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-02-20

### Added
- Initial (non-production) release of the blockchain indexer
- Support for indexing Ethereum and ZKsync blockchain data
- BigQuery integration for data storage

[0.1.0]: https://github.com/lgingerich/blockchain-indexer/releases/tag/v0.1.0

## [0.1.1] - 2025-02-20

### Changed
- Skip oversized traces as detected by error code `-32008`
- Fix ZKsync receipt parsing to retry on missing fields rather than failing
- Update retry config for longer delays, more attempts, and full jitter

[0.1.1]: https://github.com/lgingerich/blockchain-indexer/releases/tag/v0.1.1
