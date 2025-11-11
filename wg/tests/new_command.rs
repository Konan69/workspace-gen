use std::fs;

use predicates::str::contains;
use tempfile::tempdir;
use toml_edit::DocumentMut;

#[test]
fn new_workspace_with_members_and_toolchain() {
    let temp = tempdir().expect("temp dir");
    let workspace_dir = temp.path().join("demo");

    assert_cmd::cargo::cargo_bin_cmd!("wg")
        .args([
            "new",
            workspace_dir.to_str().unwrap(),
            "--lib",
            "corelib",
            "--bin",
            "applib",
            "--toolchain",
            "nightly",
        ])
        .assert()
        .success();

    assert!(workspace_dir.join("corelib").is_dir());
    assert!(workspace_dir.join("applib").is_dir());
    assert!(workspace_dir.join("rust-toolchain.toml").exists());
    assert!(workspace_dir.join(".gitignore").exists());

    let manifest = fs::read_to_string(workspace_dir.join("Cargo.toml")).expect("manifest readable");
    let doc = manifest
        .parse::<DocumentMut>()
        .expect("manifest parses as toml");
    let workspace = doc["workspace"]
        .as_table()
        .expect("workspace table present");

    assert_eq!(
        workspace["resolver"].as_str(),
        Some("3"),
        "resolver defaults to 3"
    );

    let members = workspace["members"].as_array().expect("members array");
    let member_list: Vec<_> = members
        .iter()
        .map(|item| item.as_str().unwrap().to_string())
        .collect();

    assert_eq!(member_list, vec!["corelib", "applib"]);
}

#[test]
fn refusing_to_overwrite_non_empty_dir_without_force() {
    let temp = tempdir().expect("temp dir");
    let workspace_dir = temp.path().join("demo");
    fs::create_dir_all(&workspace_dir).unwrap();
    fs::write(workspace_dir.join("some_file.txt"), "hello").unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("wg")
        .args(["new", workspace_dir.to_str().unwrap(), "--lib", "corelib"])
        .assert()
        .failure()
        .stderr(contains("not empty"));
}
