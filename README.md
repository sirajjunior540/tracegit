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

### Mac M4 (Apple Silicon) Considerations

If you're using a Mac with Apple Silicon (M1, M2, M3, M4), you might need to ensure that your Rust toolchain is set up for the arm64 architecture:

```bash
# Check your current Rust target
rustc --print target-list | grep aarch64

# If needed, add the aarch64-apple-darwin target
rustup target add aarch64-apple-darwin
```

You may also need to install the Xcode Command Line Tools:

```bash
xcode-select --install
```

If you encounter issues with the git2 dependency, make sure you have the required libraries:

```bash
brew install libgit2 openssl
```

### Building from Source

```bash
# Clone the repository
git clone https://github.com/sirajjunior540/tracegit.git
cd tracegit

# Build the project
cargo build --release

# The binary will be available at target/release/tracegit
```

## Usage

Basic usage:

```bash
tracegit --file=path/to/your/script.py --cmd="python"
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
    -p, --pytest                 Use pytest shorthand mode (automatically formats pytest command)
    -t, --test <TEST>            Test name for pytest (class::method format, used with --pytest)
    -h, --help                   Print help information
    -V, --version                Print version information
```

### Examples

Find when a Python script stopped working:
```bash
tracegit --file=scripts/analyze.py --cmd="python" --verbose
```

Find when a test started failing:
```bash
tracegit --file=tests/unit_test.js --cmd="npm test" --verbose
```

Find when a Python test started failing using pytest:
```bash
tracegit --file=tests/test_feature.py --cmd="pytest" --verbose
```

Find when a specific pytest test started failing using the pytest shorthand mode:
```bash
tracegit --file=tests/test_feature.py --pytest --verbose
```

Find when a specific pytest test method started failing using the pytest shorthand mode:
```bash
tracegit --file=tests/test_feature.py --pytest --test="TestClass::test_method" --verbose
```

The above command is equivalent to:
```bash
tracegit --file=tests/test_feature.py --cmd="pytest tests/test_feature.py::TestClass::test_method" --verbose
```

Check a specific repository:
```bash
tracegit --file=app.js --cmd="node" --repo-path=/path/to/repository
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
