//! Git helpers.
//!
//! Branch lookup is a raw read of `.git/HEAD` (and the gitdir-pointer file
//! for worktrees) — no subprocess. Status uses `git status --porcelain`
//! shelled out, behind a TTL'd disk cache (see [`crate::cache`]).
//!
//! We deliberately don't depend on `gix` here: a small shell-out keeps the
//! binary trim and `git status` is fast enough when cached for 1-5 seconds.

use crate::cache;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Returns the human-readable branch name, or a short SHA if detached.
pub fn read_branch(cwd: &Path) -> Option<String> {
    let head = find_head_file(cwd)?;
    let content = std::fs::read_to_string(&head).ok()?;
    let content = content.trim();
    if let Some(rest) = content.strip_prefix("ref: refs/heads/") {
        return Some(rest.to_string());
    }
    if let Some(rest) = content.strip_prefix("ref: refs/") {
        return Some(rest.to_string());
    }
    // Detached HEAD: short SHA.
    Some(content.chars().take(7).collect())
}

fn find_head_file(cwd: &Path) -> Option<PathBuf> {
    let mut current = cwd;
    loop {
        let dot_git = current.join(".git");
        if dot_git.is_dir() {
            return Some(dot_git.join("HEAD"));
        }
        if dot_git.is_file() {
            // git worktree or submodule: the file points at the real gitdir.
            let content = std::fs::read_to_string(&dot_git).ok()?;
            let path_str = content.strip_prefix("gitdir:")?.trim();
            return Some(PathBuf::from(path_str).join("HEAD"));
        }
        current = current.parent()?;
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub modified: u32,
    pub untracked: u32,
    pub added: u32,
    pub deleted: u32,
    pub renamed: u32,
    pub conflicted: u32,
    pub ahead: u32,
    pub behind: u32,
}

impl GitStatus {
    pub fn is_clean(&self) -> bool {
        self.modified == 0
            && self.untracked == 0
            && self.added == 0
            && self.deleted == 0
            && self.renamed == 0
            && self.conflicted == 0
            && self.ahead == 0
            && self.behind == 0
    }
}

pub fn read_status(cwd: &Path, ttl_secs: u64) -> Option<GitStatus> {
    let cwd_owned = cwd.to_path_buf();
    let key = cache::hash_key("git-status", cwd_owned.as_path());
    cache::get_or_compute(&key, ttl_secs, move || run_git_status(&cwd_owned))
}

fn run_git_status(cwd: &Path) -> Option<GitStatus> {
    let output = Command::new("git")
        .args([
            "-C",
            cwd.to_str()?,
            "status",
            "--porcelain=v1",
            "--branch",
            "-z",
        ])
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(parse_porcelain(&stdout))
}

/// Parses `git status --porcelain=v1 --branch -z` output. NUL-separated.
fn parse_porcelain(s: &str) -> GitStatus {
    let mut status = GitStatus::default();
    // -z uses NUL separators, but the first record (branch info) is also
    // NUL-terminated, so split on '\0'.
    for entry in s.split('\0') {
        if entry.is_empty() {
            continue;
        }
        if let Some(rest) = entry.strip_prefix("## ") {
            // Branch line: "main...origin/main [ahead 2, behind 1]"
            if let Some(start) = rest.find('[')
                && let Some(end) = rest[start..].find(']')
            {
                let inside = &rest[start + 1..start + end];
                for tok in inside.split(", ") {
                    if let Some(n) = tok.strip_prefix("ahead ").and_then(|s| s.parse().ok()) {
                        status.ahead = n;
                    } else if let Some(n) = tok.strip_prefix("behind ").and_then(|s| s.parse().ok())
                    {
                        status.behind = n;
                    }
                }
            }
            continue;
        }
        if entry.len() < 2 {
            continue;
        }
        let code = &entry[..2];
        match code {
            "??" => status.untracked += 1,
            "UU" | "AA" | "DD" | "AU" | "UA" | "DU" | "UD" => status.conflicted += 1,
            _ => {
                let staged = entry.as_bytes()[0];
                let worktree = entry.as_bytes()[1];
                if staged == b'A' {
                    status.added += 1;
                }
                if staged == b'M' || worktree == b'M' {
                    status.modified += 1;
                }
                if staged == b'D' || worktree == b'D' {
                    status.deleted += 1;
                }
                if staged == b'R' {
                    status.renamed += 1;
                }
            }
        }
    }
    status
}
