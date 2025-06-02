# Git Commit Tracer

A command-line tool that helps you find the last working commit for a specific file in your Git repository. This is especially useful for debugging regressions in large codebases.

## What It Does

Have you ever made changes to your code and suddenly found that a script or feature that was working before now fails? This tool helps you identify exactly when the problem was introduced by:

1. Traversing your Git commit history
2. Checking out each commit
3. Running your specified command (e.g., a test script)
4. Finding the last commit where the command succeeded

## Installation

### Prerequisites

- Rust and Cargo (install from [rustup.rs](https://rustup.rs/))
- Git (must be installed and available in your PATH)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/sirajjunior540/tracegit.git
cd git-commit-tracer

# Build the project
cargo build --release

# The binary will be available at target/release/git2
```

## Usage

Basic usage:

```bash
git2 --file=path/to/your/script.py --cmd="python"
```

This will:
1. Go through your Git history
2. Check out each commit
3. Run `python path/to/your/script.py` at each commit
4. Stop and report the last commit where the script executed successfully

### Command Line Options

```
OPTIONS:
    -f, --file <FILE>            Path to the file to check
    -c, --cmd <CMD>              Command to run to check if the file works
    -r, --repo-path <REPO_PATH>  Path to the Git repository [default: .]
    -R, --restore <RESTORE>      Restore the working tree to the original state after completion [default: true]
    -v, --verbose                Enable verbose output
    -h, --help                   Print help information
    -V, --version                Print version information
```

### Examples

Find when a Python script stopped working:
```bash
git2 --file=scripts/analyze.py --cmd="python" --verbose
```

Find when a test started failing:
```bash
git2 --file=tests/unit_test.js --cmd="npm test" --verbose
```

Check a specific repository:
```bash
git2 --file=app.js --cmd="node" --repo-path=/path/to/repository
```

## How It Works

The tool uses the following process:

1. Parses command-line arguments to determine which file to check and what command to run
2. Saves the current HEAD position to restore it later
3. Creates a Git revision walker to traverse the commit history
4. For each commit:
   - Checks if the specified file exists in that commit
   - Checks out the commit
   - Runs the specified command
   - If the command succeeds (returns exit code 0), identifies this as the last working commit
5. Restores the repository to its original state (if requested)
6. Reports the results

## Logging

The tool provides two levels of logging:
- Normal mode: Shows only important information
- Verbose mode (`--verbose`): Shows detailed debugging information

## License

This project is licensed under the MIT License - see the LICENSE file for details.