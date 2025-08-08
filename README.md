# RuneWeave

Generate Rust-Edge monorepo scaffolds from Runeforge blueprints.

## Overview

RuneWeave takes a `plan.json` file (output from Runeforge) and generates a complete Rust-Edge monorepo with:

- Workspace structure with multiple services
- CI/CD pipeline (GitHub Actions)
- Edge-ready services (Cloudflare Workers)
- SBOM generation and artifact signing
- Deterministic generation with seed support

## Installation

```bash
cargo install runeweave
```

## Usage

### Local Generation

```bash
# Generate scaffold in local directory
runeweave apply -p plan.json --seed 42 --out ./my-product
```

### GitHub Repository Creation

```bash
# Create and push to GitHub repository
runeweave apply -p plan.json --seed 42 --repo github:myorg/my-product --public
```

### With Policy Validation

```bash
# Apply with policy constraints
runeweave apply -p plan.json --policy runeweave.policy.yml --out ./my-product
```

### Verify Only

```bash
# Validate without generating
runeweave apply -p plan.json --verify
```

## Input Format

### plan.json

```json
{
  "project": {
    "name": "my-edge-product",
    "description": "A Rust-Edge monorepo",
    "version": "0.1.0",
    "rust_version": "1.80",
    "repository": "https://github.com/example/my-product",
    "license": "MIT"
  },
  "services": [
    {
      "name": "api",
      "service_type": "api",
      "description": "Main API service",
      "dependencies": ["serde", "tokio"],
      "features": ["auth", "metrics"]
    },
    {
      "name": "api-edge",
      "service_type": "api_edge",
      "description": "Edge API service",
      "dependencies": ["worker"],
      "features": ["cache"]
    }
  ],
  "tools": [
    {
      "name": "cli",
      "description": "CLI tool"
    }
  ],
  "infrastructure": {
    "ci": {
      "provider": "github-actions",
      "features": ["test", "build", "sbom", "sign"]
    },
    "edge": {
      "provider": "cloudflare-workers",
      "regions": ["auto"]
    }
  },
  "metadata": {
    "created_at": "2025-01-01T00:00:00Z",
    "runeforge_version": "0.1.0",
    "schema_version": "1.0.0"
  }
}
```

### runeweave.policy.yml (Optional)

```yaml
allowed_dependencies:
  - serde
  - tokio
  - actix-web
  - worker

banned_dependencies:
  - openssl  # Use rustls instead

allowed_licenses:
  - MIT
  - Apache-2.0
  - "MIT OR Apache-2.0"

required_features:
  - auth
  - metrics

max_crate_size_mb: 50
```

## Generated Structure

```
product/
├── Cargo.toml              # Workspace configuration
├── rust-toolchain.toml     # Rust toolchain specification
├── services/
│   ├── api/                # Actix Web service
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   └── api-edge/           # Cloudflare Workers service
│       ├── Cargo.toml
│       ├── wrangler.toml
│       └── src/
│           └── lib.rs
├── tools/
│   └── cli/                # CLI tool
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
├── .github/
│   └── workflows/
│       └── ci.yml          # CI/CD pipeline
└── weave.manifest.json     # Generation manifest
```

## Features

### Deterministic Generation

Use the `--seed` parameter to ensure reproducible scaffolds:

```bash
runeweave apply -p plan.json --seed 12345
```

The same plan + seed combination always produces identical output.

### Build Verification

RuneWeave automatically verifies the generated scaffold:

- Runs `cargo check --workspace --locked`
- Validates `wrangler.toml` for edge services
- Ensures all dependencies are valid

### CI/CD Pipeline

Generated projects include a complete GitHub Actions workflow:

- Multi-platform builds (x86_64, aarch64, wasm32)
- Automated testing and linting
- SBOM generation with Syft
- Artifact signing with Cosign
- Caching for faster builds

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Schema validation failed |
| 2 | Policy violation |
| 3 | Repository operation failed |
| 4 | Other error |

## Requirements

- Rust 1.80+
- Git (for repository operations)
- GitHub CLI (`gh`) for GitHub repository creation
- Optional: `wrangler` for edge service validation

## License

MIT OR Apache-2.0