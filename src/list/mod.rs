// src/list.rs
// Handles the list command for codemarks

use crate::load_global_projects;

pub fn list_codemarks(ephemeral: bool) {
    if ephemeral {
        println!("No code annotations available (ephemeral mode).");
        return;
    }
    let projects_db = load_global_projects(false);
    if projects_db.projects.is_empty() {
        println!("No code annotations found. Run 'codemarks scan' first to scan for annotations.");
        return;
    }
    for (project_name, codemarks) in &projects_db.projects {
        if codemarks.is_empty() {
            continue;
        }
        println!("{project_name}");
        for codemark in codemarks {
            let resolved_prefix = if codemark.resolved { "✅ " } else { "   " };
            println!(
                "{}{}:{} {}",
                resolved_prefix, codemark.file, codemark.line_number, codemark.description
            );
        }
        if projects_db.projects.len() > 1 {
            println!();
        }
    }
}

#[cfg(test)]
mod tests;
