// src/config.rs
// Handles the config command for codemarks

use crate::{
    CodemarksConfig, ConfigAction, get_global_config_path, get_global_projects_path,
    load_global_config, save_global_config,
};
use regex::Regex;

pub fn handle_config(action: ConfigAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ConfigAction::Show => {
            let config = load_global_config();
            println!("Global code annotation pattern:");
            println!("{}", config.annotation_pattern);
            if let Ok(config_path) = get_global_config_path() {
                println!("\nConfig file location: {}", config_path.display());
            }
            if let Ok(projects_path) = get_global_projects_path() {
                println!("Projects file location: {}", projects_path.display());
            }
        }
        ConfigAction::SetPattern { pattern } => match Regex::new(&pattern) {
            Ok(_) => {
                let config = CodemarksConfig {
                    annotation_pattern: pattern.clone(),
                };
                save_global_config(&config)?;
                println!("Global code annotation pattern updated to: {pattern}");
            }
            Err(e) => {
                eprintln!("Invalid regex pattern: {e}");
                return Err(e.into());
            }
        },
        ConfigAction::Reset => {
            let config = CodemarksConfig::default();
            save_global_config(&config)?;
            println!(
                "Global code annotation pattern reset to default: {0}",
                config.annotation_pattern
            );
        }
    }
    Ok(())
}
