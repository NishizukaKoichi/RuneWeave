use anyhow::{Context, Result};
use git2::{Repository, Signature};
use std::path::Path;

pub struct GitOps {
    pub repo_url: String,
}

impl GitOps {
    pub fn new(repo_spec: &str) -> Result<Self> {
        // Parse github:owner/repo format
        if !repo_spec.starts_with("github:") {
            anyhow::bail!("Only github: repos are supported currently");
        }

        let parts: Vec<&str> = repo_spec
            .strip_prefix("github:")
            .unwrap()
            .split('/')
            .collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid repo format. Use github:owner/repo");
        }

        let repo_url = format!("https://github.com/{}/{}.git", parts[0], parts[1]);

        Ok(Self { repo_url })
    }

    pub fn push_to_repo(&self, local_path: &Path, branch_name: &str) -> Result<()> {
        // Initialize git repo
        let repo = Repository::init(local_path).context("Failed to initialize git repository")?;

        // Add all files
        let mut index = repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        // Create initial commit
        let sig = Signature::now("RuneWeave", "runeweave@example.com")?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        let oid = repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "Initial scaffold commit",
            &tree,
            &[],
        )?;

        // Create branch
        repo.branch(branch_name, &repo.find_commit(oid)?, false)?;

        // Add remote
        repo.remote("origin", &self.repo_url)?;

        // Note: Actual push would require authentication
        // For now, we just prepare the repo for manual push
        println!("Repository prepared at {local_path:?}");
        println!("To push: cd {local_path:?} && git push -u origin {branch_name}");

        Ok(())
    }
}
