# GitHub Copilot Instructions for Codemarks

## Project Context

**Codemarks** is a CLI tool for scanning and managing code annotations (TODO, FIXME, HACK) in codebases. It helps developers track outstanding tasks and issues directly from source code comments.

## Core Behavioral Principles

### 1. Always Verify Compilation

- **MUST** run `cargo check` after any code modifications
- **MUST** resolve all compilation errors before concluding
- **MUST** run `cargo fmt --all` and `cargo clippy` when finished with a task and rust files were modified
- **SHOULD** follow clippy suggestions
- **SHOULD** address warnings when practical
- **EXPLAIN** any remaining warnings if they cannot be resolved

### 2. Code Quality Standards

- **FOLLOW** existing code patterns and naming conventions
- **ADD** descriptive comments for complex logic only
- **HANDLE** errors gracefully with proper Result types
- **PREFER** small, focused functions over large monolithic ones
- **USE** the `ignore` crate for file traversal to respect `.gitignore`
- **MAINTAIN** modular structure with separate files for each command

### 3. CLI Design Principles

- **PROVIDE** clear help text and argument descriptions
- **SUPPORT** both short and long argument forms where appropriate
- **VALIDATE** user inputs (regex patterns, file paths, etc.)
- **GIVE** informative error messages for invalid inputs
- **MAINTAIN** consistent output formatting across commands


### 5. Data Storage Patterns

- **MAINTAIN** backward compatibility when changing data structures
- **HANDLE** missing or corrupted data files gracefully

### 6. Source code and version control

- **MUST** do not add or commit code to git yourself

## Technical Decision Framework

### When Making Changes:

1. **EXPLAIN** the reasoning behind technical decisions
2. **IDENTIFY** potential trade-offs or alternatives
3. **CONSIDER** impact on memory usage and performance
4. **ENSURE** changes fit with existing architecture patterns
5. **VALIDATE** that CLI patterns and error handling are maintained

## Communication Style

### Code Explanations:

- **START** with a brief summary of what will be changed
- **EXPLAIN** why the change is necessary
- **DESCRIBE** how the implementation works
- **HIGHLIGHT** any important considerations or caveats

### Error Handling:

- **PROVIDE** specific error messages and debugging context
- **SUGGEST** potential solutions when compilation fails
- **EXPLAIN** the root cause of issues when possible

### Documentation:

- **USE** structured formatting (bullet points, numbered lists)
- **INCLUDE** code examples for complex concepts
- **REFERENCE** relevant Rust documentation when helpful

## When In Doubt

### Ask for Clarification:

- If requirements are ambiguous or could be interpreted multiple ways
- When multiple implementation approaches have significant trade-offs
- If proposed changes might affect system stability or performance

### Reference Existing Code:

- Look for similar patterns already implemented in the codebase
- Follow established conventions for naming, error handling, and structure
- Maintain consistency with existing CLI command patterns

### Suggest Alternatives:

- Present multiple approaches with pros/cons when appropriate
- Explain trade-offs between memory usage, performance, and code complexity
- Consider both immediate implementation and future maintainability

## Success Criteria

A successful interaction should result in:

- ✅ Code that compiles without errors
- ✅ Follows Rust best practices and patterns correctly
- ✅ Maintains efficient performance for CLI operations
- ✅ Includes proper error handling and logging
- ✅ Is well-documented and follows project conventions
- ✅ Integrates seamlessly with existing architecture

## Example Interaction Pattern

1. **Understand** the request and current code context
2. **Ask for Clarification** if needed
3. **Explain** what changes will be made and why
4. **Implement** changes following these guidelines
5. **Verify** compilation with `cargo check`
6. **Summarize** what was accomplished and any important notes
