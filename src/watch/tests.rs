use super::*;
use std::env;
use tempfile::tempdir;

fn setup_test_env() {
    // Clear any existing config
    unsafe {
        env::set_var("CODEMARKS_ANNOTATION_PATTERNS", "");
        env::set_var("CODEMARKS_IGNORE_PATTERNS", "");
    }
}

#[test]
fn test_scan_file_with_annotations() {
    setup_test_env();
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("test.rs");
    fs::write(
        &test_file,
        "// TODO: Fix this\nfn main() {}\n// FIXME: Another issue",
    )
    .unwrap();

    let pattern = Regex::new(r"(?i)(?://|#|<!--)\s*(?:TODO|FIXME|HACK|NOTE|BUG|OPTIMIZE|REVIEW)(?:\([^)]*\))?\s*:?\s*(.*)").unwrap();
    let result = scan_file(&test_file, &pattern).unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].description, "Fix this");
    assert_eq!(result[1].description, "Another issue");
}

#[test]
fn test_scan_file_without_annotations() {
    setup_test_env();
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("test.rs");
    fs::write(
        &test_file,
        "fn main() {\n    println!(\"Hello world!\");\n}",
    )
    .unwrap();

    let pattern = Regex::new(r"(?i)(?://|#|<!--)\s*(?:TODO|FIXME|HACK|NOTE|BUG|OPTIMIZE|REVIEW)(?:\([^)]*\))?\s*:?\s*(.*)").unwrap();
    let result = scan_file(&test_file, &pattern).unwrap();

    assert_eq!(result.len(), 0);
}

#[test]
fn test_should_ignore_file_with_patterns() {
    setup_test_env();
    let file_path = Path::new("/path/to/test.rs");
    let ignore_patterns = vec!["test.rs".to_string()];

    assert!(should_ignore_file(file_path, &ignore_patterns));
}

#[test]
fn test_should_ignore_file_binary_extensions() {
    setup_test_env();
    let file_path = Path::new("/path/to/image.jpg");
    let ignore_patterns = vec![];

    assert!(should_ignore_file(file_path, &ignore_patterns));
}

#[test]
fn test_should_not_ignore_source_file() {
    setup_test_env();
    let file_path = Path::new("/path/to/source.rs");
    let ignore_patterns = vec![];

    assert!(!should_ignore_file(file_path, &ignore_patterns));
}

#[test]
fn test_process_changed_file_ignored() {
    setup_test_env();
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("ignored.txt");
    fs::write(&test_file, "// TODO: This should be ignored").unwrap();

    let ignore_patterns = vec!["ignored.txt".to_string()];
    let pattern = Regex::new(r"(?i)(?://|#|<!--)\s*(?:TODO|FIXME|HACK|NOTE|BUG|OPTIMIZE|REVIEW)(?:\([^)]*\))?\s*:?\s*(.*)").unwrap();

    let result =
        process_changed_file(&test_file, &ignore_patterns, &pattern, "test_project").unwrap();
    assert_eq!(result, 0);
}

#[test]
fn test_process_changed_file_nonexistent() {
    setup_test_env();
    let nonexistent_file = Path::new("/nonexistent/file.rs");
    let ignore_patterns = vec![];
    let pattern = Regex::new(r"(?i)(?://|#|<!--)\s*(?:TODO|FIXME|HACK|NOTE|BUG|OPTIMIZE|REVIEW)(?:\([^)]*\))?\s*:?\s*(.*)").unwrap();

    let result =
        process_changed_file(nonexistent_file, &ignore_patterns, &pattern, "test_project").unwrap();
    assert_eq!(result, 0);
}

#[test]
fn test_process_changed_file_with_annotations() {
    let _temp_home = setup_temp_home();
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("test.rs");
    fs::write(
        &test_file,
        "// TODO: Important task\nlet x = 5;\n// FIXME: Bug here",
    )
    .unwrap();

    let ignore_patterns = vec![];
    let pattern = Regex::new(r"(?i)(?://|#|<!--)\s*(?:TODO|FIXME|HACK|NOTE|BUG|OPTIMIZE|REVIEW)(?:\([^)]*\))?\s*:?\s*(.*)").unwrap();

    let result =
        process_changed_file(&test_file, &ignore_patterns, &pattern, "test_project").unwrap();
    assert_eq!(result, 2); // Should find 2 annotations
}

#[test]
fn test_process_changed_file_empty_file() {
    let _temp_home = setup_temp_home();
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("empty.rs");
    fs::write(&test_file, "").unwrap();

    let ignore_patterns = vec![];
    let pattern = Regex::new(r"(?i)(?://|#|<!--)\s*(?:TODO|FIXME|HACK|NOTE|BUG|OPTIMIZE|REVIEW)(?:\([^)]*\))?\s*:?\s*(.*)").unwrap();

    let result =
        process_changed_file(&test_file, &ignore_patterns, &pattern, "test_project").unwrap();
    assert_eq!(result, 0);
}

#[test]
fn test_process_changed_file_binary_file() {
    setup_test_env();
    let temp_dir = tempdir().unwrap();
    let binary_file = temp_dir.path().join("test.bin");
    // Write some binary data
    fs::write(&binary_file, b"\x00\x01\x02\x03\xFF").unwrap();

    let ignore_patterns = vec![];
    let pattern = Regex::new(r"(?i)(?://|#|<!--)\s*(?:TODO|FIXME|HACK|NOTE|BUG|OPTIMIZE|REVIEW)(?:\([^)]*\))?\s*:?\s*(.*)").unwrap();

    let result =
        process_changed_file(&binary_file, &ignore_patterns, &pattern, "test_project").unwrap();
    assert_eq!(result, 0); // Binary files should return 0
}

#[test]
fn test_scan_file_invalid_utf8() {
    setup_test_env();
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("invalid.txt");
    // Write invalid UTF-8 bytes
    fs::write(&test_file, b"\xFF\xFE// TODO: This has invalid UTF-8").unwrap();

    let pattern = Regex::new(r"(?i)(?://|#|<!--)\s*(?:TODO|FIXME|HACK|NOTE|BUG|OPTIMIZE|REVIEW)(?:\([^)]*\))?\s*:?\s*(.*)").unwrap();

    // This should handle the error gracefully
    let result = scan_file(&test_file, &pattern);
    assert!(result.is_err());
}

#[test]
fn test_scan_file_different_annotation_types() {
    setup_test_env();
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("multi.rs");
    fs::write(
        &test_file,
        "// TODO: Task 1\n# FIXME: Bug in shell script\n<!-- HACK: Quick fix -->\n* NOTE: Important note\nlet x = 5;",
    ).unwrap();

    let pattern = Regex::new(r"(?i)(?://|#|<!--|\*)\s*(?:TODO|FIXME|HACK|NOTE|BUG|OPTIMIZE|REVIEW)(?:\([^)]*\))?\s*:?\s*(.*)").unwrap();
    let result = scan_file(&test_file, &pattern).unwrap();

    assert_eq!(result.len(), 4);
    assert_eq!(result[0].description, "Task 1");
    assert_eq!(result[1].description, "Bug in shell script");
    assert_eq!(result[2].description, "Quick fix -->");
    assert_eq!(result[3].description, "Important note");
}

#[test]
fn test_should_ignore_file_multiple_patterns() {
    setup_test_env();
    let file_path = Path::new("/path/to/build/output.js");
    let ignore_patterns = vec![
        "*.tmp".to_string(),
        "build/".to_string(),
        "node_modules/".to_string(),
    ];

    assert!(should_ignore_file(file_path, &ignore_patterns));
}

#[test]
fn test_should_ignore_file_no_match() {
    setup_test_env();
    let file_path = Path::new("/src/main.rs");
    let ignore_patterns = vec![
        "*.tmp".to_string(),
        "build/".to_string(),
        "node_modules/".to_string(),
    ];

    assert!(!should_ignore_file(file_path, &ignore_patterns));
}

#[test]
fn test_should_ignore_file_all_binary_extensions() {
    setup_test_env();
    let binary_extensions = vec![
        "test.jpg",
        "test.png",
        "test.gif",
        "test.pdf",
        "test.zip",
        "test.exe",
        "test.dll",
        "test.mp3",
        "test.mp4",
        "test.lock",
    ];

    for ext in binary_extensions {
        let file_path = Path::new(ext);
        assert!(should_ignore_file(file_path, &[]), "Should ignore {ext}");
    }
}

#[test]
fn test_should_not_ignore_source_extensions() {
    setup_test_env();
    let source_extensions = vec![
        "main.rs",
        "app.js",
        "index.html",
        "style.css",
        "script.py",
        "config.toml",
        "readme.md",
        "Dockerfile",
    ];

    for ext in source_extensions {
        let file_path = Path::new(ext);
        assert!(
            !should_ignore_file(file_path, &[]),
            "Should not ignore {ext}"
        );
    }
}

#[test]
fn test_scan_file_line_numbers_correct() {
    setup_test_env();
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("lines.rs");
    fs::write(
        &test_file,
        "fn main() {\n    println!(\"Hello\");\n    // TODO: Line 3 task\n    let x = 5;\n    // FIXME: Line 5 bug\n}",
    ).unwrap();

    let pattern = Regex::new(r"(?i)(?://|#|<!--)\s*(?:TODO|FIXME|HACK|NOTE|BUG|OPTIMIZE|REVIEW)(?:\([^)]*\))?\s*:?\s*(.*)").unwrap();
    let result = scan_file(&test_file, &pattern).unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].line_number, 3);
    assert_eq!(result[0].description, "Line 3 task");
    assert_eq!(result[1].line_number, 5);
    assert_eq!(result[1].description, "Line 5 bug");
}

#[test]
fn test_scan_file_with_complex_regex() {
    setup_test_env();
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("complex.rs");
    fs::write(
        &test_file,
        "// TODO(john): Assigned task\n// FIXME: Simple fix\n// HACK(urgent): Quick solution\n// NOTE: Just a note",
    ).unwrap();

    // More complex regex that captures assignee in parentheses
    let pattern = Regex::new(r"(?i)(?://|#|<!--)\s*(?:TODO|FIXME|HACK|NOTE|BUG|OPTIMIZE|REVIEW)(?:\([^)]*\))?\s*:?\s*(.*)").unwrap();
    let result = scan_file(&test_file, &pattern).unwrap();

    assert_eq!(result.len(), 4);
    assert_eq!(result[0].description, "Assigned task");
    assert_eq!(result[1].description, "Simple fix");
    assert_eq!(result[2].description, "Quick solution");
    assert_eq!(result[3].description, "Just a note");
}

fn setup_temp_home() -> tempfile::TempDir {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
    unsafe {
        std::env::set_var("HOME", temp_dir.path());
    }
    temp_dir
}
