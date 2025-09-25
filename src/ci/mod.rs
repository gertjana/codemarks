// src/ci.rs
// Handles the ci command for codemarks

use anyhow::Result;
use ignore::{WalkBuilder, overrides::OverrideBuilder};
use regex::Regex;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::default_annotation_pattern;

/// Helper function that returns the count instead of exiting (for testing)
#[allow(dead_code)]
pub fn count_annotations(
    directory: &Path,
    pattern: Option<String>,
    ignore_patterns: &[String],
) -> Result<usize> {
    let pattern_to_use = pattern.unwrap_or_else(default_annotation_pattern);
    let codemark_regex = Regex::new(&pattern_to_use)?;
    let mut found = 0;

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
                let path = entry.path();
                if path.is_file()
                    && let Ok(file) = std::fs::File::open(path)
                {
                    let reader = BufReader::new(file);
                    for line_content in reader.lines().map_while(Result::ok) {
                        if codemark_regex.is_match(&line_content) {
                            found += 1;
                        }
                    }
                }
            }
            Err(err) => eprintln!("Error accessing path: {err}"),
        }
    }

    Ok(found)
}

pub fn run_ci(directory: &Path, pattern: Option<String>, ignore_patterns: &[String]) -> ! {
    let pattern_to_use = pattern.unwrap_or_else(default_annotation_pattern);
    let codemark_regex = Regex::new(&pattern_to_use).expect("Invalid regex pattern");
    let mut found = 0;

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

    #[allow(clippy::manual_flatten)]
    for result in builder.build() {
        if let Ok(entry) = result {
            let file_path = entry.path();
            if entry.file_type().is_some_and(|ft| ft.is_file())
                && let Ok(file) = fs::File::open(file_path)
            {
                let reader = BufReader::new(file);
                for (line_number, line) in reader.lines().enumerate() {
                    if let Ok(line_content) = line
                        && codemark_regex.is_match(&line_content)
                    {
                        found += 1;
                        println!(
                            "{}:{}: {}",
                            file_path.display(),
                            line_number + 1,
                            line_content
                        );
                    }
                }
            }
        }
    }

    if found > 0 {
        println!("Found {found} codemarks matching pattern.");
        std::process::exit(1);
    } else {
        println!("No codemarks found matching pattern.");
        std::process::exit(0);
    }
}

#[cfg(test)]
mod tests;
