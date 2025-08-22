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
    pub r#type: ServiceType,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceType {
    Api,
    ApiEdge,
    Cli,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ToolchainConfig {
    pub rust_version: String,
    pub targets: Vec<String>,
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
    pub licenses: Vec<String>,
    pub crates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PinPolicy {
    pub rust_toolchain: String,
    pub msrv: String,
    pub worker_target: String,
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
