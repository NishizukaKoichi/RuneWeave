use anyhow::{Context, Result};
use schemars::schema_for;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct StackPlan {
    pub project: String,
    pub services: Vec<Service>,
    pub toolchain: ToolchainConfig,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Service {
    pub name: String,
    pub language: Language,
    pub framework: Option<String>,
    pub runtime: Option<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    Node,
    Python,
    Go,
    Java,
    #[serde(rename = "dotnet")]
    DotNet,
    Deno,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ToolchainConfig {
    pub rust: Option<RustToolchain>,
    pub node: Option<NodeToolchain>,
    pub python: Option<PythonToolchain>,
    pub go: Option<GoToolchain>,
    pub java: Option<JavaToolchain>,
    pub dotnet: Option<DotNetToolchain>,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RustToolchain {
    pub version: String,
    pub targets: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct NodeToolchain {
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PythonToolchain {
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GoToolchain {
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JavaToolchain {
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DotNetToolchain {
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Policy {
    pub version: u32,
    pub deny: Option<DenyPolicy>,
    pub pin: Option<PinPolicy>,
    pub ci: Option<CiPolicy>,
    pub naming: Option<NamingPolicy>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DenyPolicy {
    pub licenses: Option<Vec<String>>,
    pub crates: Option<Vec<String>>,
    pub npm: Option<Vec<String>>,
    pub pypi: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PinPolicy {
    pub rust: Option<RustPin>,
    pub node: Option<NodePin>,
    pub python: Option<PythonPin>,
    pub go: Option<GoPin>,
    pub java: Option<JavaPin>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RustPin {
    pub msrv: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodePin {
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PythonPin {
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoPin {
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JavaPin {
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CiPolicy {
    pub linux_runner: String,
    pub sbom: bool,
    pub cosign: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NamingPolicy {
    pub project: String,
    pub service: String,
}

pub fn verify_plan(plan_path: &Path) -> Result<StackPlan> {
    let plan_content = std::fs::read_to_string(plan_path)
        .with_context(|| format!("Failed to read plan from {plan_path:?}"))?;

    let plan: StackPlan =
        serde_json::from_str(&plan_content).with_context(|| "Failed to parse plan.json")?;

    // Validate schema
    let schema = schema_for!(StackPlan);
    let _ = serde_json::to_value(&schema)?;

    // Validate naming conventions
    if !is_kebab_case(&plan.project) {
        anyhow::bail!("Project name must be in kebab-case");
    }

    for service in &plan.services {
        if !is_kebab_case(&service.name) {
            anyhow::bail!("Service name '{}' must be in kebab-case", service.name);
        }
    }

    Ok(plan)
}

pub fn verify_policy(policy_path: Option<&Path>) -> Result<Option<Policy>> {
    if let Some(path) = policy_path {
        let policy_content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read policy from {path:?}"))?;

        let policy: Policy =
            serde_yaml::from_str(&policy_content).with_context(|| "Failed to parse policy YAML")?;

        if policy.version != 1 {
            anyhow::bail!("Unsupported policy version: {}", policy.version);
        }

        Ok(Some(policy))
    } else {
        Ok(None)
    }
}

fn is_kebab_case(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_lowercase() || c.is_numeric() || c == '-')
        && !s.starts_with('-')
        && !s.ends_with('-')
        && !s.contains("--")
}
