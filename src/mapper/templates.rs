//! Templates for AI context files

/// Template for clrd.md - Main usage instructions for AI agents
pub const CLRD_MD_TEMPLATE: &str = r#"# clrd - Dead Code Detection

Usage instructions for AI agents (Claude, Cursor, Copilot, etc.)

## Quick Start

```bash
npx clrd scan --format json
```

## Decision Rules

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

## Quick Reference

| confidence | kind | action |
|------------|------|--------|
| >= 0.9 | unused_import | Auto-remove |
| >= 0.9 | unused_export | Auto-remove |
| >= 0.9 | unreachable_function | Auto-remove |
| >= 0.9 | zombie_file | Ask user |
| < 0.9 | * | Ask user |
| * | public_api: true | Keep |

## Removal Process

1. Remove the dead code itself
2. Remove imports that become orphaned
3. Delete files that become empty
4. Group related removals (function + its imports)

## Common False Positives

Skip these patterns - they may appear unused but aren't:
- `__tests__/`, `*.test.ts`, `*.spec.ts` - test utilities
- `*.d.ts` - type declarations
- Files with `@public` or `@api` JSDoc tags
- Entry points in package.json (`main`, `exports`, `bin`)
- Dynamic imports: `require(variable)`, `import(expr)`
- Decorator factories: `@Injectable()`, `@Component()`

## Output Format

After cleanup, summarize:
```
Removed X items:
- 3 unused imports
- 2 unused exports
- 1 unreachable function

Skipped Y items (need confirmation):
- src/legacy.ts (zombie file)
```

## Links

- [npm](https://npmjs.com/package/clrd)
- [GitHub](https://github.com/nicola-dc/clrd)
"#;

/// Reference line to add to claude.md
pub const CLAUDE_MD_REFERENCE: &str = r#"
## Dead Code Detection

See [clrd.md](./clrd.md) for dead code cleanup instructions.
"#;

/// Reference line to add to agent.md
pub const AGENT_MD_REFERENCE: &str = r#"
## Dead Code Detection

See [clrd.md](./clrd.md) for dead code cleanup instructions.
"#;

/// Reference line to add to .cursorrules
pub const CURSORRULES_REFERENCE: &str = r#"
# Dead Code Detection
# See clrd.md for dead code cleanup instructions.
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
