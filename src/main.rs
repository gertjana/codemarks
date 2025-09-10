use clap::{Parser, Subcommand};
// use regex::Regex;
mod ci;
mod config;
mod list;
mod scan;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Codemark {
    file: String,
    line_number: usize,
    description: String,
    #[serde(default)]
    resolved: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CodemarksConfig {
    #[serde(default = "default_annotation_pattern")]
    annotation_pattern: String,
}

impl Default for CodemarksConfig {
    fn default() -> Self {
        Self {
            annotation_pattern: default_annotation_pattern(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct ProjectsDatabase {
    projects: HashMap<String, Vec<Codemark>>,
}

fn default_annotation_pattern() -> String {
    r"(?i)(?://|#|<!--|\*)\s*(?:TODO|FIXME|HACK)\s*:?\s*(.*)$".to_string()
}

fn get_global_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home_dir = std::env::var("HOME").map_err(|_| "Could not find HOME environment variable")?;
    let config_dir = PathBuf::from(home_dir).join(".codemarks");
    std::fs::create_dir_all(&config_dir)?;
    Ok(config_dir.join("config.json"))
}

fn get_global_projects_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home_dir = std::env::var("HOME").map_err(|_| "Could not find HOME environment variable")?;
    let config_dir = PathBuf::from(home_dir).join(".codemarks");
    std::fs::create_dir_all(&config_dir)?;
    Ok(config_dir.join("projects.json"))
}

fn load_global_config() -> CodemarksConfig {
    if let Ok(config_path) = get_global_config_path() {
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str::<CodemarksConfig>(&content) {
                    return config;
                }
            }
        }
    }
    CodemarksConfig::default()
}

fn save_global_config(config: &CodemarksConfig) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_global_config_path()?;
    let json_content = serde_json::to_string_pretty(config)?;
    fs::write(config_path, json_content)?;
    Ok(())
}

fn load_global_projects() -> ProjectsDatabase {
    if let Ok(projects_path) = get_global_projects_path() {
        if projects_path.exists() {
            if let Ok(content) = fs::read_to_string(&projects_path) {
                if let Ok(projects_db) = serde_json::from_str::<ProjectsDatabase>(&content) {
                    return projects_db;
                }
            }
        }
    }
    ProjectsDatabase::default()
}

fn save_global_projects(projects_db: &ProjectsDatabase) -> Result<(), Box<dyn std::error::Error>> {
    let projects_path = get_global_projects_path()?;
    let json_content = serde_json::to_string_pretty(projects_db)?;
    fs::write(projects_path, json_content)?;
    Ok(())
}

#[derive(Parser)]
#[command(name = "codemarks")]
#[command(about = "A CLI tool for scanning and managing code annotations (TODO, FIXME, HACK)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Version,
    Scan {
        #[arg(short, long, default_value = ".")]
        directory: Option<PathBuf>,
        #[arg(short, long)]
        ignore: Vec<String>,
    },
    List,
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    Ci {
        #[arg(short, long, default_value = ".")]
        directory: Option<PathBuf>,
        #[arg(short, long)]
        pattern: Option<String>,
        #[arg(short, long)]
        ignore: Vec<String>,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    Show,
    SetPattern { pattern: String },
    Reset,
}

fn initialize_codemarks() -> Result<(), Box<dyn std::error::Error>> {
    let home_dir = std::env::var("HOME").map_err(|_| "Could not find HOME environment variable")?;
    let config_dir = PathBuf::from(home_dir).join(".codemarks");
    std::fs::create_dir_all(&config_dir)?;

    let config_path = config_dir.join("config.json");
    if !config_path.exists() {
        let default_config = CodemarksConfig::default();
        let config_json = serde_json::to_string_pretty(&default_config)?;
        std::fs::write(&config_path, config_json)?;
        println!("Created default config file at {}", config_path.display());
    }

    let projects_path = config_dir.join("projects.json");
    if !projects_path.exists() {
        let default_projects = ProjectsDatabase {
            projects: HashMap::new(),
        };
        let projects_json = serde_json::to_string_pretty(&default_projects)?;
        std::fs::write(&projects_path, projects_json)?;
        println!(
            "Created default projects file at {}",
            projects_path.display()
        );
    }

    Ok(())
}

fn main() {
    if let Err(e) = initialize_codemarks() {
        eprintln!("Warning: Failed to initialize codemarks: {e}");
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::Version => {
            println!("codemarks version {}", env!("CARGO_PKG_VERSION"));
        }
        Commands::Scan { directory, ignore } => {
            let dir = directory.as_deref().unwrap_or(Path::new("."));
            match scan::scan_directory(dir, &ignore) {
                Ok(count) => {
                    println!("Found {count} code annotations and saved to global projects database")
                }
                Err(e) => eprintln!("Error scanning directory: {e}"),
            }
        }
        Commands::List => match list::list_todos() {
            Ok(()) => {}
            Err(e) => eprintln!("Error listing codemarks: {e}"),
        },
        Commands::Config { action } => match config::handle_config(action) {
            Ok(()) => {}
            Err(e) => eprintln!("Error managing config: {e}"),
        },
        Commands::Ci {
            directory,
            pattern,
            ignore,
        } => {
            let dir = directory.as_deref().unwrap_or(Path::new("."));
            ci::run_ci(dir, pattern, &ignore);
        }
    }
}
