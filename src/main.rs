
use clap::{Parser, Subcommand};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Todo {
    file: String,
    line_number: usize,
    description: String,
    #[serde(default)]
    resolved: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct TodoConfig {
    #[serde(default = "default_todo_pattern")]
    todo_pattern: String,
}

impl Default for TodoConfig {
    fn default() -> Self {
        Self {
            todo_pattern: default_todo_pattern(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectsDatabase {
    projects: HashMap<String, Vec<Todo>>,
}

impl Default for ProjectsDatabase {
    fn default() -> Self {
        Self {
            projects: HashMap::new(),
        }
    }
}

fn default_todo_pattern() -> String {
    r"(?i)(?://|#|<!--|\*)\s*(?:TODO|TO\s+DO)\s*:?\s*(.*)$".to_string()
}

fn get_global_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))?;
    let config_dir = Path::new(&home).join(".isju");
    fs::create_dir_all(&config_dir)?;
    Ok(config_dir.join("config.json"))
}

fn get_global_projects_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))?;
    let config_dir = Path::new(&home).join(".isju");
    fs::create_dir_all(&config_dir)?;
    Ok(config_dir.join("projects.json"))
}

fn load_global_config() -> TodoConfig {
    match get_global_config_path() {
        Ok(config_path) => {
            if config_path.exists() {
                if let Ok(content) = fs::read_to_string(&config_path) {
                    if let Ok(config) = serde_json::from_str::<TodoConfig>(&content) {
                        return config;
                    }
                }
            }
        }
        Err(_) => {}
    }
    TodoConfig::default()
}

fn save_global_config(config: &TodoConfig) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_global_config_path()?;
    let json_content = serde_json::to_string_pretty(config)?;
    fs::write(config_path, json_content)?;
    Ok(())
}

fn load_global_projects() -> ProjectsDatabase {
    match get_global_projects_path() {
        Ok(projects_path) => {
            if projects_path.exists() {
                if let Ok(content) = fs::read_to_string(&projects_path) {
                    if let Ok(projects_db) = serde_json::from_str::<ProjectsDatabase>(&content) {
                        return projects_db;
                    }
                }
            }
        }
        Err(_) => {}
    }
    ProjectsDatabase::default()
}

fn save_global_projects(projects_db: &ProjectsDatabase) -> Result<(), Box<dyn std::error::Error>> {
    let projects_path = get_global_projects_path()?;
    let json_content = serde_json::to_string_pretty(projects_db)?;
    fs::write(projects_path, json_content)?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct TodoDatabase {
    #[serde(default)]
    config: TodoConfig,
    projects: HashMap<String, Vec<Todo>>,
}

/// Simple CLI tool
#[derive(Parser)]
#[command(name = "isju")]
#[command(about = "A simple CLI tool for managing TODOs", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show the version
    Version,
    /// Scan a directory for TODOs
    Scan {
        /// Directory to scan
        #[arg(short, long, default_value = ".")]
        directory: Option<PathBuf>,
    },
    /// List all TODOs from the global projects database
    List,
    /// Manage global configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current global configuration
    Show,
    /// Set the global TODO pattern
    SetPattern {
        /// The regex pattern to use for matching TODOs
        pattern: String,
    },
    /// Reset to default pattern
    Reset,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Version => {
            // Hardcoded version string
            println!("isju version 1.0.0");
        }
        Commands::Scan { directory } => {
            let dir = directory.as_ref().map(|p| p.as_path()).unwrap_or(Path::new("."));
            match scan_directory(dir) {
                Ok(count) => println!("Found {} TODOs and saved to global projects database", count),
                Err(e) => eprintln!("Error scanning directory: {}", e),
            }
        }
        Commands::List => {
            match list_todos() {
                Ok(()) => {},
                Err(e) => eprintln!("Error listing TODOs: {}", e),
            }
        }
        Commands::Config { action } => {
            match handle_config(action) {
                Ok(()) => {},
                Err(e) => eprintln!("Error managing config: {}", e),
            }
        }
    }
}

fn scan_directory(directory: &Path) -> Result<usize, Box<dyn std::error::Error>> {
    let config = load_global_config();
    let mut projects_db = load_global_projects();
    
    let todo_regex = Regex::new(&config.todo_pattern)?;
    
    // Get project name from directory
    let project_name = directory
        .canonicalize()?
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Get the canonical directory path for making relative paths
    let canonical_dir = directory.canonicalize()?;
    
    // Mark all existing TODOs for this project as potentially resolved
    if let Some(existing_todos) = projects_db.projects.get_mut(&project_name) {
        for todo in existing_todos.iter_mut() {
            todo.resolved = true;
        }
    }

    // Collect current TODOs
    let mut current_todos = Vec::new();

    // Walk through all files in directory
    for entry in WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let file_path = entry.path();
        
        // Skip target directory, hidden files, and binary files
        if file_path.to_string_lossy().contains("/target/") || 
           file_path.to_string_lossy().contains("/.") ||
           file_path.file_name().and_then(|name| name.to_str()).map_or(false, |name| name.starts_with('.')) {
            continue;
        }
        
        // Skip binary files and common non-text files
        if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
            match extension.to_lowercase().as_str() {
                "exe" | "bin" | "dll" | "so" | "dylib" | "img" | "png" | "jpg" | "jpeg" | "gif" | "ico" | "zip" | "tar" | "gz" | "o" | "a" => continue,
                _ => {}
            }
        }

        // Skip if no extension and file name looks binary
        if file_path.extension().is_none() {
            if let Some(file_name) = file_path.file_name().and_then(|name| name.to_str()) {
                // Skip files that look like binaries (no extension and contain project name)
                if file_name == project_name || file_name.contains(&project_name) {
                    continue;
                }
            }
        }

        // Read file and search for TODOs
        if let Ok(file) = fs::File::open(file_path) {
            let reader = BufReader::new(file);
            for (line_number, line) in reader.lines().enumerate() {
                if let Ok(line_content) = line {
                    // Skip if line contains the regex pattern itself (avoid self-reference)
                    if line_content.contains("TODO\\s*:?\\s*(.*)") {
                        continue;
                    }
                    
                    if let Some(captures) = todo_regex.captures(&line_content) {
                        let description = captures.get(1)
                            .map(|m| m.as_str().trim().to_string())
                            .unwrap_or_else(|| "".to_string());
                        
                        // Skip if description looks like it's part of the regex or binary content
                        if description.contains("\\s*") || description.len() > 200 {
                            continue;
                        }
                        
                        // Make file path relative to the project directory
                        let relative_path = if let Ok(stripped) = file_path.strip_prefix(&canonical_dir) {
                            stripped.to_string_lossy().to_string()
                        } else {
                            file_path.to_string_lossy().to_string()
                        };
                        
                        let todo = Todo {
                            file: relative_path,
                            line_number: line_number + 1, // 1-indexed
                            description,
                            resolved: false,
                        };
                        
                        current_todos.push(todo);
                    }
                }
            }
        }
    }

    // Update or add TODOs for this project
    if let Some(existing_todos) = projects_db.projects.get_mut(&project_name) {
        // Check each current TODO against existing ones
        for current_todo in current_todos {
            let mut found = false;
            for existing_todo in existing_todos.iter_mut() {
                if existing_todo.file == current_todo.file &&
                   existing_todo.description == current_todo.description {
                    // TODO still exists, mark as not resolved and update line number
                    existing_todo.resolved = false;
                    existing_todo.line_number = current_todo.line_number;
                    found = true;
                    break;
                }
            }
            if !found {
                // New TODO, add it
                existing_todos.push(current_todo);
            }
        }
    } else {
        // No existing TODOs for this project, add all current ones
        projects_db.projects.insert(project_name.clone(), current_todos);
    }

    // Calculate total count of active (non-resolved) TODOs
    let total_count = projects_db.projects.values()
        .flat_map(|todos| todos.iter())
        .filter(|todo| !todo.resolved)
        .count();

    // Save to global projects file
    save_global_projects(&projects_db)?;

    Ok(total_count)
}

fn list_todos() -> Result<(), Box<dyn std::error::Error>> {
    let projects_db = load_global_projects();

    if projects_db.projects.is_empty() {
        println!("No TODOs found. Run 'isju scan' first to scan for TODOs.");
        return Ok(());
    }

    // Display TODOs grouped by project
    for (project_name, todos) in &projects_db.projects {
        if todos.is_empty() {
            continue;
        }
        
        println!("{}", project_name);
        
        for todo in todos {
            let resolved_prefix = if todo.resolved { "✅ " } else { "   " };
            println!("{}{}:{} {}", resolved_prefix, todo.file, todo.line_number, todo.description);
        }
        
        // Add blank line between projects if there are multiple
        if projects_db.projects.len() > 1 {
            println!();
        }
    }

    Ok(())
}

fn handle_config(action: ConfigAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ConfigAction::Show => {
            let config = load_global_config();
            println!("Global TODO pattern:");
            println!("{}", config.todo_pattern);
            
            if let Ok(config_path) = get_global_config_path() {
                println!("\nConfig file location: {}", config_path.display());
            }
            
            if let Ok(projects_path) = get_global_projects_path() {
                println!("Projects file location: {}", projects_path.display());
            }
        }
        ConfigAction::SetPattern { pattern } => {
            // Validate the regex pattern
            match Regex::new(&pattern) {
                Ok(_) => {
                    let config = TodoConfig {
                        todo_pattern: pattern.clone(),
                    };
                    save_global_config(&config)?;
                    println!("Global TODO pattern updated to: {}", pattern);
                }
                Err(e) => {
                    eprintln!("Invalid regex pattern: {}", e);
                    return Err(e.into());
                }
            }
        }
        ConfigAction::Reset => {
            let config = TodoConfig::default();
            save_global_config(&config)?;
            println!("Global TODO pattern reset to default: {}", config.todo_pattern);
        }
    }
    Ok(())
}
