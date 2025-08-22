# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-08-22

### Added
- Initial implementation of RuneWeave scaffolding tool
- Support for generating Actix Web API services
- Support for generating Cloudflare Workers (Edge API) services
- CLI command structure with `apply` and `verify` subcommands
- JSON schema validation for plan files
- Policy enforcement via YAML configuration
- Deterministic generation with seed support
- Template-based code generation using Tera
- CI/CD workflow generation for GitHub Actions
- Comprehensive test suite including unit and integration tests
- Support for SBOM generation and Cosign signing in CI

### Security
- No known vulnerabilities in dependencies
- All dependencies audited with cargo-audit