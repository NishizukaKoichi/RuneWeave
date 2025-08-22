# RuneWeave

Polyglot monorepo scaffolding tool that generates multi-language project structures for cloud-native and edge environments.

## Overview

RuneWeave takes a `plan.json` file (typically output from Runeforge) and generates a complete polyglot monorepo with:

- **Multi-language support**: Rust, Node.js/TypeScript, Python, Go, Java, .NET
- **Framework templates**: Actix Web, Fastify, FastAPI, Gin, Spring Boot, and more
- **Edge computing**: Cloudflare Workers support (TypeScript/Rust)
- **CI/CD configuration**: GitHub Actions with language matrix builds
- **Policy enforcement**: Dependencies, licenses, and naming conventions
- **Deterministic generation**: Same input + seed = identical output

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
      "name": "api-rs",
      "language": "rust",
      "framework": "actix",
      "runtime": null,
      "dependencies": []
    },
    {
      "name": "api-ts",
      "language": "node",
      "framework": "fastify",
      "runtime": null,
      "dependencies": []
    },
    {
      "name": "worker-cf",
      "language": "node",
      "framework": "hono",
      "runtime": "cloudflare",
      "dependencies": []
    }
  ],
  "toolchain": {
    "rust": {
      "version": "1.82",
      "targets": ["wasm32-unknown-unknown"]
    },
    "node": {
      "version": "22.6.0"
    }
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
  npm: ["left-pad@*"]
  pypi: ["cryptography<42.0"]
pin:
  rust:
    msrv: "1.82"
  node:
    version: "22.6.0"
  python:
    version: "3.12.6"
  go:
    version: "1.22.5"
  java:
    version: "21"
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
├── services/
│   ├── api-rs/           # Rust/Actix service
│   ├── api-ts/           # Node/Fastify service
│   ├── worker-cf/        # Cloudflare Workers (TS/Rust)
│   ├── job-py/           # Python service
│   └── job-go/           # Go service
├── toolchain/            # Version files for each language
│   ├── rust-toolchain.toml
│   ├── .node-version
│   ├── .python-version
│   ├── go.mod
│   └── .java-version
├── schemas/              # JSON schemas
└── .github/workflows/ci.yml   # Multi-language CI/CD
```

## Features

- **Deterministic Generation**: Same seed produces identical output
- **Schema Validation**: Validates input against JSON schema
- **Policy Enforcement**: Enforces naming conventions and dependency rules
- **CI/CD Ready**: Generates complete GitHub Actions workflows with language matrix
- **Edge Optimized**: Supports WASM targets and Cloudflare Workers
- **Language Pack System**: Extensible architecture for adding new languages
- **Multi-toolchain Support**: Manages versions for all supported languages

## Supported Languages & Frameworks

- **Rust**: Actix Web, Workers-rs
- **Node.js/TypeScript**: Fastify, Hono (for Cloudflare Workers)
- **Python**: FastAPI, Poetry package manager
- **Go**: Gin, Fiber, standard library
- **Java**: Spring Boot, Maven
- **.NET**: (Coming soon)
- **Deno**: (Coming soon)

## Requirements

- Rust 1.82+ (MSRV)
- Git (for repository operations)

## License

MIT OR Apache-2.0