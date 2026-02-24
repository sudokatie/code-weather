use crate::error::Error;
use chrono::{DateTime, Duration, Utc};
use git2::{Repository, Time};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct GitMetrics {
    pub is_repo: bool,
    pub commits_7d: usize,
    pub commits_30d: usize,
    pub commits_90d: usize,
    pub contributors: usize,
    pub last_commit_days: Option<i64>,
    pub is_abandoned: bool,
}

pub fn analyze_git(path: &Path) -> Result<GitMetrics, Error> {
    let repo = match Repository::discover(path) {
        Ok(r) => r,
        Err(_) => {
            return Ok(GitMetrics {
                is_repo: false,
                ..Default::default()
            });
        }
    };

    let mut commits_7d = 0;
    let mut commits_30d = 0;
    let mut commits_90d = 0;
    let mut contributors: HashSet<String> = HashSet::new();
    let mut last_commit_time: Option<DateTime<Utc>> = None;

    let now = Utc::now();
    let seven_days_ago = now - Duration::days(7);
    let thirty_days_ago = now - Duration::days(30);
    let ninety_days_ago = now - Duration::days(90);

    // Walk commit history
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head().ok(); // Ignore error if no HEAD

    for oid in revwalk.flatten() {
        let commit = match repo.find_commit(oid) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let commit_time = git_time_to_datetime(commit.time());

        // Track last commit
        if last_commit_time.is_none() || commit_time > last_commit_time.unwrap() {
            last_commit_time = Some(commit_time);
        }

        // Count by time period
        if commit_time >= ninety_days_ago {
            commits_90d += 1;
            if commit_time >= thirty_days_ago {
                commits_30d += 1;
                if commit_time >= seven_days_ago {
                    commits_7d += 1;
                }
            }
        }

        // Track contributors
        if let Some(author) = commit.author().email() {
            contributors.insert(author.to_string());
        }

        // Stop after 90 days for performance
        if commit_time < ninety_days_ago {
            break;
        }
    }

    let last_commit_days = last_commit_time.map(|t| (now - t).num_days());
    let is_abandoned = last_commit_days.map(|d| d > 30).unwrap_or(true);

    Ok(GitMetrics {
        is_repo: true,
        commits_7d,
        commits_30d,
        commits_90d,
        contributors: contributors.len(),
        last_commit_days,
        is_abandoned,
    })
}

fn git_time_to_datetime(time: Time) -> DateTime<Utc> {
    DateTime::from_timestamp(time.seconds(), 0)
        .unwrap_or_else(Utc::now)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    fn init_git_repo(dir: &Path) {
        Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .output()
            .expect("git init failed");
        
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(dir)
            .output()
            .expect("git config email failed");
        
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(dir)
            .output()
            .expect("git config name failed");
    }

    fn make_commit(dir: &Path, msg: &str) {
        std::fs::write(dir.join("file.txt"), msg).unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(dir)
            .output()
            .expect("git add failed");
        Command::new("git")
            .args(["commit", "-m", msg])
            .current_dir(dir)
            .output()
            .expect("git commit failed");
    }

    #[test]
    fn test_non_repo_returns_false() {
        let dir = TempDir::new().unwrap();
        let metrics = analyze_git(dir.path()).unwrap();
        assert!(!metrics.is_repo);
    }

    #[test]
    fn test_valid_repo() {
        let dir = TempDir::new().unwrap();
        init_git_repo(dir.path());
        make_commit(dir.path(), "initial");
        
        let metrics = analyze_git(dir.path()).unwrap();
        assert!(metrics.is_repo);
    }

    #[test]
    fn test_counts_recent_commits() {
        let dir = TempDir::new().unwrap();
        init_git_repo(dir.path());
        make_commit(dir.path(), "commit 1");
        make_commit(dir.path(), "commit 2");
        
        let metrics = analyze_git(dir.path()).unwrap();
        assert!(metrics.commits_7d >= 2);
        assert!(metrics.commits_30d >= 2);
    }

    #[test]
    fn test_tracks_contributors() {
        let dir = TempDir::new().unwrap();
        init_git_repo(dir.path());
        make_commit(dir.path(), "commit 1");
        
        let metrics = analyze_git(dir.path()).unwrap();
        assert!(metrics.contributors >= 1);
    }

    #[test]
    fn test_last_commit_days() {
        let dir = TempDir::new().unwrap();
        init_git_repo(dir.path());
        make_commit(dir.path(), "commit 1");
        
        let metrics = analyze_git(dir.path()).unwrap();
        assert!(metrics.last_commit_days.is_some());
        assert!(metrics.last_commit_days.unwrap() < 1);
    }

    #[test]
    fn test_active_repo_not_abandoned() {
        let dir = TempDir::new().unwrap();
        init_git_repo(dir.path());
        make_commit(dir.path(), "commit 1");
        
        let metrics = analyze_git(dir.path()).unwrap();
        assert!(!metrics.is_abandoned);
    }

    #[test]
    fn test_empty_repo_abandoned() {
        let dir = TempDir::new().unwrap();
        init_git_repo(dir.path());
        // No commits
        
        let metrics = analyze_git(dir.path()).unwrap();
        assert!(metrics.is_abandoned);
    }

    #[test]
    fn test_git_time_conversion() {
        let time = Time::new(1700000000, 0);
        let dt = git_time_to_datetime(time);
        assert!(dt.timestamp() == 1700000000);
    }
}
