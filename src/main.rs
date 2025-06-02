use anyhow::{Context, Result};
use clap::Parser;
use git2::{Commit, Repository};
use log::{debug, info, warn};
use std::path::PathBuf;
use std::process::Command;

/// A tool to trace the last Git commit where a specific script was working fine.
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Path to the file to check
    #[clap(long, short = 'f')]
    file: PathBuf,

    /// Command to run to check if the file works
    #[clap(long, short = 'c')]
    cmd: String,

    /// Path to the Git repository (defaults to current directory)
    #[clap(long, short = 'r', default_value = ".")]
    repo_path: PathBuf,

    /// Restore the working tree to the original state after completion
    #[clap(long, short = 'R', default_value = "true")]
    restore: bool,

    /// Verbose output
    #[clap(long, short = 'v')]
    verbose: bool,

    /// Use pytest shorthand mode (automatically formats pytest command)
    #[clap(long, short = 'p')]
    pytest: bool,

    /// Test name for pytest (class::method format, used with --pytest)
    #[clap(long, short = 't')]
    test: Option<String>,
}

fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(
        if args.verbose { "debug" } else { "info" },
    ))
    .init();

    info!("Starting tracegit");
    debug!("Arguments: {:?}", args);

    // Save current HEAD to restore later if needed
    let repo = Repository::open(&args.repo_path)
        .with_context(|| format!("Failed to open repository at {:?}", args.repo_path))?;

    let original_head = repo
        .head()
        .with_context(|| "Failed to get current HEAD")?;

    let original_head_commit = original_head
        .peel_to_commit()
        .with_context(|| "Failed to get current HEAD commit")?;

    info!("Current HEAD is at commit: {}", original_head_commit.id());

    // Set up revision walker
    let mut revwalk = repo.revwalk().with_context(|| "Failed to create revision walker")?;
    revwalk
        .push_head()
        .with_context(|| "Failed to push HEAD to revision walker")?;

    // Track if we found a working commit
    let mut found_working_commit = false;

    // Traverse commits
    for oid_result in revwalk {
        let oid = oid_result.with_context(|| "Failed to get commit OID")?;
        let commit = repo
            .find_commit(oid)
            .with_context(|| format!("Failed to find commit {}", oid))?;

        debug!("Checking commit: {} ({})", commit.id(), commit.summary().unwrap_or("No summary"));

        // Extract the actual file path for pytest-style paths (file::class::method)
        let file_path_str = args.file.to_string_lossy().to_string();
        let actual_file_path = if file_path_str.contains("::") {
            PathBuf::from(file_path_str.split("::").next().unwrap())
        } else {
            args.file.clone()
        };

        // Check if the file exists in this commit
        let file_exists = repo.revparse_single(&format!("{}:{}", commit.id(), actual_file_path.display()))
            .is_ok();

        if !file_exists {
            debug!("File {:?} does not exist in commit {}", actual_file_path, commit.id());
            continue;
        }

        // Prepare command based on arguments
        let effective_cmd = if args.pytest {
            // If pytest mode is enabled, format the command appropriately
            let test_path = if let Some(test) = &args.test {
                // If test is specified, use it with the file path
                format!("{}::{}", args.file.display(), test)
            } else {
                // Otherwise, just use the file path
                args.file.display().to_string()
            };
            format!("pytest {}", test_path)
        } else {
            // Use the command as provided
            args.cmd.clone()
        };

        // Check if this commit works
        if check_commit(&repo, &commit, &effective_cmd, &args.file)? {
            info!("Found working commit: {}", commit.id());
            info!("Commit message: {}", commit.message().unwrap_or("No message"));
            info!("Commit date: {}", commit.time().seconds());
            found_working_commit = true;
            break;
        }
    }

    // Restore original HEAD if requested
    if args.restore {
        info!("Restoring original HEAD");
        restore_head(&repo, &original_head_commit)?;
    }

    if !found_working_commit {
        warn!("No working commit found in the history");
    }

    Ok(())
}

/// Check if a commit works by checking out the commit and running the command
fn check_commit(repo: &Repository, commit: &Commit, cmd: &str, file_path: &PathBuf) -> Result<bool> {
    // Checkout the commit
    let tree = commit
        .tree()
        .with_context(|| format!("Failed to get tree for commit {}", commit.id()))?;

    // Convert tree to object before checkout
    let obj = tree.as_object();
    repo.checkout_tree(obj, None)
        .with_context(|| format!("Failed to checkout tree for commit {}", commit.id()))?;

    repo.set_head_detached(commit.id())
        .with_context(|| format!("Failed to set HEAD to commit {}", commit.id()))?;

    // Run the command
    // For commands that start with "pytest", assume the file path is already included
    let effective_cmd = if cmd.starts_with("pytest ") {
        // If it's a pytest command, use it as is (we've already formatted it correctly)
        cmd.to_string()
    } else {
        // For other commands, check if the file path is already included
        let file_str = file_path.to_string_lossy().to_string();
        if cmd.contains(&file_str) {
            // If the command already includes the file path, use it as is
            cmd.to_string()
        } else {
            // Otherwise, append the file path to the command
            // This works for simple commands like "python" as well as testing frameworks
            format!("{} {}", cmd, file_path.display())
        }
    };

    debug!("Running command: {}", effective_cmd);
    let output = Command::new("sh")
        .arg("-c")
        .arg(&effective_cmd)
        .output()
        .with_context(|| format!("Failed to execute command: {}", effective_cmd))?;

    // Check if the command succeeded
    let success = output.status.success();
    if success {
        debug!("Command succeeded");
    } else {
        debug!(
            "Command failed with exit code: {}",
            output.status.code().unwrap_or(-1)
        );
        if !output.stderr.is_empty() {
            debug!(
                "Command stderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    Ok(success)
}

/// Restore the repository to the original HEAD
fn restore_head(repo: &Repository, original_head: &Commit) -> Result<()> {
    let tree = original_head
        .tree()
        .with_context(|| "Failed to get tree for original HEAD")?;

    // Convert tree to object before checkout
    let obj = tree.as_object();
    repo.checkout_tree(obj, None)
        .with_context(|| "Failed to checkout tree for original HEAD")?;

    repo.set_head_detached(original_head.id())
        .with_context(|| "Failed to set HEAD to original commit")?;

    Ok(())
}
