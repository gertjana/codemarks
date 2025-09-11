use crate::{ProjectsDatabase, load_global_projects, save_global_projects};
use anyhow::Result;
use std::collections::HashMap;

pub fn clean_resolved(dry_run: bool, project_filter: Option<String>) -> Result<()> {
    let projects_db = load_global_projects();
    let mut total_removed = 0;
    let mut projects_affected = 0;

    // Track what will be removed for reporting
    let mut removed_by_project: HashMap<String, usize> = HashMap::new();

    // Create a new database with only unresolved items
    let mut cleaned_db = ProjectsDatabase {
        projects: HashMap::new(),
    };

    for (project_name, codemarks) in &projects_db.projects {
        // Skip projects not matching the filter if one is specified
        if let Some(ref filter) = project_filter {
            if project_name != filter {
                // Keep this project as-is if it doesn't match the filter
                cleaned_db
                    .projects
                    .insert(project_name.clone(), codemarks.clone());
                continue;
            }
        }

        let original_count = codemarks.len();
        let unresolved_codemarks: Vec<_> = codemarks
            .iter()
            .filter(|codemark| !codemark.resolved)
            .cloned()
            .collect();

        let removed_count = original_count - unresolved_codemarks.len();

        if removed_count > 0 {
            total_removed += removed_count;
            projects_affected += 1;
            removed_by_project.insert(project_name.clone(), removed_count);

            if dry_run {
                println!(
                    "Would remove {removed_count} resolved annotations from project '{project_name}'"
                );
            }
        }

        // Only keep the project if it has unresolved items
        if !unresolved_codemarks.is_empty() {
            cleaned_db
                .projects
                .insert(project_name.clone(), unresolved_codemarks);
        } else if !dry_run && removed_count > 0 {
            // Project will be completely removed
            println!(
                "Removed project '{project_name}' (all {removed_count} annotations were resolved)"
            );
        } else if dry_run && removed_count == original_count {
            println!(
                "Would remove project '{project_name}' (all {removed_count} annotations are resolved)"
            );
        }
    }

    if dry_run {
        if total_removed == 0 {
            println!("No resolved annotations found to clean");
        } else {
            println!("\nDry run summary:");
            println!(
                "Would remove {total_removed} resolved annotations from {projects_affected} projects"
            );
            if let Some(filter) = project_filter {
                println!("Filter applied: only project '{filter}'");
            }
            println!("Use 'codemarks clean' (without --dry-run) to perform the actual cleanup");
        }
    } else if total_removed == 0 {
        println!("No resolved annotations found to clean");
    } else {
        // Save the cleaned database
        save_global_projects(&cleaned_db)?;
        println!(
            "Successfully removed {total_removed} resolved annotations from {projects_affected} projects"
        );

        // Show details of what was removed
        for (project, count) in removed_by_project {
            println!("  - {project}: {count} resolved annotations removed");
        }
    }

    Ok(())
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
        for (_project_name, codemarks) in &test_db.projects {
            let unresolved: Vec<_> = codemarks.iter().filter(|c| !c.resolved).collect();
            let resolved: Vec<_> = codemarks.iter().filter(|c| c.resolved).collect();

            assert_eq!(unresolved.len(), 1);
            assert_eq!(resolved.len(), 1);
            assert_eq!(unresolved[0].description, "TODO");
            assert_eq!(resolved[0].description, "Done");
        }
    }
}
