use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command as StdCommand;

use anyhow::{Context, Result, anyhow, bail};
use camino::{Utf8Path, Utf8PathBuf};
use clap::{Args, ValueHint};
use toml_edit::{Array, DocumentMut, Item, Table, value};

#[derive(Debug, Args)]
pub struct NewArgs {
    /// Directory where the workspace will be created (created if missing)
    #[arg(value_hint = ValueHint::DirPath)]
    path: Utf8PathBuf,

    /// Library crate to scaffold (repeatable)
    #[arg(long = "lib", value_name = "NAME")]
    libs: Vec<String>,

    /// Binary crate to scaffold (repeatable)
    #[arg(long = "bin", value_name = "NAME")]
    bins: Vec<String>,

    /// Initialize a git repository in the new workspace
    #[arg(long = "git")]
    git: bool,

    /// Overwrite non-empty directories (dangerous)
    #[arg(long = "force")]
    force: bool,

    /// Optional rust-toolchain.toml channel (e.g. stable, nightly)
    #[arg(long = "toolchain", value_name = "CHANNEL")]
    toolchain: Option<String>,
}

impl NewArgs {
    pub fn run(&self) -> Result<()> {
        ensure_no_duplicate_members(&self.libs, &self.bins)?;

        let root = resolve_path(&self.path)?;
        ensure_target_dir(&root, self.force)?;

        // Create baseline workspace manifest before scaffolding members so
        // `cargo new` has a valid root manifest to inspect.
        write_workspace_manifest(&root, &[])?;
        write_gitignore(&root)?;
        if let Some(channel) = &self.toolchain {
            write_toolchain_file(&root, channel)?;
        }

        if self.git {
            init_git_repo(&root)?;
        }

        scaffold_members(&root, &self.libs, MemberKind::Lib)?;
        scaffold_members(&root, &self.bins, MemberKind::Bin)?;

        let members = collect_members(&self.libs, &self.bins);
        write_workspace_manifest(&root, &members)?;

        println!("workspace created at {}", root);
        if !members.is_empty() {
            println!("members: {}", members.join(", "));
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
enum MemberKind {
    Lib,
    Bin,
}

fn resolve_path(input: &Utf8Path) -> Result<Utf8PathBuf> {
    if input.is_absolute() {
        return Ok(input.to_path_buf());
    }

    let cwd = std::env::current_dir().context("failed to read current directory")?;
    let mut buf = PathBuf::from(cwd);
    buf.push(input);
    Utf8PathBuf::from_path_buf(buf)
        .map_err(|_| anyhow!("target path is not valid UTF-8: {}", input))
}

fn ensure_target_dir(path: &Utf8Path, force: bool) -> Result<()> {
    if path.exists() {
        if !path.is_dir() {
            bail!("{path} exists and is not a directory");
        }

        if !force && fs::read_dir(path)?.next().is_some() {
            bail!("directory {} is not empty (use --force to override)", path);
        }
    } else {
        fs::create_dir_all(path)
            .with_context(|| format!("failed to create workspace directory {path}"))?;
    }

    Ok(())
}

fn collect_members(libs: &[String], bins: &[String]) -> Vec<String> {
    let mut combined = Vec::with_capacity(libs.len() + bins.len());
    combined.extend(libs.iter().cloned());
    combined.extend(bins.iter().cloned());
    combined
}

fn ensure_no_duplicate_members(libs: &[String], bins: &[String]) -> Result<()> {
    let mut seen = HashSet::new();
    for name in libs.iter().chain(bins.iter()) {
        if !seen.insert(name) {
            bail!("member {name} declared multiple times");
        }
    }
    Ok(())
}

fn write_workspace_manifest(root: &Utf8Path, members: &[String]) -> Result<()> {
    let cargo_toml = root.join("Cargo.toml");
    let mut doc = if cargo_toml.exists() {
        let contents = fs::read_to_string(&cargo_toml)
            .with_context(|| format!("failed to read {}", cargo_toml))?;
        contents
            .parse::<DocumentMut>()
            .with_context(|| format!("failed to parse {}", cargo_toml))?
    } else {
        "[workspace]\n".parse::<DocumentMut>().unwrap()
    };

    let workspace = ensure_workspace_table(&mut doc)?;
    workspace["resolver"] = value("3");

    let mut array = Array::default();
    for member in members {
        array.push(member.clone());
    }
    workspace["members"] = Item::Value(array.into());

    fs::write(&cargo_toml, doc.to_string())
        .with_context(|| format!("failed to write {}", cargo_toml))?;

    Ok(())
}

fn ensure_workspace_table(doc: &mut DocumentMut) -> Result<&mut Table> {
    let entry = doc
        .entry("workspace")
        .or_insert(Item::Table(Table::default()));

    entry
        .as_table_mut()
        .ok_or_else(|| anyhow!("workspace section is not a table"))
}

fn write_gitignore(root: &Utf8Path) -> Result<()> {
    let gitignore = root.join(".gitignore");
    if gitignore.exists() {
        return Ok(());
    }

    let mut file =
        fs::File::create(&gitignore).with_context(|| format!("failed to create {}", gitignore))?;
    file.write_all(b"/target\nCargo.lock\n")
        .context("failed to write .gitignore")?;
    Ok(())
}

fn write_toolchain_file(root: &Utf8Path, channel: &str) -> Result<()> {
    let path = root.join("rust-toolchain.toml");
    let contents = format!("[toolchain]\nchannel = \"{channel}\"\n");
    fs::write(&path, contents).with_context(|| format!("failed to write {}", path))?;
    Ok(())
}

fn init_git_repo(root: &Utf8Path) -> Result<()> {
    let status = StdCommand::new("git")
        .arg("init")
        .current_dir(root)
        .status()
        .context("failed to run git init")?;

    if !status.success() {
        return Err(anyhow!("git init exited with {}", status));
    }

    Ok(())
}

fn scaffold_members(root: &Utf8Path, names: &[String], kind: MemberKind) -> Result<()> {
    for name in names {
        let member_dir = root.join(name);
        if member_dir.exists() {
            bail!("member {name} already exists at {}", member_dir);
        }

        let mut cmd = StdCommand::new("cargo");
        cmd.arg("new")
            .arg(name)
            .arg("--edition")
            .arg("2024")
            .arg("--vcs")
            .arg("none")
            .arg("--quiet");

        match kind {
            MemberKind::Lib => {
                cmd.arg("--lib");
            }
            MemberKind::Bin => {
                cmd.arg("--bin");
            }
        }

        let status = cmd
            .current_dir(root)
            .status()
            .with_context(|| format!("failed to run cargo new for {name}"))?;

        if !status.success() {
            bail!("cargo new failed for {name} with {status}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_detection() {
        let libs = vec!["core".into()];
        let bins = vec!["core".into()];
        let err = ensure_no_duplicate_members(&libs, &bins).unwrap_err();
        assert!(err.to_string().contains("core"));
    }
}
