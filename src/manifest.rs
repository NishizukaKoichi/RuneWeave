use crate::types::{Plan, Result, ToolchainInfo, WeaveManifest};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub fn create_manifest(plan: &Plan, files: &[String], seed: u64) -> Result<WeaveManifest> {
    let template_hash = calculate_template_hash(files)?;
    let plan_hash = calculate_plan_hash(plan)?;

    let toolchain = ToolchainInfo {
        rust_version: plan
            .project
            .rust_version
            .clone()
            .unwrap_or_else(|| "stable".to_string()),
        targets: vec![
            "wasm32-unknown-unknown".to_string(),
            "x86_64-unknown-linux-musl".to_string(),
        ],
    };

    Ok(WeaveManifest {
        template_hash,
        seed,
        toolchain,
        generated_at: Utc::now(),
        plan_hash,
        files_generated: files.to_vec(),
    })
}

fn calculate_template_hash(files: &[String]) -> Result<String> {
    let mut hasher = Sha256::new();
    
    let mut sorted_files = files.to_vec();
    sorted_files.sort();
    
    for file in &sorted_files {
        hasher.update(file.as_bytes());
        hasher.update(b"\n");
    }
    
    Ok(hex::encode(hasher.finalize()))
}

fn calculate_plan_hash(plan: &Plan) -> Result<String> {
    let plan_json = serde_json::to_string(plan)?;
    let mut hasher = Sha256::new();
    hasher.update(plan_json.as_bytes());
    Ok(hex::encode(hasher.finalize()))
}