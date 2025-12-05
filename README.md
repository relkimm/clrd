<div align="center">

# clrd

**AI-Native Dead Code Detection**

*Transparent, Delicate, and Fast*

[![Crates.io](https://img.shields.io/crates/v/clrd.svg)](https://crates.io/crates/clrd)
[![npm](https://img.shields.io/npm/v/clrd.svg)](https://www.npmjs.com/package/clrd)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.77%2B-orange.svg)](https://www.rust-lang.org)

[Installation](#installation) · [Quick Start](#quick-start) · [Documentation](#documentation) · [Contributing](#contributing)

</div>

---

## Why clrd?

Traditional dead code tools blindly flag unused code. But modern codebases are complex—dynamic imports, barrel files, and framework magic create false positives everywhere.

**clrd** is different. Built for the **Agentic era**, it combines:

- **Rust Speed** — Oxc parser + Rayon parallelism for instant analysis
- **AI Intelligence** — Confidence scoring lets LLMs make the final call
- **Context Awareness** — Generates AI context files so agents understand your codebase

```bash
# Find dead code in seconds, not minutes
$ clrd scan

  Found 23 issues in 1,847 files (142ms)

  src/utils/legacy.ts
    ├─ unusedHelper (unused_export) — 0 references [confidence: 0.95]
    └─ deprecatedFn (unused_export) — 0 references [confidence: 0.87]

  src/components/Button.tsx
    └─ OldButtonProps (unused_type) — 0 references [confidence: 0.92]
```

---

## Installation

### Via npm (Recommended)

```bash
npx clrd scan
```

Or install globally:

```bash
npm install -g clrd
```

### Via Cargo

```bash
cargo install clrd
```

### From Source

```bash
git clone https://github.com/relkimm/clrd.git
cd clrd
cargo build --release
```

---

## Quick Start

### 1. Initialize (Optional)

Generate AI context files for Claude, Cursor, or other AI tools:

```bash
clrd init
```

This creates:
- `claude.md` — Context for Claude Code
- `agent.md` — Universal AI agent guide
- `.cursorrules` — Cursor editor context

### 2. Scan for Dead Code

```bash
# Pretty output (default)
clrd scan

# JSON output for LLM consumption
clrd scan --format json

# Interactive TUI
clrd scan --format tui

# Filter by confidence
clrd scan --confidence 0.8
```

### 3. Update AI Context

Keep your AI agents informed with the latest dead code report:

```bash
clrd map
```

### 4. Fix Dead Code

```bash
# Preview changes (dry run)
clrd fix --dry-run

# Comment out instead of delete
clrd fix --soft

# Actually remove (requires clean git)
clrd fix --force
```

---

## Features

### Dead Code Detection

| Type | Description |
|------|-------------|
| `unused_export` | Exported symbols with no external references |
| `unused_import` | Imports never used in the file |
| `zombie_file` | Files never imported by others |
| `unreachable_function` | Functions never called |
| `unused_type` | Types/Interfaces never referenced |
| `unused_class` | Classes never instantiated |
| `unused_enum` | Enums never used |

### Confidence Scoring

Not all dead code is equal. clrd assigns confidence scores to minimize false positives:

| Score | Meaning | Recommendation |
|-------|---------|----------------|
| **0.8+** | High confidence | Safe to remove |
| **0.5-0.8** | Medium confidence | Review recommended |
| **<0.5** | Low confidence | LLM judgment needed |

Factors that lower confidence:
- Dynamic imports (`import()`, `require()`)
- Test files
- Entry points (`index.ts`, `main.ts`)
- Public API markers

### AI Integration

clrd is designed to work seamlessly with AI agents:

```bash
# Output JSON schema for LLM tool use
clrd schema

# The JSON output is perfect for AI consumption
clrd scan --format json --output dead-code.json
```

---

## CLI Reference

```
clrd - AI-native code maintenance tool

USAGE:
    clrd <COMMAND>

COMMANDS:
    init     Initialize clrd with AI context files
    scan     Scan for dead code
    map      Update AI context files with scan results
    fix      Remove or comment out dead code
    schema   Output JSON schema for LLM integration

OPTIONS:
    -v, --verbose         Enable verbose output
    -C, --directory <DIR> Working directory
    -h, --help            Print help
    -V, --version         Print version
```

### `clrd scan`

```
OPTIONS:
    -f, --format <FORMAT>      Output format [default: pretty]
                               [values: pretty, json, compact, tui]
    -e, --extensions <EXT>     File extensions (comma-separated)
    -i, --ignore <PATTERN>     Patterns to ignore (comma-separated globs)
        --include-tests        Include test files in analysis
        --confidence <FLOAT>   Minimum confidence threshold [default: 0.5]
    -o, --output <FILE>        Output file (for json format)
```

### `clrd map`

```
OPTIONS:
        --scan                 Run a fresh scan before mapping
        --confidence <FLOAT>   Minimum confidence for reporting [default: 0.5]
```

### `clrd fix`

```
OPTIONS:
        --dry-run              Preview changes without modifying files
        --soft                 Comment out code instead of deleting
        --force                Force removal (requires clean git status)
        --confidence <FLOAT>   Only fix items above threshold [default: 0.8]
    -f, --files <FILES>        Specific files to fix
```

---

## Configuration

### Supported File Types

By default, clrd scans: `.ts`, `.tsx`, `.js`, `.jsx`, `.mjs`, `.cjs`

```bash
# Scan only TypeScript
clrd scan --extensions ts,tsx
```

### Ignore Patterns

Default ignores: `node_modules`, `dist`, `build`, `.git`

```bash
# Add custom ignores
clrd scan --ignore "**/*.test.ts,**/*.spec.ts,**/fixtures/**"
```

---

## How It Works

```
┌─────────────────────────────────────────────────────────────────┐
│                         clrd Pipeline                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   1. COLLECT         2. ANALYZE          3. DETECT              │
│   ┌──────────┐      ┌──────────┐       ┌────────────┐           │
│   │FileWalker│─────▶│   Oxc    │──────▶│ Reference  │           │
│   │ (rayon)  │      │  Parser  │       │   Graph    │           │
│   └──────────┘      └──────────┘       └────────────┘           │
│        │                 │                   │                  │
│        ▼                 ▼                   ▼                  │
│   1,847 files      AST Analysis       Dead Code Items           │
│     in 50ms        with exports/      with confidence           │
│                    imports            scores                    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

1. **FileWalker** — Parallel file traversal using `rayon`, respects `.gitignore`
2. **Oxc Parser** — Lightning-fast JavaScript/TypeScript AST parsing
3. **Reference Graph** — Cross-file analysis to find unused exports/imports

---

## Benchmarks

Tested on a large TypeScript monorepo (50,000+ files):

| Tool | Time | Memory |
|------|------|--------|
| **clrd** | **2.3s** | **180MB** |
| ts-prune | 45s | 1.2GB |
| knip | 38s | 890MB |

*Benchmarks run on Apple M2, 16GB RAM*

---

## Programmatic API

### Rust

```rust
use clrd::Scanner;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let scanner = Scanner::new("./src")
        .with_extensions(vec!["ts".into(), "tsx".into()])
        .with_confidence_threshold(0.8);

    let result = scanner.scan().await?;
    println!("Found {} issues", result.dead_code.len());
    Ok(())
}
```

---

## Comparison

| Feature | clrd | ts-prune | knip | unimported |
|---------|-----|----------|------|------------|
| Speed | Instant | Slow | Moderate | Moderate |
| Confidence scoring | Yes | No | No | No |
| AI integration | Native | No | No | No |
| JSON schema | Yes | No | Partial | No |
| Interactive TUI | Yes | No | No | No |
| Zero config | Yes | Yes | No | Yes |

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

```bash
# Clone the repo
git clone https://github.com/relkimm/clrd.git
cd clrd

# Install dependencies
cargo build

# Run tests
cargo test

# Run locally
cargo run -- scan --format pretty
```

---

## License

MIT License - see [LICENSE](LICENSE) for details.

---

<div align="center">

**Built with Rust and Oxc for the Agentic era**

[Report Bug](https://github.com/relkimm/clrd/issues) · [Request Feature](https://github.com/relkimm/clrd/issues)

</div>
