# Codemarks

Codemarks is a CLI tool for scanning and managing code annotations such as `TODO`, `FIXME`, and `HACK` in your codebase. It helps you keep track of outstanding tasks and issues directly from your source code comments, storing them in a global database for easy review.

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

### Clean Resolved Annotations
Remove resolved annotations from the global database. This helps keep your database clean by removing annotations that have been completed.

```sh
./codemarks clean
```

#### Clean Command Options
- Preview what would be removed: `./codemarks clean --dry-run`
- Clean specific project only: `./codemarks clean --project "my_project"`
- Combine options: `./codemarks clean --dry-run --project "my_project"`

The clean command will:
- Remove all annotations marked as resolved (`resolved: true`) from the database
- Preserve unresolved annotations for continued tracking
- Remove entire projects if all their annotations are resolved
- Show detailed summary of what was removed

Use the `--dry-run` option to preview what would be cleaned before making changes.

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

### Watch for Changes
Watch a directory for file changes and automatically scan any modified files for annotations. This is perfect for development environments where you want real-time feedback on code annotations.

```sh
./codemarks watch
```

#### Watch Command Options
- Watch specific directory: `./codemarks watch --directory src/`
- Ignore files/directories: `./codemarks watch --ignore "*.md" --ignore "docs/"`
- Set debounce time: `./codemarks watch --debounce 1000` (in milliseconds)
- Combine options: `./codemarks watch --directory src/ --ignore "test_*" --debounce 750`

The watch command will:
- Monitor the specified directory for file system changes
- Automatically scan modified files for annotations
- Update the global projects database in real-time
- Respect `.gitignore` patterns and custom ignore rules
- Use debouncing to avoid duplicate scans of rapidly changing files

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

# Clean resolved annotations
./codemarks clean

# Preview what would be cleaned
./codemarks clean --dry-run

# Check for annotations in CI
./codemarks ci

# Watch for changes in real-time
./codemarks watch
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

### Watch Mode Examples
```sh
# Watch current directory for changes
./codemarks watch

# Watch specific directory with ignore patterns
./codemarks watch --directory src/ --ignore "*.test.js" --ignore "*.spec.ts"

# Watch with custom debounce time (useful for large projects)
./codemarks watch --debounce 1000 --ignore "node_modules/" --ignore "target/"
```

## License
MIT
