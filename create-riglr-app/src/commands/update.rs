//! Update command implementation

use anyhow::{Context, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_TEMPLATE_REPO: &str = "https://github.com/riglr/riglr-templates.git";
const DEFAULT_BRANCH: &str = "main";

/// Update templates from remote repository
#[allow(dead_code)]
pub async fn run() -> Result<()> {
    run_with_options(None, None).await
}

/// Update templates with custom options
pub async fn run_with_options(custom_url: Option<String>, branch: Option<String>) -> Result<()> {
    let repo_url = custom_url.as_deref().unwrap_or(DEFAULT_TEMPLATE_REPO);
    let branch_name = branch.as_deref().unwrap_or(DEFAULT_BRANCH);

    // Check if git is installed
    if !is_git_installed()? {
        return Err(anyhow::anyhow!(
            "Git is not installed or not available in PATH. Please install Git to update templates."
        ));
    }

    println!("{}", style("Updating templates...").cyan());
    println!("Repository: {}", style(repo_url).dim());
    println!("Branch: {}", style(branch_name).dim());

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Fetching latest templates...");

    let cache_dir = get_template_cache_dir()?;

    // Create cache directory if it doesn't exist
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir).context("Failed to create template cache directory")?;
    }

    // Check if repository exists
    if cache_dir.join(".git").exists() {
        pb.set_message("Updating existing templates...");
        update_existing_repo(&cache_dir, branch_name)?;
    } else {
        pb.set_message("Cloning template repository...");
        clone_new_repo(repo_url, &cache_dir, branch_name)?;
    }

    pb.finish_with_message(format!(
        "✅ Templates updated successfully from {}",
        style(branch_name).green()
    ));

    Ok(())
}

/// Get the template cache directory path
pub fn get_template_cache_dir() -> Result<PathBuf> {
    let cache_dir = dirs::cache_dir()
        .context("Could not determine cache directory")?
        .join("create-riglr-app")
        .join("templates");
    Ok(cache_dir)
}

/// Check if git is installed
fn is_git_installed() -> Result<bool> {
    match Command::new("git").arg("--version").output() {
        Ok(output) => Ok(output.status.success()),
        Err(_) => Ok(false),
    }
}

/// Clone a new repository to the cache directory
fn clone_new_repo(url: &str, path: &Path, branch: &str) -> Result<()> {
    let output = Command::new("git")
        .args([
            "clone",
            "--branch",
            branch,
            "--depth",
            "1",
            url,
            path.to_str().unwrap(),
        ])
        .output()
        .context("Failed to execute git clone")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "Failed to clone repository from {}: {}",
            url,
            stderr
        ));
    }

    Ok(())
}

/// Update an existing repository
fn update_existing_repo(path: &Path, branch: &str) -> Result<()> {
    // First, ensure we're on the correct branch
    let output = Command::new("git")
        .current_dir(path)
        .args(["checkout", branch])
        .output()
        .context("Failed to execute git checkout")?;

    if !output.status.success() {
        let _stderr = String::from_utf8_lossy(&output.stderr);
        // Try to create the branch if it doesn't exist
        let output = Command::new("git")
            .current_dir(path)
            .args(["checkout", "-b", branch, &format!("origin/{}", branch)])
            .output()
            .context("Failed to execute git checkout -b")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Failed to checkout branch {}: {}",
                branch,
                stderr
            ));
        }
    }

    // Fetch the latest changes
    let output = Command::new("git")
        .current_dir(path)
        .args(["fetch", "origin", branch])
        .output()
        .context("Failed to execute git fetch")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to fetch from origin: {}", stderr));
    }

    // Pull the latest changes
    let output = Command::new("git")
        .current_dir(path)
        .args(["pull", "origin", branch])
        .output()
        .context("Failed to execute git pull")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Check if it's just "already up to date"
        if stderr.contains("Already up to date") || stderr.contains("Already up-to-date") {
            return Ok(());
        }
        return Err(anyhow::anyhow!(
            "Failed to pull from origin/{}: {}",
            branch,
            stderr
        ));
    }

    Ok(())
}
