// src/scan.rs
// Handles the scan command for codemarks

use ignore::{WalkBuilder, overrides::OverrideBuilder};
use regex::Regex;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::{Codemark, load_global_config, load_global_projects, save_global_projects};

pub fn scan_directory(
    directory: &Path,
    ignore_patterns: &[String],
) -> Result<usize, Box<dyn std::error::Error>> {
    let config = load_global_config();
    let mut projects_db = load_global_projects();
    // Use the original pattern for matching only
    let todo_regex = Regex::new(&config.annotation_pattern)?;
    let project_name = directory
        .canonicalize()?
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string();
    let canonical_dir = directory.canonicalize()?;
    if let Some(existing_todos) = projects_db.projects.get_mut(&project_name) {
        for todo in existing_todos.iter_mut() {
            todo.resolved = true;
        }
    }
    let mut current_todos = Vec::new();

    let mut builder = WalkBuilder::new(directory);

    // Add custom ignore patterns using overrides
    if !ignore_patterns.is_empty() {
        let mut override_builder = OverrideBuilder::new(directory);
        for pattern in ignore_patterns {
            // Add as negative override (ignore pattern)
            if let Err(e) = override_builder.add(&format!("!{pattern}")) {
                eprintln!("Warning: Invalid ignore pattern '{pattern}': {e}");
            }
        }
        if let Ok(overrides) = override_builder.build() {
            builder.overrides(overrides);
        }
    }

    for result in builder.build() {
        match result {
            Ok(entry) => {
                let file_path = entry.path();
                if entry.file_type().is_some_and(|ft| ft.is_file()) {
                    if let Ok(file) = fs::File::open(file_path) {
                        let reader = BufReader::new(file);
                        for (line_number, line) in reader.lines().enumerate() {
                            if let Ok(line_content) = line {
                                // Use the pattern only to match, but always store the entire line
                                if todo_regex.is_match(&line_content) {
                                    let description = line_content.clone();
                                    let relative_path = if let Ok(stripped) =
                                        file_path.strip_prefix(&canonical_dir)
                                    {
                                        stripped.to_string_lossy().to_string()
                                    } else {
                                        file_path.to_string_lossy().to_string()
                                    };
                                    let todo = Codemark {
                                        file: relative_path,
                                        line_number: line_number + 1,
                                        description,
                                        resolved: false,
                                    };
                                    current_todos.push(todo);
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => continue,
        }
    }
    if let Some(existing_todos) = projects_db.projects.get_mut(&project_name) {
        for current_todo in current_todos {
            let mut found = false;
            for existing_todo in existing_todos.iter_mut() {
                if existing_todo.file == current_todo.file
                    && existing_todo.description == current_todo.description
                {
                    existing_todo.resolved = false;
                    existing_todo.line_number = current_todo.line_number;
                    found = true;
                    break;
                }
            }
            if !found {
                existing_todos.push(current_todo);
            }
        }
    } else {
        projects_db
            .projects
            .insert(project_name.clone(), current_todos);
    }
    let total_count = projects_db
        .projects
        .values()
        .flat_map(|todos| todos.iter())
        .filter(|todo| !todo.resolved)
        .count();
    save_global_projects(&projects_db)?;
    Ok(total_count)
}
