// src/scan.rs
// Handles the scan command for codemarks

use anyhow::Result;
use ignore::{WalkBuilder, overrides::OverrideBuilder};
use regex::Regex;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::{Codemark, load_global_config, load_global_projects, save_global_projects};

pub fn scan_directory(
    directory: &Path,
    ignore_patterns: &[String],
    ephemeral: bool,
) -> Result<usize> {
    let config = load_global_config(ephemeral);
    let mut projects_db = load_global_projects(ephemeral);
    // Use the original pattern for matching only
    let codemark_regex = Regex::new(&config.annotation_pattern)?;
    let project_name = directory
        .canonicalize()?
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string();
    let canonical_dir = directory.canonicalize()?;
    if let Some(existing_codemarks) = projects_db.projects.get_mut(&project_name) {
        for codemark in existing_codemarks.iter_mut() {
            codemark.resolved = true;
        }
    }
    let mut current_codemarks = Vec::new();

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
        let Ok(entry) = result else { continue };
        let file_path = entry.path();
        if entry.file_type().is_some_and(|ft| ft.is_file()) {
            if let Ok(file) = fs::File::open(file_path) {
                let reader = BufReader::new(file);
                for (line_number, line) in reader.lines().enumerate() {
                    if let Ok(line_content) = line {
                        // Use the pattern only to match, but always store the entire line
                        if codemark_regex.is_match(&line_content) {
                            let description = line_content.clone();
                            let relative_path =
                                if let Ok(stripped) = file_path.strip_prefix(&canonical_dir) {
                                    stripped.to_string_lossy().to_string()
                                } else {
                                    file_path.to_string_lossy().to_string()
                                };
                            let codemark = Codemark {
                                file: relative_path,
                                line_number: line_number + 1,
                                description,
                                resolved: false,
                            };
                            current_codemarks.push(codemark);
                        }
                    }
                }
            }
        }
    }
    if let Some(existing_codemarks) = projects_db.projects.get_mut(&project_name) {
        for current_codemark in current_codemarks {
            let mut found = false;
            for existing_codemark in existing_codemarks.iter_mut() {
                if existing_codemark.file == current_codemark.file
                    && existing_codemark.description == current_codemark.description
                {
                    existing_codemark.resolved = false;
                    existing_codemark.line_number = current_codemark.line_number;
                    found = true;
                    break;
                }
            }
            if !found {
                existing_codemarks.push(current_codemark);
            }
        }
    } else {
        projects_db
            .projects
            .insert(project_name.clone(), current_codemarks);
    }
    let total_count = projects_db
        .projects
        .values()
        .flat_map(|codemarks| codemarks.iter())
        .filter(|codemark| !codemark.resolved)
        .count();
    save_global_projects(&projects_db, ephemeral)?;
    Ok(total_count)
}

#[cfg(test)]
mod tests;
