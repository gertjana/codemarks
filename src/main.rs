use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

mod ci;
mod clean;
mod config;
mod list;
mod scan;
mod watch;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Codemark {
    pub file: String,
    pub line_number: usize,
    pub description: String,
    #[serde(default)]
    pub resolved: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodemarksConfig {
    #[serde(default = "default_annotation_pattern")]
    pub annotation_pattern: String,
}

impl Default for CodemarksConfig {
    fn default() -> Self {
        Self {
            annotation_pattern: default_annotation_pattern(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ProjectsDatabase {
    pub projects: HashMap<String, Vec<Codemark>>,
}

#[must_use]
pub fn default_annotation_pattern() -> String {
    r"(?i)(?://|#|<!--|\*)\s*(?:TODO|FIXME|HACK)\s*:?\s*(.*)$".to_string()
}

pub fn get_global_config_path() -> Result<PathBuf> {
    let home_dir = std::env::var("HOME")
        .map_err(|_| anyhow::anyhow!("Could not find HOME environment variable"))?;
    let config_dir = PathBuf::from(home_dir).join(".codemarks");
    std::fs::create_dir_all(&config_dir)?;
    Ok(config_dir.join("config.json"))
}

pub fn get_global_projects_path() -> Result<PathBuf> {
    let home_dir = std::env::var("HOME")
        .map_err(|_| anyhow::anyhow!("Could not find HOME environment variable"))?;
    let config_dir = PathBuf::from(home_dir).join(".codemarks");
    std::fs::create_dir_all(&config_dir)?;
    Ok(config_dir.join("projects.json"))
}

#[must_use]
pub fn load_global_config() -> CodemarksConfig {
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

#[must_use]
pub fn load_global_config_no_storage() -> CodemarksConfig {
    CodemarksConfig::default()
}

pub fn save_global_config(config: &CodemarksConfig) -> Result<()> {
    let config_path = get_global_config_path()?;
    let json_content = serde_json::to_string_pretty(config)?;
    fs::write(config_path, json_content)?;
    Ok(())
}

pub fn save_global_config_no_storage(_config: &CodemarksConfig) -> Result<()> {
    // No-op when storage is disabled
    Ok(())
}

#[must_use]
pub fn load_global_projects() -> ProjectsDatabase {
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

#[must_use]
pub fn load_global_projects_no_storage() -> ProjectsDatabase {
    ProjectsDatabase::default()
}

pub fn save_global_projects(projects_db: &ProjectsDatabase) -> Result<()> {
    let projects_path = get_global_projects_path()?;
    let json_content = serde_json::to_string_pretty(projects_db)?;
    fs::write(projects_path, json_content)?;
    Ok(())
}

pub fn save_global_projects_no_storage(_projects_db: &ProjectsDatabase) -> Result<()> {
    // No-op when storage is disabled
    Ok(())
}

#[derive(Parser)]
#[command(name = "codemarks")]
#[command(
    about = "A CLI tool for scanning and managing code annotations (TODO, FIXME, HACK)",
    long_about = "Codemarks helps you track code annotations across your projects. Scan directories for TODO, FIXME, and HACK comments, watch for real-time changes, and integrate with CI/CD pipelines."
)]
struct Cli {
    /// Disable storage (don't create or read ~/.codemarks files)
    #[arg(long, global = true)]
    no_storage: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show version information
    Version,
    /// Scan a directory for code annotations
    Scan {
        /// Directory to scan for annotations
        #[arg(short, long, default_value = ".")]
        directory: Option<PathBuf>,
        /// Patterns to ignore when scanning files
        #[arg(short, long)]
        ignore: Vec<String>,
    },
    /// List all found annotations from the global database
    List,
    /// Manage global configuration settings
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Run in CI mode (returns non-zero exit code if annotations found)
    Ci {
        /// Directory to scan for annotations
        #[arg(short, long, default_value = ".")]
        directory: Option<PathBuf>,
        /// Custom regex pattern for annotations
        #[arg(short, long)]
        pattern: Option<String>,
        /// Patterns to ignore when scanning files
        #[arg(short, long)]
        ignore: Vec<String>,
    },
    /// Watch directory for changes and scan modified files in real-time
    Watch {
        /// Directory to watch for changes
        #[arg(short, long, default_value = ".")]
        directory: Option<PathBuf>,
        /// Patterns to ignore when watching files
        #[arg(short, long)]
        ignore: Vec<String>,
        /// Debounce time in milliseconds to avoid duplicate events
        #[arg(long, default_value = "500")]
        debounce: Option<u64>,
    },
    /// Remove resolved annotations from the global database
    Clean {
        /// Show what would be removed without actually removing it
        #[arg(short = 'n', long)]
        dry_run: bool,
        /// Specific project to clean (if not specified, cleans all projects)
        #[arg(short, long)]
        project: Option<String>,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    Show,
    SetPattern { pattern: String },
    Reset,
}

fn initialize_codemarks() -> Result<()> {
    let home_dir = std::env::var("HOME")
        .map_err(|_| anyhow::anyhow!("Could not find HOME environment variable"))?;
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
    let cli = Cli::parse();

    if !cli.no_storage {
        if let Err(e) = initialize_codemarks() {
            eprintln!("Warning: Failed to initialize codemarks: {e}");
        }
    }

    match cli.command {
        Commands::Version => {
            println!("codemarks version {}", env!("CARGO_PKG_VERSION"));
        }
        Commands::Scan { directory, ignore } => {
            let dir = directory.as_deref().unwrap_or(Path::new("."));
            match scan::scan_directory(dir, &ignore, cli.no_storage) {
                Ok(count) => {
                    if cli.no_storage {
                        println!("Found {count} code annotations");
                    } else {
                        println!("Found {count} and saved ode annotations");
                    }
                }
                Err(e) => eprintln!("Error scanning directory: {e}"),
            }
        }
        Commands::List => {
            list::list_codemarks(cli.no_storage);
        }
        Commands::Config { action } => {
            if cli.no_storage {
                eprintln!(
                    "Config management is not available when storage is disabled (--no-storage)"
                );
                std::process::exit(1);
            }
            match config::handle_config(action) {
                Ok(()) => {}
                Err(e) => eprintln!("Error managing config: {e}"),
            }
        }
        Commands::Ci {
            directory,
            pattern,
            ignore,
        } => {
            let dir = directory.as_deref().unwrap_or(Path::new("."));
            // CI mode defaults to no-storage behavior (override the global flag)
            ci::run_ci(dir, pattern, &ignore);
        }
        Commands::Watch {
            directory,
            ignore,
            debounce,
        } => {
            let dir = directory.as_deref().unwrap_or(Path::new("."));
            match watch::watch_directory(dir, &ignore, debounce, cli.no_storage) {
                Ok(()) => {}
                Err(e) => eprintln!("Error watching directory: {e}"),
            }
        }
        Commands::Clean { dry_run, project } => {
            if cli.no_storage {
                eprintln!("Clean command is not available when storage is disabled (--no-storage)");
                std::process::exit(1);
            }
            match clean::clean_resolved(dry_run, project) {
                Ok(()) => {}
                Err(e) => eprintln!("Error cleaning resolved annotations: {e}"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    /// Helper function to set up a temporary home directory for testing
    fn setup_temp_home() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        unsafe {
            env::set_var("HOME", temp_dir.path());
        }
        temp_dir
    }

    #[test]
    fn test_codemark_creation() {
        let codemark = Codemark {
            file: "test.rs".to_string(),
            line_number: 42,
            description: "This is a test TODO".to_string(),
            resolved: false,
        };

        assert_eq!(codemark.file, "test.rs");
        assert_eq!(codemark.line_number, 42);
        assert_eq!(codemark.description, "This is a test TODO");
        assert!(!codemark.resolved);
    }

    #[test]
    fn test_codemark_serialization() {
        let codemark = Codemark {
            file: "test.rs".to_string(),
            line_number: 42,
            description: "This is a test TODO".to_string(),
            resolved: false,
        };

        let json = serde_json::to_string(&codemark).expect("Failed to serialize");
        let deserialized: Codemark = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(codemark.file, deserialized.file);
        assert_eq!(codemark.line_number, deserialized.line_number);
        assert_eq!(codemark.description, deserialized.description);
        assert_eq!(codemark.resolved, deserialized.resolved);
    }

    #[test]
    fn test_default_annotation_pattern() {
        let pattern = default_annotation_pattern();
        assert!(pattern.contains("TODO"));
        assert!(pattern.contains("FIXME"));
        assert!(pattern.contains("HACK"));
    }

    #[test]
    fn test_codemarks_config_default() {
        let config = CodemarksConfig::default();
        assert_eq!(config.annotation_pattern, default_annotation_pattern());
    }

    #[test]
    fn test_codemarks_config_serialization() {
        let config = CodemarksConfig {
            annotation_pattern: "CUSTOM_PATTERN".to_string(),
        };

        let json = serde_json::to_string(&config).expect("Failed to serialize config");
        let deserialized: CodemarksConfig =
            serde_json::from_str(&json).expect("Failed to deserialize config");

        assert_eq!(config.annotation_pattern, deserialized.annotation_pattern);
    }

    #[test]
    fn test_projects_database_default() {
        let db = ProjectsDatabase::default();
        assert!(db.projects.is_empty());
    }

    #[test]
    fn test_projects_database_operations() {
        let mut db = ProjectsDatabase::default();
        let codemark = Codemark {
            file: "test.rs".to_string(),
            line_number: 1,
            description: "Test annotation".to_string(),
            resolved: false,
        };

        // Add a project with codemarks
        db.projects
            .insert("test_project".to_string(), vec![codemark.clone()]);

        assert_eq!(db.projects.len(), 1);
        assert!(db.projects.contains_key("test_project"));
        assert_eq!(db.projects["test_project"].len(), 1);
        assert_eq!(
            db.projects["test_project"][0].description,
            "Test annotation"
        );
    }

    #[test]
    fn test_get_global_config_path() {
        let _temp_home = setup_temp_home();

        let config_path = get_global_config_path().expect("Failed to get config path");
        assert!(
            config_path
                .to_string_lossy()
                .ends_with(".codemarks/config.json")
        );

        // The directory should be created
        assert!(config_path.parent().unwrap().exists());
    }

    #[test]
    fn test_get_global_projects_path() {
        let _temp_home = setup_temp_home();

        let projects_path = get_global_projects_path().expect("Failed to get projects path");
        assert!(
            projects_path
                .to_string_lossy()
                .ends_with(".codemarks/projects.json")
        );

        // The directory should be created
        assert!(projects_path.parent().unwrap().exists());
    }

    #[test]
    fn test_load_global_config_default() {
        let _temp_home = setup_temp_home();

        // Load config when no file exists should return default
        let config = load_global_config();
        assert_eq!(config.annotation_pattern, default_annotation_pattern());
    }

    #[test]
    fn test_load_global_projects_default() {
        let _temp_home = setup_temp_home();

        // Load projects when no file exists should return default
        let projects = load_global_projects();
        assert!(projects.projects.is_empty());
    }
}
