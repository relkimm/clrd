# Contributing to clr

First off, thanks for taking the time to contribute!

## Code of Conduct

This project and everyone participating in it is governed by our commitment to providing a welcoming and inclusive environment. Please be respectful and constructive in all interactions.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates.

When creating a bug report, include:

- **Clear title** describing the issue
- **Steps to reproduce** the behavior
- **Expected behavior** vs actual behavior
- **Environment details** (OS, Rust version, Node version)
- **Code samples** if applicable

### Suggesting Features

Feature requests are welcome! Please:

- Check if the feature has already been requested
- Describe the use case and why it would be valuable
- Consider how it fits with the project's philosophy

### Pull Requests

1. **Fork** the repository
2. **Create a branch** for your feature (`git checkout -b feature/amazing-feature`)
3. **Make your changes**
4. **Write or update tests** as needed
5. **Run the test suite** (`cargo test`)
6. **Commit your changes** (`git commit -m 'Add amazing feature'`)
7. **Push to your branch** (`git push origin feature/amazing-feature`)
8. **Open a Pull Request**

## Development Setup

### Prerequisites

- Rust 1.75 or higher
- Node.js 18 or higher (for npm builds)

### Getting Started

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/clr.git
cd clr

# Build the project
cargo build

# Run tests
cargo test

# Run the CLI
cargo run -- scan --format pretty
```

### Building for npm

```bash
# Install Node dependencies
npm install

# Build native module
npm run build

# Test npm package
node bin/clr.js scan
```

## Project Structure

```
src/
├── main.rs           # CLI binary entry point
├── lib.rs            # Library root + NAPI exports
├── cli/              # Command implementations
├── scanner/          # Dead code detection engine
├── mapper/           # AI context file generation
├── tui/              # Terminal UI
└── types/            # Core data structures
```

## Coding Guidelines

### Rust Style

- Follow standard Rust formatting (`cargo fmt`)
- Use `clippy` for linting (`cargo clippy`)
- Write doc comments for public APIs
- Use `anyhow::Result` for error handling in application code
- Use `thiserror` for library error types

### Commit Messages

- Use present tense ("Add feature" not "Added feature")
- Use imperative mood ("Move cursor to..." not "Moves cursor to...")
- Keep the first line under 72 characters
- Reference issues when applicable

Good examples:
```
Add confidence threshold flag to scan command
Fix false positive detection for barrel files
Improve parallel file walking performance
```

### Testing

- Write unit tests for new functionality
- Ensure existing tests pass
- Add integration tests for CLI commands when appropriate

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_scanner_creation

# Run with output
cargo test -- --nocapture
```

## Areas We Need Help

- **Module Resolution** — Improving TypeScript-style path resolution
- **Framework Detection** — Better handling of React, Vue, Angular patterns
- **Performance** — Caching and incremental scanning
- **Documentation** — Examples, tutorials, and guides
- **Platform Testing** — Testing on different OS and architectures

## Questions?

Feel free to open an issue for any questions about contributing.

Thank you for helping make clr better!
