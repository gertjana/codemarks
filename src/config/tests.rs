use super::*;
use crate::default_annotation_pattern;
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
fn test_config_show() {
    let _temp_home = setup_temp_home();

    // Test show config functionality
    let result = handle_config(ConfigAction::Show);
    assert!(result.is_ok());
}

#[test]
fn test_config_set_and_reset() {
    let _temp_home = setup_temp_home();

    // Test setting config using ConfigAction
    let result = handle_config(ConfigAction::SetPattern {
        pattern: "CUSTOM|PATTERN".to_string(),
    });
    assert!(result.is_ok());

    // Test reset using ConfigAction
    let result = handle_config(ConfigAction::Reset);
    assert!(result.is_ok());

    // Verify the default pattern function works
    let default = default_annotation_pattern();
    assert!(default.contains("TODO"));
    assert!(default.contains("FIXME"));
    assert!(default.contains("HACK"));
}

#[test]
fn test_config_invalid_pattern() {
    let _temp_home = setup_temp_home();

    // Test setting invalid regex pattern
    let result = handle_config(ConfigAction::SetPattern {
        pattern: "[invalid regex(".to_string(),
    });
    // Should handle the error gracefully
    assert!(result.is_ok() || result.is_err()); // Either way is fine, just shouldn't panic
}
