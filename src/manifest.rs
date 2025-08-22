use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct WeaveManifest {
    pub template_hash: String,
    pub seed: u64,
    pub toolchain: String,
    pub plan_hash: String,
    pub generated_at: String,
}

pub fn generate_manifest(
    plan_content: &str,
    seed: u64,
    toolchain: &str,
    template_version: &str,
) -> Result<WeaveManifest> {
    let plan_hash = hash_content(plan_content);
    let template_hash = hash_content(template_version);

    Ok(WeaveManifest {
        template_hash,
        seed,
        toolchain: toolchain.to_string(),
        plan_hash,
        generated_at: chrono::Utc::now().to_rfc3339(),
    })
}

pub fn write_manifest(manifest: &WeaveManifest, out_dir: &Path) -> Result<()> {
    let manifest_path = out_dir.join("weave.manifest.json");
    let content = serde_json::to_string_pretty(manifest)?;
    std::fs::write(manifest_path, content)?;
    Ok(())
}

fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}
