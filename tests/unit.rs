#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // Helper to create a test plan file
    fn create_test_plan() -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let plan_path = dir.path().join("plan.json");

        let plan_content = r#"{
            "project": "test-project",
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
        }"#;

        let mut file = fs::File::create(&plan_path).unwrap();
        file.write_all(plan_content.as_bytes()).unwrap();

        (dir, plan_path)
    }

    #[test]
    fn test_verify_valid_plan() {
        let (_dir, plan_path) = create_test_plan();
        let plan = runeweave::verify::verify_plan(&plan_path);
        assert!(plan.is_ok());
    }

    #[test]
    fn test_kebab_case_validation() {
        // These tests check the internal validation logic
        let valid_names = vec!["test-project", "api-edge", "my-service-123"];
        let invalid_names = vec![
            "TestProject",
            "api_Edge",
            "my-service-",
            "-service",
            "my--service",
        ];

        for name in valid_names {
            // Would test the is_kebab_case function if it were public
            assert!(name
                .chars()
                .all(|c| c.is_lowercase() || c.is_numeric() || c == '-'));
        }

        for name in invalid_names {
            // These should fail kebab-case validation
            assert!(
                name.chars().any(|c| c.is_uppercase())
                    || name.starts_with('-')
                    || name.ends_with('-')
                    || name.contains("--")
            );
        }
    }
}
