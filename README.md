# Codemarks

Codemarks is a CLI tool for scanning and managing code annotations such as `TODO`, `FIXME`, and `HACK` in your codebase. It helps you keep track of outstanding tasks and issues directly from your source code comments, storing them in a global database for easy review.

## Features
- Scan directories for code annotations (TODO, FIXME, HACK)
- List all found annotations grouped by project
- Manage a global configuration for annotation patterns
- Store and update annotation status (resolved/unresolved)
- CI/CD integration with non-zero exit codes for found annotations

## Installation
Build from source using Cargo:

```sh
cargo build --release
```

## Usage

### Show Version
Print the current version of Codemarks.

```sh
./codemarks version
```

### Scan for Annotations
Scan a directory (default: current directory) for code annotations and update the global database.

```sh
./codemarks scan --directory path/to/your/project
```

You can also ignore specific files or directories:

```sh
./codemarks scan --ignore "*.md" --ignore "docs/" --ignore "test_*"
```

### List Annotations
List all code annotations found across scanned projects.

```sh
./codemarks list
```

### CI/CD Mode
Run in CI mode to scan for codemarks and return a non-zero exit code if any are found. Perfect for continuous integration pipelines.

```sh
./codemarks ci
```

#### CI Command Options
- Use a custom pattern: `./codemarks ci --pattern "TODO|FIXME"`
- Scan specific directory: `./codemarks ci --directory src/`
- Ignore files/directories: `./codemarks ci --ignore "*.md" --ignore "docs/"`
- Combine options: `./codemarks ci --directory src/ --ignore "test_*" --pattern "TODO"`

The CI command will:
- Print found annotations with file paths and line numbers
- Return exit code 0 if no annotations are found
- Return exit code 1 if annotations are found (causing CI pipelines to fail)

### Manage Configuration
Show or update the global regex pattern for code annotations.

#### Show Current Configuration
```sh
./codemarks config show
```

#### Set a Custom Annotation Pattern
```sh
./codemarks config set-pattern "<your-regex-pattern>"
```

#### Reset to Default Pattern
```sh
./codemarks config reset
```

## Annotation Pattern
By default, Codemarks matches lines like:
- `// TODO: ...`
- `# FIXME ...`
- `<!-- HACK ... -->`

You can customize the regex pattern to match your team's conventions.

## Data Storage
- Configuration and annotation data are stored in `~/.codemarks/config.json` and `~/.codemarks/projects.json`.
- The tool respects `.gitignore` files and standard git ignore patterns.

## Examples

### Basic Usage
```sh
# Scan current directory
./codemarks scan

# List all found annotations
./codemarks list

# Check for annotations in CI
./codemarks ci
```

### Advanced CI Usage
```sh
# Only check source files, ignore documentation
./codemarks ci --directory src/ --ignore "*.md"

# Use custom pattern for only TODO comments
./codemarks ci --pattern "TODO" --ignore "vendor/" --ignore "node_modules/"

# Check multiple directories with different patterns
./codemarks ci --directory "src/" --directory "lib/" --pattern "FIXME|HACK"
```

## License
MIT
