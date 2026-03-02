use crate::error::Error;
use chrono::{DateTime, Duration, Utc};
use git2::{DiffOptions, Repository, Time};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct ChurnMetrics {
    pub files_changed: usize,
    pub total_files: usize,
    pub churn_percent: f64,
    pub additions: usize,
    pub deletions: usize,
    pub net_change: i64,
    pub trend: ChurnTrend,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ChurnTrend {
    Growing,
    Shrinking,
    Refactoring,
    #[default]
    Stable,
}

pub fn analyze_churn(path: &Path, days: i64) -> Result<ChurnMetrics, Error> {
    let repo = match Repository::discover(path) {
        Ok(r) => r,
        Err(_) => return Ok(ChurnMetrics::default()),
    };

    let cutoff = Utc::now() - Duration::days(days);

    let mut files_changed: HashSet<String> = HashSet::new();
    let mut total_additions: usize = 0;
    let mut total_deletions: usize = 0;

    // Walk commit history
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head().ok();

    let mut prev_tree = None;

    for oid in revwalk.flatten() {
        let commit = match repo.find_commit(oid) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let commit_time = git_time_to_datetime(commit.time());
        if commit_time < cutoff {
            break;
        }

        let tree = match commit.tree() {
            Ok(t) => t,
            Err(_) => continue,
        };

        if let Some(ref prev) = prev_tree {
            let mut opts = DiffOptions::new();
            opts.ignore_submodules(true);

            // Diff from this (older) commit to prev (newer) commit - shows what was added/deleted
            if let Ok(diff) = repo.diff_tree_to_tree(Some(&tree), Some(prev), Some(&mut opts)) {
                // Get changed files
                diff.foreach(
                    &mut |delta, _progress| {
                        if let Some(path) = delta.new_file().path() {
                            // Skip binary files
                            if !is_binary_path(path) {
                                files_changed.insert(path.to_string_lossy().to_string());
                            }
                        }
                        true
                    },
                    None,
                    None,
                    None,
                )
                .ok();

                // Get line stats
                if let Ok(stats) = diff.stats() {
                    total_additions += stats.insertions();
                    total_deletions += stats.deletions();
                }
            }
        }

        prev_tree = Some(tree);
    }

    // Count total files in repo
    let total_files = count_source_files(&repo);

    let churn_percent = if total_files > 0 {
        (files_changed.len() as f64 / total_files as f64) * 100.0
    } else {
        0.0
    };

    let net_change = total_additions as i64 - total_deletions as i64;

    let trend = determine_trend(total_additions, total_deletions, files_changed.len());

    Ok(ChurnMetrics {
        files_changed: files_changed.len(),
        total_files,
        churn_percent,
        additions: total_additions,
        deletions: total_deletions,
        net_change,
        trend,
    })
}

fn determine_trend(additions: usize, deletions: usize, files_changed: usize) -> ChurnTrend {
    if additions == 0 && deletions == 0 {
        return ChurnTrend::Stable;
    }

    let net = additions as i64 - deletions as i64;
    let total = additions + deletions;

    // High churn with net zero = refactoring
    if files_changed > 0 && net.abs() < (total as i64 / 10) {
        return ChurnTrend::Refactoring;
    }

    // Significant growth
    if net > (total as i64 / 4) {
        return ChurnTrend::Growing;
    }

    // Significant shrink
    if net < -(total as i64 / 4) {
        return ChurnTrend::Shrinking;
    }

    ChurnTrend::Stable
}

fn is_binary_path(path: &Path) -> bool {
    let binary_extensions = [
        "png", "jpg", "jpeg", "gif", "ico", "svg", "woff", "woff2", "ttf", "eot", "pdf", "zip",
        "tar", "gz", "exe", "dll", "so", "dylib", "mp3", "mp4", "wav", "avi",
    ];

    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        binary_extensions.contains(&ext.to_lowercase().as_str())
    } else {
        false
    }
}

fn count_source_files(repo: &Repository) -> usize {
    let source_extensions = ["ts", "tsx", "js", "jsx", "py", "rs", "go"];

    if let Ok(head) = repo.head() {
        if let Ok(tree) = head.peel_to_tree() {
            let mut count = 0;
            tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
                if let Some(name) = entry.name() {
                    if let Some(ext) = Path::new(name).extension().and_then(|e| e.to_str()) {
                        if source_extensions.contains(&ext.to_lowercase().as_str()) {
                            count += 1;
                        }
                    }
                }
                git2::TreeWalkResult::Ok
            })
            .ok();
            return count;
        }
    }
    0
}

fn git_time_to_datetime(time: Time) -> DateTime<Utc> {
    DateTime::from_timestamp(time.seconds(), 0).unwrap_or_else(Utc::now)
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
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    fn make_commit(dir: &Path, filename: &str, content: &str) {
        std::fs::write(dir.join(filename), content).unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "update"])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    #[test]
    fn test_no_changes_zero_churn() {
        let dir = TempDir::new().unwrap();
        let metrics = analyze_churn(dir.path(), 30).unwrap();
        assert_eq!(metrics.churn_percent, 0.0);
    }

    #[test]
    fn test_all_files_changed() {
        let dir = TempDir::new().unwrap();
        init_git_repo(dir.path());
        make_commit(dir.path(), "main.ts", "const a = 1;");
        make_commit(dir.path(), "main.ts", "const a = 2;");

        let metrics = analyze_churn(dir.path(), 30).unwrap();
        assert!(metrics.files_changed >= 1);
    }

    #[test]
    fn test_additions_tracked() {
        let dir = TempDir::new().unwrap();
        init_git_repo(dir.path());
        make_commit(dir.path(), "main.ts", "line1");
        make_commit(dir.path(), "main.ts", "line1\nline2\nline3");

        let metrics = analyze_churn(dir.path(), 30).unwrap();
        assert!(metrics.additions >= 2);
    }

    #[test]
    fn test_deletions_tracked() {
        let dir = TempDir::new().unwrap();
        init_git_repo(dir.path());
        make_commit(dir.path(), "main.ts", "line1\nline2\nline3");
        make_commit(dir.path(), "main.ts", "line1");

        let metrics = analyze_churn(dir.path(), 30).unwrap();
        assert!(metrics.deletions >= 2);
    }

    #[test]
    fn test_growing_trend() {
        let trend = determine_trend(100, 10, 5);
        assert_eq!(trend, ChurnTrend::Growing);
    }

    #[test]
    fn test_shrinking_trend() {
        let trend = determine_trend(10, 100, 5);
        assert_eq!(trend, ChurnTrend::Shrinking);
    }

    #[test]
    fn test_refactoring_trend() {
        let trend = determine_trend(100, 100, 10);
        assert_eq!(trend, ChurnTrend::Refactoring);
    }
}
