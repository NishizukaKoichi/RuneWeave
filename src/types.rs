use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Plan {
    pub project: ProjectConfig,
    pub services: Vec<ServiceConfig>,
    pub tools: Option<Vec<ToolConfig>>,
    pub infrastructure: Option<InfrastructureConfig>,
    pub metadata: PlanMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProjectConfig {
    pub name: String,
    pub description: String,
    pub version: String,
    pub rust_version: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ServiceConfig {
    pub name: String,
    pub service_type: ServiceType,
    pub description: String,
    pub dependencies: Option<Vec<String>>,
    pub features: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ServiceType {
    Api,
    ApiEdge,
    Worker,
    Library,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolConfig {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InfrastructureConfig {
    pub ci: Option<CiConfig>,
    pub edge: Option<EdgeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CiConfig {
    pub provider: String,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EdgeConfig {
    pub provider: String,
    pub regions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanMetadata {
    pub created_at: DateTime<Utc>,
    pub runeforge_version: String,
    pub schema_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaveManifest {
    pub template_hash: String,
    pub seed: u64,
    pub toolchain: ToolchainInfo,
    pub generated_at: DateTime<Utc>,
    pub plan_hash: String,
    pub files_generated: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainInfo {
    pub rust_version: String,
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub allowed_dependencies: Option<Vec<String>>,
    pub banned_dependencies: Option<Vec<String>>,
    pub allowed_licenses: Option<Vec<String>>,
    pub required_features: Option<Vec<String>>,
    pub max_crate_size_mb: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct CliArgs {
    pub plan_path: PathBuf,
    pub seed: Option<u64>,
    pub repo: Option<String>,
    pub policy_path: Option<PathBuf>,
    pub output_dir: Option<PathBuf>,
    pub verify_only: bool,
    pub public: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum RuneWeaveError {
    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),
    
    #[error("Policy violation: {0}")]
    PolicyViolation(String),
    
    #[error("Git operation failed: {0}")]
    GitError(#[from] git2::Error),
    
    #[error("Template rendering failed: {0}")]
    TemplateError(#[from] tera::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Build verification failed: {0}")]
    BuildVerification(String),
}

pub type Result<T> = std::result::Result<T, RuneWeaveError>;