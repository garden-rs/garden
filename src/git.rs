use super::cmd;
use super::errors;
use super::model::GitTreeDetails;
use super::model::GitTreeType;
use super::path;

/// Return Ok(garden::model::GitTreeDetails) for the specified path on success
/// or Err(garden::errors::CommandError) when Git commands error out.
pub fn worktree_details(pathbuf: &std::path::Path) -> Result<GitTreeDetails, errors::CommandError> {
    let mut worktree_count = 0;
    let cmd = ["git", "worktree", "list", "--porcelain"];
    let path = path::abspath(pathbuf);
    let exec = cmd::exec_in_dir(&cmd, &path);
    let output = cmd::capture(exec)?.stdout_str();

    let worktree_token = "worktree ";
    let branch_token = "branch refs/heads/";
    let bare_token = "bare";

    let path_str = path.to_string_lossy().to_string();
    let mut parent_path = String::new();
    let mut branch = String::new();
    let mut is_current = false;
    let mut is_bare = false;

    for line in output.lines() {
        if let Some(worktree_path) = line.strip_prefix(worktree_token) {
            is_current = worktree_path == path_str;
            // The first worktree is the "parent" worktree.
            if worktree_count == 0 {
                parent_path = worktree_path.to_string();
            }
            worktree_count += 1;
        } else if is_current && line.starts_with(branch_token) {
            branch = line[branch_token.len()..].to_string();
        } else if is_current && line == bare_token {
            // Is this a bare repository?
            is_bare = true;
        }
    }

    // 0 or 1 worktrees implies that this is a regular worktree.
    // 0 doesn't happen in practice.
    if worktree_count < 2 {
        return Ok(GitTreeDetails {
            branch,
            tree_type: match is_bare {
                true => GitTreeType::Bare,
                false => GitTreeType::Tree,
            },
        });
    }

    if path_str == parent_path {
        return Ok(GitTreeDetails {
            branch,
            tree_type: GitTreeType::Parent,
        });
    }

    Ok(GitTreeDetails {
        branch,
        tree_type: GitTreeType::Worktree(parent_path),
    })
}
