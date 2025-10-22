# git-helper

A CLI tool built with Rust that automates common Git workflows, particularly intelligent branch synchronization with rebasing.

## Features

- **Smart Sync**: Automatically synchronize your branches with the main branch
  - Fetches and prunes remote branches
  - Fast-forwards the main branch to match remote
  - Rebases current branch onto updated main (when needed)
  - Optionally pushes changes (fast-forward only)
- **Dry-run by default**: Preview changes before applying them
- **Configurable**: Use a TOML config file or command-line overrides

## Installation

### Build from source

```bash
git clone <repository-url>
cd git-helper
cargo build --release
```

The binary will be available at `target/release/git-helper`.

Optionally, install it to your cargo bin directory:

```bash
cargo install --path .
```

## Usage

### Sync Command

The `sync` command keeps your branches up-to-date with the main branch.

#### Basic usage (dry-run mode)

```bash
git-helper sync
```

This will show you what operations would be performed without actually executing them.

#### Execute the sync

```bash
git-helper --no-dry-run sync
```

#### Sync with push

```bash
git-helper --no-dry-run sync --push
```

This will also push your branches to the remote (only if fast-forward is possible).

#### Specify main branch

```bash
git-helper sync --main develop
```

#### Non-interactive rebase

```bash
git-helper sync --non-interactive
```

### Configuration File

You can create a configuration file to set defaults:

```toml
[defaults]
main = "main"        # Default branch name
remote = "origin"    # Default remote name
```

Use it with:

```bash
git-helper --config path/to/config.toml sync
```

### What the sync command does

1. **Fetch and prune** from the remote repository
2. **Fast-forward** the main branch to match its remote tracking branch
3. **Rebase** the current branch onto the updated main branch (if not already up-to-date)
4. **Push** branches to remote (only with `--push` flag and only if fast-forward is possible)

## Examples

### Typical workflow

```bash
# On feature branch, sync with main and push
git-helper --no-dry-run sync --push

# Preview what would happen
git-helper sync

# Sync with a different main branch
git-helper --no-dry-run sync --main develop
```

## Development

### Running tests

```bash
cargo test
```

### Building

```bash
cargo build          # Debug build
cargo build --release # Release build
```

### Linting

```bash
cargo clippy
```

### Formatting

```bash
cargo fmt
```

## How it works

The tool uses a plan-based approach:

1. **Plan Phase**: Analyzes the repository state and builds a sequence of operations
2. **Display Phase**: Shows the planned operations to the user
3. **Execution Phase**: Applies the operations (unless in dry-run mode)

This ensures you always know what will happen before it happens.
