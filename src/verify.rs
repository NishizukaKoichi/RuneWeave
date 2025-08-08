use crate::types::{Plan, Policy, Result, RuneWeaveError};
use std::path::Path;
use std::process::Command;

pub fn validate_plan(plan: &Plan) -> Result<()> {
    if plan.project.name.is_empty() {
        return Err(RuneWeaveError::SchemaValidation(
            "Project name cannot be empty".to_string(),
        ));
    }

    if !is_valid_crate_name(&plan.project.name) {
        return Err(RuneWeaveError::SchemaValidation(
            format!("Invalid project name: {}", plan.project.name),
        ));
    }

    for service in &plan.services {
        if !is_valid_crate_name(&service.name) {
            return Err(RuneWeaveError::SchemaValidation(
                format!("Invalid service name: {}", service.name),
            ));
        }
    }

    if let Some(rust_version) = &plan.project.rust_version {
        if !is_valid_rust_version(rust_version) {
            return Err(RuneWeaveError::SchemaValidation(
                format!("Invalid Rust version: {}", rust_version),
            ));
        }
    }

    Ok(())
}

pub fn check_policy(plan: &Plan, policy: &Policy) -> Result<()> {
    if let Some(banned_deps) = &policy.banned_dependencies {
        for service in &plan.services {
            if let Some(deps) = &service.dependencies {
                for dep in deps {
                    if banned_deps.contains(dep) {
                        return Err(RuneWeaveError::PolicyViolation(
                            format!("Banned dependency: {}", dep),
                        ));
                    }
                }
            }
        }
    }

    if let Some(allowed_licenses) = &policy.allowed_licenses {
        if let Some(license) = &plan.project.license {
            if !allowed_licenses.contains(license) {
                return Err(RuneWeaveError::PolicyViolation(
                    format!("License not allowed: {}", license),
                ));
            }
        }
    }

    if let Some(required_features) = &policy.required_features {
        for feature in required_features {
            let has_feature = plan.services.iter().any(|s| {
                s.features
                    .as_ref()
                    .map(|f| f.contains(feature))
                    .unwrap_or(false)
            });

            if !has_feature {
                return Err(RuneWeaveError::PolicyViolation(
                    format!("Required feature missing: {}", feature),
                ));
            }
        }
    }

    Ok(())
}

pub fn verify_build(project_dir: &Path) -> Result<()> {
    println!("🔍 Verifying build...");

    let output = Command::new("cargo")
        .arg("check")
        .arg("--workspace")
        .arg("--locked")
        .current_dir(project_dir)
        .output()
        .map_err(|e| RuneWeaveError::BuildVerification(format!("Failed to run cargo check: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(RuneWeaveError::BuildVerification(
            format!("cargo check failed:\n{}", stderr),
        ));
    }

    // Check all edge services for wrangler.toml
    let services_dir = project_dir.join("services");
    if services_dir.exists() {
        for entry in std::fs::read_dir(&services_dir)? {
            let entry = entry?;
            let service_path = entry.path();
            let wrangler_path = service_path.join("wrangler.toml");
            
            if wrangler_path.exists() {
                let output = Command::new("wrangler")
                    .arg("validate")
                    .current_dir(&service_path)
                    .output()
                    .map_err(|e| RuneWeaveError::BuildVerification(
                        format!("Failed to run wrangler validate: {}", e)
                    ))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(RuneWeaveError::BuildVerification(
                        format!("wrangler validate failed:\n{}", stderr),
                    ));
                }
                
                println!("✓ wrangler validate passed for {}", service_path.file_name().unwrap().to_string_lossy());
            }
        }
    }

    Ok(())
}

fn is_valid_crate_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        && name.chars().next().unwrap().is_alphabetic()
}

fn is_valid_rust_version(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 2 && parts.len() != 3 {
        return false;
    }

    parts.iter().all(|part| part.parse::<u32>().is_ok())
}