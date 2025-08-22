# RuneWeave

Rust-Edge monorepo scaffolding tool that generates project structures for Cloudflare Workers and other edge environments.

## Overview

RuneWeave takes a `plan.json` file (typically output from Runeforge) and generates a complete Rust-based monorepo with:

- Actix Web API services
- Cloudflare Workers (Edge API) services using workers-rs
- CLI tools for testing and operations
- CI/CD configuration (GitHub Actions)
- Policy enforcement for dependencies and naming conventions

## Installation

```bash
cargo install --path .
```

## Usage

### Basic usage

```bash
# Generate scaffold locally
runeweave apply -p plan.json --seed 99 --out ./my-product

# Verify plan without generating
runeweave verify -p plan.json

# Apply with policy file
runeweave apply -p plan.json --policy runeweave.policy.yml --out ./scaffold
```

### Plan Format

The `plan.json` file should follow this structure:

```json
{
  "project": "my-project",
  "services": [
    {
      "name": "api",
      "type": "api",
      "dependencies": []
    },
    {
      "name": "api-edge",
      "type": "api-edge",
      "dependencies": []
    }
  ],
  "toolchain": {
    "rust_version": "1.80",
    "targets": ["wasm32-unknown-unknown"]
  }
}
```

### Policy File

Optional `runeweave.policy.yml` for enforcing project standards:

```yaml
version: 1
deny:
  licenses: ["AGPL-3.0"]
  crates: ["openssl-sys"]
pin:
  rust_toolchain: "stable"
  msrv: "1.80"
  worker_target: "wasm32-unknown-unknown"
ci:
  linux_runner: "ubuntu-24.04"
  sbom: true
  cosign: true
naming:
  project: "kebab-case"
  service: "kebab-case"
```

## Generated Structure

```
my-product/
├── Cargo.toml                 # Workspace configuration
├── rust-toolchain.toml        # Rust toolchain specification
├── services/
│   ├── api/                   # Actix Web service
│   └── api-edge/              # Cloudflare Workers service
├── tools/cli/                 # CLI tools
├── schemas/                   # JSON schemas
└── .github/workflows/ci.yml   # CI/CD configuration
```

## Features

- **Deterministic Generation**: Same seed produces identical output
- **Schema Validation**: Validates input against JSON schema
- **Policy Enforcement**: Enforces naming conventions and dependency rules
- **CI/CD Ready**: Generates complete GitHub Actions workflows
- **Edge Optimized**: Supports WASM targets for edge deployment

## Requirements

- Rust 1.80+ (MSRV)
- Git (for repository operations)

## License

MIT OR Apache-2.0