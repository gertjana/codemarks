use crate::{
    Codemark, detect_project_name, load_global_config, load_global_projects, save_global_projects,
};
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
        if let Some(captures) = annotation_pattern.captures(line)
            && let Some(description) = captures.get(1)
        {
            let codemark = Codemark {
                file: file_path.to_string_lossy().to_string(),
                line_number: line_number + 1,
                description: description.as_str().trim().to_string(),
                resolved: false,
            };
            codemarks.push(codemark);
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
    ephemeral: bool,
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
                        if !ephemeral {
                            let mut projects_db = load_global_projects(false);
                            if let Some(project_codemarks) =
                                projects_db.projects.get_mut(project_name)
                            {
                                let old_count = project_codemarks.len();
                                project_codemarks
                                    .retain(|cm| cm.file != file_path.to_string_lossy());
                                let new_count = project_codemarks.len();
                                if old_count != new_count {
                                    save_global_projects(&projects_db, false)?;
                                    println!("  Removed {} old annotations", old_count - new_count);
                                }
                            }
                        }
                        Ok(0)
                    } else {
                        if !ephemeral {
                            let mut projects_db = load_global_projects(false);

                            // Remove old codemarks for this file
                            if let Some(project_codemarks) =
                                projects_db.projects.get_mut(project_name)
                            {
                                project_codemarks
                                    .retain(|cm| cm.file != file_path.to_string_lossy());
                            } else {
                                projects_db
                                    .projects
                                    .insert(project_name.to_string(), Vec::new());
                            }

                            // Add new codemarks
                            if let Some(project_codemarks) =
                                projects_db.projects.get_mut(project_name)
                            {
                                project_codemarks.extend(codemarks.clone());
                            }

                            save_global_projects(&projects_db, false)?;
                        }

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
    ephemeral: bool,
) -> Result<()> {
    let config = load_global_config(ephemeral);
    let annotation_pattern = Regex::new(&config.annotation_pattern)
        .map_err(|e| anyhow::anyhow!("Invalid regex pattern: {e}"))?;

    // Intelligently detect the project name from configuration files
    let project_name = detect_project_name(directory);

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
                                    if let Some(last_time) = recent_events.get(&path)
                                        && now.duration_since(*last_time) < debounce_duration
                                    {
                                        continue; // Skip this event due to debouncing
                                    }
                                    recent_events.insert(path.clone(), now);

                                    // Process the file
                                    match process_changed_file(
                                        &path,
                                        ignore_patterns,
                                        &annotation_pattern,
                                        &project_name,
                                        ephemeral,
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
mod tests;
