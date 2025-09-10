// src/ci.rs
// Handles the ci command for codemarks

use regex::Regex;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use ignore::{WalkBuilder, overrides::OverrideBuilder};

use crate::default_annotation_pattern;

pub fn run_ci(directory: &Path, pattern: Option<String>, ignore_patterns: &[String]) -> ! {
    let pattern_to_use = pattern.unwrap_or_else(|| default_annotation_pattern());
    let todo_regex = Regex::new(&pattern_to_use).expect("Invalid regex pattern");
    let mut found = 0;
    
    let mut builder = WalkBuilder::new(directory);
    
    // Add custom ignore patterns using overrides
    if !ignore_patterns.is_empty() {
        let mut override_builder = OverrideBuilder::new(directory);
        for pattern in ignore_patterns {
            // Add as negative override (ignore pattern)
            if let Err(e) = override_builder.add(&format!("!{}", pattern)) {
                eprintln!("Warning: Invalid ignore pattern '{}': {}", pattern, e);
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
                if entry.file_type().map_or(false, |ft| ft.is_file()) {
                    if let Ok(file) = fs::File::open(file_path) {
                        let reader = BufReader::new(file);
                        for (line_number, line) in reader.lines().enumerate() {
                            if let Ok(line_content) = line {
                                if todo_regex.is_match(&line_content) {
                                    found += 1;
                                    println!("{}:{}: {}", file_path.display(), line_number + 1, line_content);
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => continue,
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
