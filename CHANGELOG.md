# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-08-22

### Added
- **Polyglot support**: Now generates multi-language monorepos
  - Rust (Actix Web, Workers)
  - Node.js/TypeScript (Fastify, Hono for Cloudflare Workers)
  - Python (FastAPI, Poetry)
  - Go (Gin, Fiber, stdlib)
  - Java (Spring Boot, Maven)
- Language pack plugin system for extensibility
- Multi-language CI/CD matrix builds
- Toolchain directory with version files for each language
- Extended policy support for language-specific dependencies

### Changed
- Service definition now uses `language` field instead of `type`
- Toolchain configuration restructured to support multiple languages
- CI workflow uses matrix strategy for parallel language builds

### Security
- Extended dependency scanning for npm, PyPI packages
- All dependencies audited with cargo-audit

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