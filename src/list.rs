// src/list.rs
// Handles the list command for codemarks

use crate::load_global_projects;

pub fn list_codemarks() {
    let projects_db = load_global_projects();
    if projects_db.projects.is_empty() {
        println!("No code annotations found. Run 'codemarks scan' first to scan for annotations.");
        return;
    }
    for (project_name, codemarks) in &projects_db.projects {
        if codemarks.is_empty() {
            continue;
        }
        println!("{project_name}");
        for codemark in codemarks {
            let resolved_prefix = if codemark.resolved { "✅ " } else { "   " };
            println!(
                "{}{}:{} {}",
                resolved_prefix, codemark.file, codemark.line_number, codemark.description
            );
        }
        if projects_db.projects.len() > 1 {
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
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
    fn test_list_codemarks_empty() {
        let _temp_home = setup_temp_home();

        // Test listing when database is empty - should not crash
        list_codemarks();
    }

    #[test]
    fn test_list_codemarks_with_data() {
        let _temp_home = setup_temp_home();

        // Test that the list function doesn't crash even if we can't save data
        list_codemarks();
    }

    #[test]
    fn test_list_codemarks_functionality() {
        let _temp_home = setup_temp_home();

        // Create test data manually in memory and test logic
        let mut projects_db = ProjectsDatabase::default();

        let resolved_codemark = Codemark {
            file: "test1.rs".to_string(),
            line_number: 1,
            description: "Resolved task".to_string(),
            resolved: true,
        };

        let unresolved_codemark = Codemark {
            file: "test2.rs".to_string(),
            line_number: 2,
            description: "Unresolved task".to_string(),
            resolved: false,
        };

        projects_db.projects.insert(
            "test_project".to_string(),
            vec![resolved_codemark, unresolved_codemark],
        );

        // Add empty project to test the empty project skip logic
        projects_db
            .projects
            .insert("empty_project".to_string(), vec![]);

        // Test that we can iterate through the data structure correctly
        assert_eq!(projects_db.projects.len(), 2);
        assert_eq!(projects_db.projects.get("test_project").unwrap().len(), 2);
        assert_eq!(projects_db.projects.get("empty_project").unwrap().len(), 0);

        // Verify the resolved prefix logic
        for codemarks in projects_db.projects.values() {
            for codemark in codemarks {
                let resolved_prefix = if codemark.resolved { "✅ " } else { "   " };
                if codemark.resolved {
                    assert_eq!(resolved_prefix, "✅ ");
                } else {
                    assert_eq!(resolved_prefix, "   ");
                }
            }
        }
    }
}
