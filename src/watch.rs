use crate::{Codemark, load_global_config, load_global_projects, save_global_projects};
use anyhow::Result;
use ignore::WalkBuilder;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

/// Scans a single file for code annotations and returns found codemarks
fn scan_file(file_path: &Path, annotation_pattern: &Regex) -> Result<Vec<Codemark>> {
    let content = fs::read_to_string(file_path)?;
    let mut codemarks = Vec::new();

    for (line_number, line) in content.lines().enumerate() {
        if let Some(captures) = annotation_pattern.captures(line) {
            if let Some(description) = captures.get(1) {
                let codemark = Codemark {
                    file: file_path.to_string_lossy().to_string(),
                    line_number: line_number + 1,
                    description: description.as_str().trim().to_string(),
                    resolved: false,
                };
                codemarks.push(codemark);
            }
        }
    }

    Ok(codemarks)
}

/// Checks if a file should be ignored based on ignore patterns
fn should_ignore_file(file_path: &Path, ignore_patterns: &[String]) -> bool {
    let file_str = file_path.to_string_lossy();

    for pattern in ignore_patterns {
        if file_str.contains(pattern) {
            return true;
        }
    }

    // Skip common non-source file extensions
    if let Some(extension) = file_path.extension() {
        let ext = extension.to_string_lossy().to_lowercase();
        matches!(
            ext.as_str(),
            "jpg"
                | "jpeg"
                | "png"
                | "gif"
                | "bmp"
                | "ico"
                | "svg"
                | "pdf"
                | "doc"
                | "docx"
                | "xls"
                | "xlsx"
                | "ppt"
                | "pptx"
                | "zip"
                | "tar"
                | "gz"
                | "rar"
                | "7z"
                | "mp3"
                | "wav"
                | "mp4"
                | "avi"
                | "mov"
                | "exe"
                | "dll"
                | "so"
                | "dylib"
                | "lock"
                | "log"
        )
    } else {
        false
    }
}

/// Processes a changed file by scanning it for annotations
fn process_changed_file(
    file_path: &Path,
    ignore_patterns: &[String],
    annotation_pattern: &Regex,
    project_name: &str,
) -> Result<usize> {
    // Check if file should be ignored
    if should_ignore_file(file_path, ignore_patterns) {
        return Ok(0);
    }

    // Check if file exists (it might have been deleted)
    if !file_path.exists() {
        println!("File deleted: {}", file_path.display());
        return Ok(0);
    }

    // Check if it's a text file by trying to read it
    match fs::read_to_string(file_path) {
        Ok(_) => {
            // File is readable as text, proceed with scanning
            println!("Scanning changed file: {}", file_path.display());

            match scan_file(file_path, annotation_pattern) {
                Ok(codemarks) => {
                    if codemarks.is_empty() {
                        // No annotations found, but still need to clean up old ones
                        let mut projects_db = load_global_projects();
                        if let Some(project_codemarks) = projects_db.projects.get_mut(project_name)
                        {
                            let old_count = project_codemarks.len();
                            project_codemarks.retain(|cm| cm.file != file_path.to_string_lossy());
                            let new_count = project_codemarks.len();
                            if old_count != new_count {
                                save_global_projects(&projects_db)?;
                                println!("  Removed {} old annotations", old_count - new_count);
                            }
                        }
                        Ok(0)
                    } else {
                        let mut projects_db = load_global_projects();

                        // Remove old codemarks for this file
                        if let Some(project_codemarks) = projects_db.projects.get_mut(project_name)
                        {
                            project_codemarks.retain(|cm| cm.file != file_path.to_string_lossy());
                        } else {
                            projects_db
                                .projects
                                .insert(project_name.to_string(), Vec::new());
                        }

                        // Add new codemarks
                        if let Some(project_codemarks) = projects_db.projects.get_mut(project_name)
                        {
                            project_codemarks.extend(codemarks.clone());
                        }

                        save_global_projects(&projects_db)?;

                        println!("  Found {} annotations:", codemarks.len());
                        for codemark in &codemarks {
                            println!(
                                "    Line {}: {}",
                                codemark.line_number, codemark.description
                            );
                        }

                        Ok(codemarks.len())
                    }
                }
                Err(e) => {
                    eprintln!("  Error scanning file: {e}");
                    Ok(0)
                }
            }
        }
        Err(_) => {
            // File is not readable as text (binary file), skip it
            Ok(0)
        }
    }
}

/// Main watch function that monitors a directory for changes
pub fn watch_directory(
    directory: &Path,
    ignore_patterns: &[String],
    debounce_ms: Option<u64>,
) -> Result<()> {
    let config = load_global_config();
    let annotation_pattern = Regex::new(&config.annotation_pattern)
        .map_err(|e| anyhow::anyhow!("Invalid regex pattern: {e}"))?;

    // Use the directory name as project name
    let project_name = directory
        .file_name()
        .unwrap_or(directory.as_os_str())
        .to_string_lossy()
        .to_string();

    println!("Watching directory: {}", directory.display());
    println!("Project name: {project_name}");
    if !ignore_patterns.is_empty() {
        println!("Ignore patterns: {ignore_patterns:?}");
    }
    println!("Annotation pattern: {}", config.annotation_pattern);
    println!("Debounce: {}ms", debounce_ms.unwrap_or(500));
    println!("Press Ctrl+C to stop watching...\n");

    // Create a channel to receive file system events
    let (tx, rx) = channel();

    // Create a watcher
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Watch the directory recursively
    watcher.watch(directory, RecursiveMode::Recursive)?;

    // Track recent events to implement debouncing
    let mut recent_events: HashMap<PathBuf, Instant> = HashMap::new();
    let debounce_duration = Duration::from_millis(debounce_ms.unwrap_or(500));

    // Process events
    loop {
        match rx.recv() {
            Ok(event_result) => {
                match event_result {
                    Ok(Event { kind, paths, .. }) => {
                        // Only process write/create/remove events
                        match kind {
                            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                                for path in paths {
                                    // Skip if path doesn't exist or is a directory
                                    if path.is_dir() {
                                        continue;
                                    }

                                    // Check if we should ignore this path based on gitignore
                                    let walker = WalkBuilder::new(&path).max_depth(Some(0)).build();

                                    let mut should_process = false;
                                    for entry in walker.flatten() {
                                        if entry.path() == path {
                                            should_process = true;
                                            break;
                                        }
                                    }
                                    if !should_process {
                                        continue;
                                    }

                                    // Implement debouncing
                                    let now = Instant::now();
                                    if let Some(last_time) = recent_events.get(&path) {
                                        if now.duration_since(*last_time) < debounce_duration {
                                            continue; // Skip this event due to debouncing
                                        }
                                    }
                                    recent_events.insert(path.clone(), now);

                                    // Process the file
                                    match process_changed_file(
                                        &path,
                                        ignore_patterns,
                                        &annotation_pattern,
                                        &project_name,
                                    ) {
                                        Ok(count) => {
                                            if count > 0 {
                                                println!("Updated project database\n");
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "Error processing {}: {}\n",
                                                path.display(),
                                                e
                                            );
                                        }
                                    }
                                }
                            }
                            _ => {
                                // Ignore other event types (access, etc.)
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Watch error: {e:?}");
                    }
                }
            }
            Err(e) => {
                eprintln!("Channel error: {e:?}");
                break;
            }
        }

        // Clean up old entries from recent_events map
        let now = Instant::now();
        recent_events.retain(|_, &mut time| now.duration_since(time) < debounce_duration * 2);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
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
            process_changed_file(nonexistent_file, &ignore_patterns, &pattern, "test_project")
                .unwrap();
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
}
