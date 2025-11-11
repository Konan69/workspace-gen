# wg – Workspace Generator

`wg` is a single-binary CLI that scaffolds Cargo workspaces with consistent defaults. Use it to bootstrap new Rust monorepos with a couple of commands, then keep hacking inside this repo to evolve the tool.

## Features
- Creates the workspace root with resolver 3, `Cargo.toml`, `.gitignore`, optional `rust-toolchain.toml`, and can `git init` for you.
- Adds any mix of `--lib`/`--bin` members via `cargo new --quiet`, preserving Cargo defaults.
- Guards against duplicate crate names and refuses to touch non-empty directories unless `--force` is set.
- Relies on `toml_edit` so it never clobbers formatting or unrelated keys when editing manifests.

## Installation

You have two easy options:

- **Install once, run anywhere**
  ```bash
  cargo install --path wg           # from a local checkout
  # or, once published, use: cargo install wg
  ```

- **Run ad hoc without installing**
  ```bash
  cargo run -p wg -- <command>
  ```

After installation, make sure `~/.cargo/bin` is on your `PATH` so you can just call `wg`.

## Usage

```bash
# Create a workspace with libs + bins
wg new ~/code/my-workspace \
  --lib core --lib data \
  --bin api --bin cli \
  --git --toolchain nightly
```

### Global commands
- `wg new PATH [options]` – Create a new workspace at `PATH`.
- `wg help` – Show top-level or command-specific help.

### `wg new` options
- `--lib <NAME>`: scaffold a library crate (repeatable).
- `--bin <NAME>`: scaffold a binary crate (repeatable).
- `--git`: initialize a Git repo in the new workspace.
- `--force`: overwrite non-empty directories (use with care).
- `--toolchain <CHANNEL>`: write `rust-toolchain.toml` with the given channel (e.g., `stable`, `nightly`).
- `-h`, `--help`: command help.

## Quick Start

```bash
cargo install --path wg
wg new /tmp/demo --lib core --bin api --git --toolchain nightly
```

## Development

```bash
cargo fmt
cargo clippy
cargo test
cargo run -p wg -- new $(mktemp -d)/demo --lib data --bin cli --git --toolchain stable
```

Have ideas for new subcommands or templates? Open an issue or PR!
