use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_plan_validation() {
    let plan_json = r#"{
        "project": {
            "name": "test-project",
            "description": "Test project",
            "version": "0.1.0"
        },
        "services": [
            {
                "name": "test-service",
                "service_type": "api",
                "description": "Test service"
            }
        ],
        "metadata": {
            "created_at": "2025-01-01T00:00:00Z",
            "runeforge_version": "0.1.0",
            "schema_version": "1.0.0"
        }
    }"#;

    let plan: runeweave::types::Plan = serde_json::from_str(plan_json).unwrap();
    assert_eq!(plan.project.name, "test-project");
    assert_eq!(plan.services.len(), 1);
}

#[test]
fn test_scaffold_generation() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("plan.json");
    
    let plan_json = r#"{
        "project": {
            "name": "test-scaffold",
            "description": "Test scaffold",
            "version": "0.1.0"
        },
        "services": [
            {
                "name": "api",
                "service_type": "api",
                "description": "API service"
            }
        ],
        "metadata": {
            "created_at": "2025-01-01T00:00:00Z",
            "runeforge_version": "0.1.0",
            "schema_version": "1.0.0"
        }
    }"#;
    
    fs::write(&plan_path, plan_json).unwrap();
    
    // TODO: Add actual CLI invocation test once the binary is built
}

#[test]
fn test_deterministic_generation() {
    // Test that same seed produces same output
    let plan = runeweave::types::Plan {
        project: runeweave::types::ProjectConfig {
            name: "deterministic-test".to_string(),
            description: "Test deterministic generation".to_string(),
            version: "0.1.0".to_string(),
            rust_version: Some("1.80".to_string()),
            repository: None,
            license: None,
        },
        services: vec![],
        tools: None,
        infrastructure: None,
        metadata: runeweave::types::PlanMetadata {
            created_at: chrono::Utc::now(),
            runeforge_version: "0.1.0".to_string(),
            schema_version: "1.0.0".to_string(),
        },
    };
    
    // TODO: Implement deterministic test once render module is public
}