use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper function to create a command with a temporary home directory
fn cmd_with_temp_home() -> (Command, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let mut cmd = Command::cargo_bin("codemarks").expect("Failed to find binary");
    cmd.env("HOME", temp_dir.path());
    (cmd, temp_dir)
}

/// Helper function to create test files with annotations
fn create_test_files(dir: &std::path::Path) {
    let rust_file = dir.join("test.rs");
    fs::write(
        &rust_file,
        r#"// This is a test file
// TODO: Implement this function
fn main() {
    // FIXME: Add error handling
    println!("Hello, world!");
}

// HACK: Quick workaround
fn helper() {
    // Regular comment (should not be detected)
}
"#,
    )
    .expect("Failed to write test file");

    let js_file = dir.join("test.js");
    fs::write(
        &js_file,
        r#"// TODO: Add validation
function validate(data) {
    // FIXME: Implement proper validation
    return true;
}
"#,
    )
    .expect("Failed to write JS test file");
}

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("codemarks").expect("Failed to find binary");
    cmd.arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains("codemarks version"));
}

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("codemarks").expect("Failed to find binary");
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Codemarks helps you track code annotations",
        ));
}

#[test]
fn test_scan_command() {
    let (mut cmd, temp_home) = cmd_with_temp_home();
    let test_dir = TempDir::new().expect("Failed to create test directory");
    create_test_files(test_dir.path());

    cmd.arg("scan")
        .arg("--directory")
        .arg(test_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Found"))
        .stdout(predicate::str::contains("code annotations"));

    // Verify that the config directory was created
    let config_dir = temp_home.path().join(".codemarks");
    assert!(config_dir.exists());
    assert!(config_dir.join("config.json").exists());
    assert!(config_dir.join("projects.json").exists());
}

#[test]
fn test_scan_with_ignore_patterns() {
    let (mut cmd, _temp_home) = cmd_with_temp_home();
    let test_dir = TempDir::new().expect("Failed to create test directory");
    create_test_files(test_dir.path());

    cmd.arg("scan")
        .arg("--directory")
        .arg(test_dir.path())
        .arg("--ignore")
        .arg("*.js")
        .assert()
        .success()
        .stdout(predicate::str::contains("Found"));
}

#[test]
fn test_list_command_empty() {
    let (mut cmd, _temp_home) = cmd_with_temp_home();

    cmd.arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No code annotations found"));
}

#[test]
fn test_ci_command_with_annotations() {
    let (mut cmd, _temp_home) = cmd_with_temp_home();
    let test_dir = TempDir::new().expect("Failed to create test directory");
    create_test_files(test_dir.path());

    // CI command should return exit code 1 when annotations are found
    cmd.arg("ci")
        .arg("--directory")
        .arg(test_dir.path())
        .assert()
        .failure() // CI mode returns non-zero exit code when annotations found
        .stdout(predicate::str::contains("Found"))
        .stdout(predicate::str::contains("codemarks matching pattern"));
}

#[test]
fn test_ci_command_no_annotations() {
    let (mut cmd, _temp_home) = cmd_with_temp_home();
    let test_dir = TempDir::new().expect("Failed to create test directory");

    // Create a file with no annotations
    let clean_file = test_dir.path().join("clean.rs");
    fs::write(
        &clean_file,
        r#"// This is a clean file
fn main() {
    println!("No annotations here!");
}
"#,
    )
    .expect("Failed to write clean test file");

    // CI command should return exit code 0 when no annotations are found
    cmd.arg("ci")
        .arg("--directory")
        .arg(test_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No codemarks found matching pattern",
        ));
}

#[test]
fn test_ci_command_with_custom_pattern() {
    let (mut cmd, _temp_home) = cmd_with_temp_home();
    let test_dir = TempDir::new().expect("Failed to create test directory");

    let test_file = test_dir.path().join("custom.rs");
    fs::write(
        &test_file,
        r#"// TODO: This should be found
// FIXME: This should not be found with our custom pattern
fn main() {
    println!("Custom pattern test");
}
"#,
    )
    .expect("Failed to write custom test file");

    // Test with custom pattern that only matches TODO
    cmd.arg("ci")
        .arg("--directory")
        .arg(test_dir.path())
        .arg("--pattern")
        .arg("TODO")
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "Found 1 codemarks matching pattern",
        ));
}

#[test]
fn test_config_show_command() {
    let (mut cmd, _temp_home) = cmd_with_temp_home();

    cmd.arg("config")
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains("Global code annotation pattern"));
}

#[test]
fn test_config_set_pattern_command() {
    let (mut cmd, temp_home) = cmd_with_temp_home();
    let custom_pattern = "CUSTOM_ANNOTATION";

    cmd.arg("config")
        .arg("set-pattern")
        .arg(custom_pattern)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Global code annotation pattern updated",
        ));

    // Verify the pattern was saved
    let config_file = temp_home.path().join(".codemarks").join("config.json");
    let config_content = fs::read_to_string(config_file).expect("Failed to read config file");
    assert!(config_content.contains(custom_pattern));
}

#[test]
fn test_config_reset_command() {
    let (mut cmd, _temp_home) = cmd_with_temp_home();

    cmd.arg("config")
        .arg("reset")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Global code annotation pattern reset to default",
        ));
}

#[test]
fn test_watch_command_help() {
    let mut cmd = Command::cargo_bin("codemarks").expect("Failed to find binary");

    cmd.arg("watch")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Watch directory for changes"))
        .stdout(predicate::str::contains("--debounce"))
        .stdout(predicate::str::contains("--ignore"));
}

#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("codemarks").expect("Failed to find binary");

    cmd.arg("invalid_command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn test_scan_nonexistent_directory() {
    let (mut cmd, _temp_home) = cmd_with_temp_home();

    cmd.arg("scan")
        .arg("--directory")
        .arg("/nonexistent/directory")
        .assert()
        .success() // scan command doesn't exit with error, just prints to stderr
        .stderr(predicate::str::contains("Error scanning directory"));
}

#[test]
fn test_clean_command_help() {
    let (mut cmd, _temp_home) = cmd_with_temp_home();

    cmd.arg("clean")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Remove resolved annotations from the global database",
        ))
        .stdout(predicate::str::contains("--dry-run"))
        .stdout(predicate::str::contains("--project"));
}

#[test]
fn test_clean_command_empty_database() {
    let (mut cmd, _temp_home) = cmd_with_temp_home();

    cmd.arg("clean")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No resolved annotations found to clean",
        ));
}

#[test]
fn test_clean_command_dry_run() {
    let (mut cmd, temp_home) = cmd_with_temp_home();

    // First, scan some files to populate the database
    let test_dir = temp_home.path().join("test_project");
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");
    create_test_files(&test_dir);

    // Scan to populate database
    cmd.arg("scan")
        .arg("--directory")
        .arg(&test_dir)
        .assert()
        .success();

    // Test dry run when no resolved items exist
    let (mut cmd2, _) = cmd_with_temp_home();
    cmd2.env("HOME", temp_home.path());
    cmd2.arg("clean")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No resolved annotations found to clean",
        ));
}

#[test]
fn test_clean_command_with_project_filter() {
    let (mut cmd, _temp_home) = cmd_with_temp_home();

    // Test clean with specific project filter on empty database
    cmd.arg("clean")
        .arg("--project")
        .arg("nonexistent_project")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No resolved annotations found to clean",
        ));
}
