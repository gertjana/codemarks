// src/list.rs
// Handles the list command for codemarks

use crate::load_global_projects;

pub fn list_todos() -> Result<(), Box<dyn std::error::Error>> {
    let projects_db = load_global_projects();
    if projects_db.projects.is_empty() {
        println!("No code annotations found. Run 'codemarks scan' first to scan for annotations.");
        return Ok(());
    }
    for (project_name, todos) in &projects_db.projects {
        if todos.is_empty() {
            continue;
        }
        println!("{project_name}");
        for todo in todos {
            let resolved_prefix = if todo.resolved { "✅ " } else { "   " };
            println!(
                "{}{}:{} {}",
                resolved_prefix, todo.file, todo.line_number, todo.description
            );
        }
        if projects_db.projects.len() > 1 {
            println!();
        }
    }
    Ok(())
}
