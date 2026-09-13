use std::path::Path;
use anyhow::{Result, Context};
use git2::Repository;
use colored::*;

pub async fn clone_repository(git_url: &str, target_dir: &Path) -> Result<()> {
    println!("{} {}", " Cloning repository from:".bright_blue(), git_url.bright_white());

    if !target_dir.exists() {
        std::fs::create_dir_all(target_dir).context("Failed to create target directory")?;
    }
    Repository::clone(git_url, target_dir).context("Failed to clone repository")?;

    println!("{} {}", " Repository cloned successfully to:".bright_green(), target_dir.display().to_string().bright_white());
    Ok(())
}

pub fn validate_git_url(url: &str) -> bool {
    //check ulang regex
    let patterns = [
        r"^https://github\.com/[\w\-\.]+/[\w\-\.]+(?:\.git)?/?$",
        r"^https://gitlab\.com/[\w\-\.]+/[\w\-\.]+(?:\.git)?/?$",
        r"^https://bitbucket\.org/[\w\-\.]+/[\w\-\.]+(?:\.git)?/?$",
        r"^git@github\.com:[\w\-\.]+/[\w\-\.]+\.git$",
        r"^git@gitlab\.com:[\w\-\.]+/[\w\-\.]+\.git$",
        r"^https://.*\.git$",
        r"^git@.*:.*\.git$",
    ];

    patterns.iter().any(|pattern| {
        regex::Regex::new(pattern)
            .map(|re| re.is_match(url))
            .unwrap_or(false)
    })
}

pub fn extract_repo_name(git_url: &str) -> String {
    let url = git_url.trim_end_matches('/').trim_end_matches(".git");

    if let Some(name) = url.split('/').last() {
        name.to_string()
    } else {
        "repository".to_string()
    }
}

pub fn get_git_service(git_url: &str) -> String {
    if git_url.contains("github.com") {
        "GitHub".to_string()
    } else if git_url.contains("gitlab.com") {
        "GitLab".to_string()
    }
    // else if git_url.contains("bitbucket.org") {
    //     "Bitbucket".to_string()
    // } else if git_url.contains("gitea") {
    //     "Gitea".to_string()
    // } else if git_url.contains("dev.azure.com") {
    //     "Azure DevOps".to_string()
    // }
   else {
        "Git Server".to_string()
    }
}
