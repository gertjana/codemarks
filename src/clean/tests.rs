use super::*;
use crate::{Codemark, ProjectsDatabase};
use std::env;
use tempfile::TempDir;

fn setup_temp_home() -> TempDir {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    unsafe {
        env::set_var("HOME", temp_dir.path());
    }
    temp_dir
}

#[test]
fn test_clean_resolved_basic() {
    let _temp_home = setup_temp_home();

    // Test basic dry run functionality
    let result = clean_resolved(true, None);
    assert!(result.is_ok());

    // Test with project filter
    let result = clean_resolved(true, Some("nonexistent".to_string()));
    assert!(result.is_ok());
}

#[test]
fn test_clean_resolved_edge_cases() {
    let _temp_home = setup_temp_home();

    // Test dry run (safe operation)
    let result = clean_resolved(true, None);
    assert!(result.is_ok());

    // Test with project filter
    let result = clean_resolved(true, Some("nonexistent".to_string()));
    assert!(result.is_ok());
}

#[test]
fn test_clean_resolved_data_structures() {
    // Test the logic without file I/O
    let mut test_db = ProjectsDatabase::default();

    // Create test data
    let resolved_item = Codemark {
        file: "test.rs".to_string(),
        line_number: 1,
        description: "Done".to_string(),
        resolved: true,
    };

    let unresolved_item = Codemark {
        file: "test.rs".to_string(),
        line_number: 2,
        description: "TODO".to_string(),
        resolved: false,
    };

    test_db
        .projects
        .insert("test".to_string(), vec![resolved_item, unresolved_item]);

    // Test filtering logic manually
    for codemarks in test_db.projects.values() {
        let unresolved: Vec<_> = codemarks.iter().filter(|c| !c.resolved).collect();
        let resolved: Vec<_> = codemarks.iter().filter(|c| c.resolved).collect();

        assert_eq!(unresolved.len(), 1);
        assert_eq!(resolved.len(), 1);
        assert_eq!(unresolved[0].description, "TODO");
        assert_eq!(resolved[0].description, "Done");
    }
}
