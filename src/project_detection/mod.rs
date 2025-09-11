use std::fs;
use std::path::Path;

/// Intelligently determine the project name based on language-specific configuration files
pub fn detect_project_name(directory: &Path) -> String {
    let canonical_dir = match directory.canonicalize() {
        Ok(dir) => dir,
        Err(_) => directory.to_path_buf(),
    };

    // Helper function to read and parse JSON files
    let read_json_field = |file_path: &Path, field: &str| -> Option<String> {
        if file_path.exists() {
            if let Ok(content) = fs::read_to_string(file_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(name) = json.get(field).and_then(|v| v.as_str()) {
                        return Some(name.to_string());
                    }
                }
            }
        }
        None
    };

    // Helper function to read simple key=value files
    let read_key_value = |file_path: &Path, key: &str| -> Option<String> {
        if file_path.exists() {
            if let Ok(content) = fs::read_to_string(file_path) {
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with(key) && line.contains('=') {
                        if let Some(value) = line.split('=').nth(1) {
                            return Some(value.trim().trim_matches('"').to_string());
                        }
                    }
                }
            }
        }
        None
    };

    // Check various project configuration files in order of preference

    // Rust: Cargo.toml
    if let Some(name) = read_key_value(&canonical_dir.join("Cargo.toml"), "name") {
        return name;
    }

    // Node.js: package.json (npm, yarn, bun)
    if let Some(name) = read_json_field(&canonical_dir.join("package.json"), "name") {
        return name;
    }

    // Go: go.mod
    if let Ok(content) = fs::read_to_string(canonical_dir.join("go.mod")) {
        if let Some(first_line) = content.lines().next() {
            if first_line.starts_with("module ") {
                let module_name = first_line.trim_start_matches("module ").trim();
                // Extract just the project name from the full module path
                if let Some(project_name) = module_name.split('/').next_back() {
                    return project_name.to_string();
                }
                return module_name.to_string();
            }
        }
    }

    // Scala: build.sbt
    if let Ok(content) = fs::read_to_string(canonical_dir.join("build.sbt")) {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("name :=") {
                if let Some(name_part) = line.split(":=").nth(1) {
                    let name = name_part.trim().trim_matches('"').trim();
                    return name.to_string();
                }
            }
        }
    }

    // Java: pom.xml (Maven)
    if let Ok(content) = fs::read_to_string(canonical_dir.join("pom.xml")) {
        // Simple XML parsing for <artifactId>
        if let Some(start) = content.find("<artifactId>") {
            if let Some(end) = content[start..].find("</artifactId>") {
                let artifact_start = start + "<artifactId>".len();
                let artifact_end = start + end;
                if artifact_end > artifact_start {
                    return content[artifact_start..artifact_end].trim().to_string();
                }
            }
        }
    }

    // Java: build.gradle or build.gradle.kts (Gradle)
    for gradle_file in ["build.gradle", "build.gradle.kts"] {
        if let Ok(content) = fs::read_to_string(canonical_dir.join(gradle_file)) {
            // Look for rootProject.name or archivesBaseName
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("rootProject.name") && line.contains('=') {
                    if let Some(name_part) = line.split('=').nth(1) {
                        let name = name_part.trim().trim_matches('"').trim_matches('\'');
                        return name.to_string();
                    }
                }
            }
        }
    }

    // Elixir: mix.exs
    if let Ok(content) = fs::read_to_string(canonical_dir.join("mix.exs")) {
        // Look for "app: :project_name" in mix.exs
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("app:") && line.contains(':') {
                if let Some(app_part) = line.split(':').nth(1) {
                    let app_name = app_part.trim().trim_matches(',').trim();
                    if app_name.starts_with(':') {
                        return app_name.trim_start_matches(':').to_string();
                    }
                }
            }
        }
    }

    // Python: pyproject.toml
    if let Ok(content) = fs::read_to_string(canonical_dir.join("pyproject.toml")) {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("name =") {
                if let Some(name_part) = line.split('=').nth(1) {
                    let name = name_part.trim().trim_matches('"').trim_matches('\'');
                    return name.to_string();
                }
            }
        }
    }

    // Python: setup.py (basic pattern matching)
    if let Ok(content) = fs::read_to_string(canonical_dir.join("setup.py")) {
        // Look for name= in setup() call
        if let Some(name_start) = content.find("name=") {
            let after_equals = &content[name_start + 5..];
            if let Some(quote_start) = after_equals.find('"').or_else(|| after_equals.find('\'')) {
                let quote_char = after_equals.chars().nth(quote_start).unwrap();
                let after_quote = &after_equals[quote_start + 1..];
                if let Some(quote_end) = after_quote.find(quote_char) {
                    return after_quote[..quote_end].to_string();
                }
            }
        }
    }

    // Fallback to directory name
    canonical_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string()
}

#[cfg(test)]
mod tests;
