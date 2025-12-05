//! Templates for AI context files

/// Template for claude.md - Claude Code context file
pub const CLAUDE_MD_TEMPLATE: &str = r#"# Project Context for Claude

## Dead Code Detection (clrd)

This project uses [clrd](https://npmjs.com/package/clrd) for dead code detection.

### Usage

When user asks to "clean up unused code" or "remove dead code":

```bash
npx clrd scan --format json
```

### Decision Rules

```
if confidence >= 0.9:
    if kind in [unused_import, unused_export, unreachable_function]:
        → Auto-remove
    if kind == zombie_file:
        → Ask user: "Delete {filename}?"

if confidence < 0.9:
    → Show list to user, ask which to remove

if public_api == true:
    → Never remove (exposed in package.json exports)
```

### Removal Checklist

1. Remove the dead code itself
2. Remove imports that become orphaned
3. Delete files that become empty
4. Group related removals (function + its imports)

### Common False Positives

Skip these patterns - they may appear unused but aren't:
- `__tests__/`, `*.test.ts`, `*.spec.ts` - test utilities
- `*.d.ts` - type declarations
- Files with `@public` or `@api` JSDoc tags
- Entry points in package.json (`main`, `exports`, `bin`)
- Dynamic imports: `require(variable)`, `import(expr)`
- Decorator factories: `@Injectable()`, `@Component()`

### Output Format

After cleanup, summarize:
```
Removed X items:
- 3 unused imports
- 2 unused exports
- 1 unreachable function

Skipped Y items (need confirmation):
- src/legacy.ts (zombie file)
```

---

## Project Structure
<!-- Describe your project structure -->

## Conventions
<!-- Document coding conventions -->
"#;

/// Template for agent.md - Universal AI agent guide
pub const AGENT_MD_TEMPLATE: &str = r#"# Agent Guide

## Dead Code Detection (clrd)

Scan: `npx clrd scan --format json`

### Quick Reference

| confidence | kind | action |
|------------|------|--------|
| >= 0.9 | unused_import | Auto-remove |
| >= 0.9 | unused_export | Auto-remove |
| >= 0.9 | unreachable_function | Auto-remove |
| >= 0.9 | zombie_file | Ask user |
| < 0.9 | * | Ask user |
| * | public_api: true | Keep |

### Process

```
1. Run scan
2. Filter by confidence >= 0.9
3. Remove dead code
4. Clean orphaned imports
5. Delete empty files
6. Report summary to user
```

### Skip These (False Positives)

- Test files (`*.test.ts`, `__tests__/`)
- Type declarations (`*.d.ts`)
- Package entry points (check `package.json`)
- Dynamic imports (`require(var)`)
- Decorated classes (`@Component`)

---

## Project Overview
<!-- Describe your project -->
"#;

/// Template for .cursorrules - Cursor editor rules
pub const CURSORRULES_TEMPLATE: &str = r#"# Cursor Rules

## Dead Code (clrd)

Scan: npx clrd scan --format json

Auto-remove if confidence >= 0.9:
  unused_import, unused_export, unreachable_function

Ask user first:
  zombie_file, confidence < 0.9

Never remove:
  public_api: true

After removal: clean orphaned imports, delete empty files.

Skip: test files, *.d.ts, package.json entries, dynamic imports.
"#;

/// JSON Schema for LLM communication
pub const DEAD_CODE_JSON_SCHEMA: &str = r##"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ClrScanOutput",
  "description": "Output from clrd dead code scanner",
  "type": "object",
  "required": ["version", "root", "timestamp", "dead_code", "summary"],
  "properties": {
    "version": {
      "type": "string",
      "description": "clrd version that generated this output"
    },
    "root": {
      "type": "string",
      "description": "Root directory that was scanned"
    },
    "timestamp": {
      "type": "string",
      "description": "ISO 8601 timestamp of the scan"
    },
    "dead_code": {
      "type": "array",
      "items": {
        "$ref": "#/definitions/DeadCodeItem"
      }
    },
    "summary": {
      "$ref": "#/definitions/Summary"
    }
  },
  "definitions": {
    "DeadCodeItem": {
      "type": "object",
      "required": ["file_path", "span", "code_snippet", "kind", "name", "reason", "confidence"],
      "properties": {
        "file_path": { "type": "string" },
        "relative_path": { "type": "string" },
        "span": {
          "type": "object",
          "properties": {
            "start": { "type": "integer" },
            "end": { "type": "integer" }
          }
        },
        "code_snippet": { "type": "string" },
        "kind": {
          "type": "string",
          "enum": ["unused_export", "unreachable_function", "unused_variable", "unused_import", "zombie_file", "unused_type"]
        },
        "name": { "type": "string" },
        "reason": { "type": "string" },
        "confidence": {
          "type": "number",
          "minimum": 0,
          "maximum": 1
        },
        "context": {
          "type": "object",
          "properties": {
            "possibly_dynamic": { "type": "boolean" },
            "in_test_file": { "type": "boolean" },
            "public_api": { "type": "boolean" }
          }
        }
      }
    },
    "Summary": {
      "type": "object",
      "properties": {
        "unused_exports": { "type": "integer" },
        "unreachable_functions": { "type": "integer" },
        "unused_imports": { "type": "integer" },
        "zombie_files": { "type": "integer" },
        "total_issues": { "type": "integer" },
        "high_confidence_issues": { "type": "integer" }
      }
    }
  }
}"##;
