# clrd - AI-Native Code Maintenance Tool

> **"Transparent, Delicate, and Fast"**

## Project Overview

**clrd** (pronounced "Cleared") is a high-performance dead code detection tool built with Rust for the modern "Agentic" era. Unlike traditional linters that blindly delete code, clrd combines **Rust's speed** with **LLM intelligence** for safe, accurate code cleanup.

### Core Philosophy

1. **Transparent**: Clear reporting of what code is dead and why
2. **Delicate**: Confidence scoring to minimize false positives
3. **Fast**: Oxc parser + Rayon parallelism for maximum performance

### Target Users

- AI agents (Claude, Cursor, GPT-based tools)
- Developers using AI-assisted code maintenance
- Teams wanting to keep codebases clean

---

## Architecture: The Scan-Map-Judge-Act Protocol

```
┌───────────────────────────────────────────────────────────────────┐
│                        clrd Architecture                          │
├───────────────────────────────────────────────────────────────────┤
│                                                                   │
│   Phase 1: SCAN          Phase 2: MAP          Phase 3: ACT      │
│   ┌─────────────┐       ┌─────────────┐       ┌─────────────┐    │
│   │ FileWalker  │       │   Mapper    │       │    Fix      │    │
│   │ (parallel)  │──────▶│ (context)   │──────▶│  (remove)   │    │
│   │ AstAnalyzer │       │ claude.md   │       │ --dry-run   │    │
│   │ RefGraph    │       │ agent.md    │       │ --soft      │    │
│   └─────────────┘       └─────────────┘       │ --force     │    │
│         │                      │              └─────────────┘    │
│         ▼                      ▼                    │            │
│   ┌─────────────┐       ┌─────────────┐            │            │
│   │ ScanOutput  │       │ AI Context  │◀───────────┘            │
│   │ (JSON)      │──────▶│ Files       │    LLM Judgment         │
│   └─────────────┘       └─────────────┘                         │
│                                                                   │
└───────────────────────────────────────────────────────────────────┘
```

### Phase 1: Scan (Rust Speed)
- Parallel file walking with `rayon` and `ignore` crate
- AST parsing with `oxc_parser` (fastest JS/TS parser)
- Cross-file reference graph building
- Confidence-based dead code detection

### Phase 2: Map (Context Injection)
- Generates AI context files: `claude.md`, `agent.md`, `.cursorrules`
- Provides "vision" to AI agents before asking them to fix code
- Smart section replacement preserving user content

### Phase 3: Act (Delicate Execution)
- `--dry-run`: Preview mode (default)
- `--soft`: Comment out code instead of deleting
- `--force`: Hard delete (requires clean git status)

---

## Directory Structure

```
src/
├── main.rs              # CLI binary entry point
├── lib.rs               # Library root + NAPI exports for npm
├── cli/
│   ├── mod.rs           # CLI argument parsing (clap)
│   └── commands/
│       ├── init.rs      # `clrd init` - create context files
│       ├── scan.rs      # `clrd scan` - detect dead code
│       ├── map.rs       # `clrd map` - update context files
│       ├── fix.rs       # `clrd fix` - remove dead code
│       └── schema.rs    # `clrd schema` - output JSON schema
├── scanner/
│   ├── mod.rs           # Scanner orchestrator
│   ├── file_walker.rs   # Parallel file system traversal
│   ├── analyzer.rs      # Oxc-based AST analysis
│   └── reference_graph.rs # Cross-file reference tracking
├── mapper/
│   ├── mod.rs           # Context file generator
│   └── templates.rs     # Template content for context files
├── tui/
│   └── mod.rs           # Interactive terminal UI (ratatui)
└── types/
    └── mod.rs           # Core data structures + JSON schemas
```

---

## Key Modules

### Scanner (`src/scanner/`)
The core engine that detects dead code through three phases:

| Component | Responsibility |
|-----------|----------------|
| `FileWalker` | Parallel file collection, respects .gitignore |
| `AstAnalyzer` | Oxc-based AST parsing, extracts exports/imports |
| `ReferenceGraph` | Builds cross-file reference map, finds dead code |

**Dead Code Types Detected:**
- `UnusedExport` - Exported symbol with no external references
- `UnusedImport` - Import never used in file
- `ZombieFile` - File never imported by others
- `UnreachableFunction` - Function never called
- `UnusedType/Class/Enum` - Type definitions never referenced

### Mapper (`src/mapper/`)
Generates and updates AI context files:
- **claude.md**: XML-formatted for Claude Code
- **agent.md**: Universal markdown format
- **.cursorrules**: Cursor editor format

### Types (`src/types/`)
Core data structures with `schemars` JSON schema support for LLM integration:
- `DeadCodeItem` - Individual finding with confidence score
- `ScanOutput` - Complete scan results
- `LlmJudgmentRequest/Response` - LLM communication format

---

## Tech Stack

| Category | Technology | Purpose |
|----------|------------|---------|
| Parser | `oxc` (v0.56) | Ultra-fast JS/TS parsing |
| Parallelism | `rayon` | Data parallel processing |
| CLI | `clap` (v4.5) | Argument parsing |
| TUI | `ratatui` + `crossterm` | Interactive terminal UI |
| Serialization | `serde` + `schemars` | JSON + schema generation |
| Node.js | `napi-rs` | NPM distribution |
| Async | `tokio` | Async runtime |

---

## CLI Commands

```bash
# Initialize project with AI context files
clrd init [--force]

# Scan for dead code
clrd scan [--format pretty|json|compact|tui]
         [--extensions ts,tsx,js]
         [--ignore "**/*.test.ts"]
         [--confidence 0.5]
         [--include-tests]
         [--output report.json]

# Update AI context files with scan results
clrd map [--confidence 0.5]

# Fix dead code
clrd fix [--dry-run]      # Preview only (default)
        [--soft]         # Comment out instead of delete
        [--force]        # Hard delete (requires clean git)
        [--confidence 0.8]

# Output JSON schema for LLM integration
clrd schema
```

---

## Build & Development

### Prerequisites
- Rust 1.77+
- Node.js 18+ (for npm distribution)

### Build Commands
```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Build for npm (via napi-rs)
npm run build

# Debug build for npm
npm run build:debug
```

### Running Locally
```bash
# Via Cargo
cargo run -- scan --format pretty

# Via npm (after build)
node bin/clrd.js scan --format pretty
```

---

## Code Style Guidelines

### Rust Conventions
- Use `anyhow::Result` for error handling in CLI/application code
- Use `thiserror` for library error types
- Prefer `tracing` over `println!` for logging
- Use builder pattern for configurable structs (see `Scanner`)

### Module Organization
- Each module has `mod.rs` as entry point
- Public types re-exported at module level
- Internal implementation in separate files

### Performance Considerations
- Use `rayon::par_iter()` for CPU-bound parallel operations
- Minimize lock contention with `Arc<Mutex<>>` only when necessary
- Prefer owned types over references in parallel code

---

## Confidence Scoring

Dead code items have a confidence score (0.0 - 1.0):

| Score | Meaning | Action |
|-------|---------|--------|
| 0.8+ | High confidence | Safe to auto-remove |
| 0.5-0.8 | Medium confidence | Review recommended |
| <0.5 | Low confidence | LLM judgment needed |

**Factors that lower confidence:**
- Dynamic imports/requires
- Test files
- Entry point files (index, main, app)
- Public API markers

---

## LLM Integration

### JSON Schema
Use `clrd schema` to output JSON schema for LLM tool use.

### Judgment Request Format
```json
{
  "items": [
    {
      "file_path": "src/utils/helper.ts",
      "name": "unusedFunction",
      "kind": "unused_export",
      "confidence": 0.75,
      "code_snippet": "export function unusedFunction() {...}",
      "reason": "0 references found"
    }
  ],
  "project_context": {
    "name": "my-project",
    "framework": "react"
  }
}
```

### Expected Response Format
```json
{
  "confirmed": [
    { "file_path": "src/utils/helper.ts", "name": "unusedFunction", "action": "delete" }
  ],
  "rejected": [
    { "file_path": "...", "name": "...", "reason": "Used via dynamic import" }
  ]
}
```

---

## Platform Support

Built for cross-platform distribution via npm:
- Windows (x64)
- macOS (x64, arm64)
- Linux (x64 gnu, arm64 gnu)

---

## Contributing

1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Ensure `cargo test` passes
5. Submit a pull request

### Areas for Improvement
- [ ] Configuration file support (.clrdrc.json)
- [ ] Incremental scanning with caching
- [ ] Plugin system for custom detectors
- [ ] IDE/LSP integration
- [ ] Module resolution improvements
