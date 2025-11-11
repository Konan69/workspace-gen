# wg – Workspace Generator

`wg` is a personal command-line tool that scaffolds Cargo workspaces with sane defaults. It lives inside this repository as a binary crate (`wg/`), so you can hack on it and then install it locally for quick use.

## Quick Start

```bash
# run directly
cargo run -p wg -- new /tmp/demo --lib core --bin api --git --toolchain nightly

# or install the binary for a shorter command
cargo install --path wg
wg new ~/code/my-workspace --lib core --bin cli
```

## Features
- Creates a workspace root with resolver 3, `.gitignore`, optional `rust-toolchain.toml`, and optional `git init`.
- Scaffolds any number of `--lib` and `--bin` members via `cargo new --quiet`, preserving Cargo’s defaults.
- Validates duplicate member names and refuses to overwrite non-empty directories unless `--force` is given.
- Uses `toml_edit` to update `Cargo.toml` so formatting and future fields stay intact.

## Development

```bash
# format + lint
cargo fmt
cargo clippy

# run unit + integration tests
cargo test

# manual smoke test
cargo run -p wg -- new $(mktemp -d)/demo --lib data --bin cli --git --toolchain stable
```

Happy hacking! Feel free to extend `wg` with new subcommands (e.g., `add`, `lint init`) or hook it up to your own templates.
