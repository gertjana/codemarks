use super::*;
use std::path::Path;
use tempfile::TempDir;

/// Helper function to set up a temporary directory for testing
fn setup_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

#[test]
fn test_detect_project_name_rust() {
    let temp_dir = setup_temp_dir();
    let cargo_toml = temp_dir.path().join("Cargo.toml");
    std::fs::write(
        &cargo_toml,
        r#"[package]
name = "my-rust-project"
version = "0.1.0""#,
    )
    .unwrap();

    let project_name = detect_project_name(temp_dir.path());
    assert_eq!(project_name, "my-rust-project");
}

#[test]
fn test_detect_project_name_nodejs() {
    let temp_dir = setup_temp_dir();
    let package_json = temp_dir.path().join("package.json");
    std::fs::write(
        &package_json,
        r#"{"name": "my-node-project", "version": "1.0.0"}"#,
    )
    .unwrap();

    let project_name = detect_project_name(temp_dir.path());
    assert_eq!(project_name, "my-node-project");
}

#[test]
fn test_detect_project_name_go() {
    let temp_dir = setup_temp_dir();
    let go_mod = temp_dir.path().join("go.mod");
    std::fs::write(&go_mod, "module github.com/user/my-go-project\n\ngo 1.21").unwrap();

    let project_name = detect_project_name(temp_dir.path());
    assert_eq!(project_name, "my-go-project");
}

#[test]
fn test_detect_project_name_fallback() {
    let temp_dir = setup_temp_dir();
    // No config files, should use directory name

    let project_name = detect_project_name(temp_dir.path());
    // The temp directory name will be something like .tmpXXXXXX,
    // so we just verify it's not empty and not "unknown"
    assert!(!project_name.is_empty());
    assert_ne!(project_name, "unknown");
}

#[test]
fn test_detect_project_name_invalid_directory() {
    let non_existent_path = Path::new("/this/path/does/not/exist");
    let project_name = detect_project_name(non_existent_path);
    assert_eq!(project_name, "exist");
}
