use super::*;
use std::env;
use std::fs;
use tempfile::tempdir;

fn setup_test_env() {
    // Clear any existing config
    unsafe {
        env::set_var("CODEMARKS_ANNOTATION_PATTERNS", "");
        env::set_var("CODEMARKS_IGNORE_PATTERNS", "");
    }
}

#[test]
fn test_count_annotations_empty_directory() {
    setup_test_env();
    let temp_dir = tempdir().unwrap();
    let result = count_annotations(temp_dir.path(), None, &[]);
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_count_annotations_with_todos() {
    setup_test_env();
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("test.rs");
    fs::write(
        &test_file,
        "// TODO: Fix this\nfn main() {}\n// FIXME: Another issue",
    )
    .unwrap();

    let result = count_annotations(temp_dir.path(), None, &[]);
    assert_eq!(result.unwrap(), 2);
}

#[test]
fn test_count_annotations_with_custom_pattern() {
    setup_test_env();
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("test.rs");
    fs::write(
        &test_file,
        "// CUSTOM: Fix this\nfn main() {}\n// TODO: Another issue",
    )
    .unwrap();

    let result = count_annotations(temp_dir.path(), Some("CUSTOM".to_string()), &[]);
    assert_eq!(result.unwrap(), 1);
}

#[test]
fn test_count_annotations_invalid_regex() {
    setup_test_env();
    let temp_dir = tempdir().unwrap();
    let result = count_annotations(temp_dir.path(), Some("[".to_string()), &[]);
    assert!(result.is_err());
}

#[test]
fn test_count_annotations_with_ignore_patterns() {
    setup_test_env();
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("test.rs");
    let ignored_file = temp_dir.path().join("ignored.rs");
    fs::write(&test_file, "// TODO: Fix this").unwrap();
    fs::write(&ignored_file, "// TODO: Ignored todo").unwrap();

    let result = count_annotations(temp_dir.path(), None, &["ignored.rs".to_string()]);
    assert_eq!(result.unwrap(), 1);
}
