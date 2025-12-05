//! Templates for AI context files

/// Template for claude.md - Claude Code context file
pub const CLAUDE_MD_TEMPLATE: &str = r#"# Project Context for Claude

## Dead Code Cleanup Tool (clrd)

This project uses `clrd` for dead code detection. When the user asks you to clean up unused code, follow this workflow:

### Step 1: Scan for Dead Code
```bash
npx clrd scan --format json
```

This outputs a JSON with detected dead code:
```json
{
  "dead_code": [
    {
      "file_path": "src/utils.ts",
      "name": "unusedFunction",
      "kind": "unused_export",
      "confidence": 0.95,
      "reason": "No references found in codebase",
      "span": { "start": 10, "end": 15 },
      "code_snippet": "export function unusedFunction() { ... }"
    }
  ],
  "summary": {
    "total_issues": 5,
    "high_confidence_issues": 3
  }
}
```

### Step 2: Analyze Results
- **confidence >= 0.9**: Safe to remove (no references found)
- **confidence 0.7-0.9**: Review carefully (might be dynamically used)
- **confidence < 0.7**: Keep unless user confirms

Check for:
- Dynamic imports: `require(variable)` or `import(variable)`
- Reflection: `Object.keys()`, `eval()`, string-based access
- Public API exports that external packages might use
- Test utilities that are imported in test files

### Step 3: Remove Dead Code
For each confirmed dead code item:
1. Read the file to understand context
2. Remove the dead code (function, export, import, or entire file)
3. Remove any orphaned imports created by the removal

### Step 4: Verify Changes
```bash
# Run project's test command
npm test

# Or common alternatives
npm run test
yarn test
pnpm test
```

If tests fail, revert the problematic removal and inform the user.

### Important Notes
- Always preserve public API unless user explicitly confirms
- For `zombie_file` (unused files), confirm with user before deleting
- Group related removals (e.g., remove function + its import together)
- Run tests after each batch of removals, not at the very end

---

## Project Structure
<!-- Add your project structure description here -->

## Conventions
<!-- Document coding conventions and patterns -->
"#;

/// Template for agent.md - Universal AI agent guide
pub const AGENT_MD_TEMPLATE: &str = r#"# Agent Guide

## Dead Code Cleanup with clrd

This project uses `clrd` to detect dead code. As an AI agent, you can use this tool to help maintain code quality.

### Quick Reference

```bash
# Scan and get JSON output
npx clrd scan --format json

# Scan with specific extensions
npx clrd scan --format json --extensions ts,tsx,js

# Scan excluding test files
npx clrd scan --format json --no-tests
```

### Dead Code Types

| Kind | Description | Action |
|------|-------------|--------|
| `unused_export` | Exported but never imported | Remove export or entire function |
| `unused_import` | Imported but never used | Remove import statement |
| `unreachable_function` | Function never called | Remove function |
| `zombie_file` | File with no imports | Confirm with user, then delete |
| `unused_type` | Type/interface never referenced | Remove type definition |

### Confidence Levels

- **0.9-1.0**: Definitely unused, safe to remove
- **0.7-0.9**: Likely unused, but verify no dynamic usage
- **0.5-0.7**: Uncertain, ask user before removing
- **<0.5**: Probably used dynamically, keep it

### Workflow

1. Run `npx clrd scan --format json`
2. Filter items with confidence >= 0.8
3. For each item, remove the dead code
4. Run tests: `npm test`
5. If tests pass, continue. If fail, revert last removal.

### Edge Cases to Watch

- Re-exported modules (`export * from './module'`)
- Dynamic requires (`require(variablePath)`)
- Decorator usage (`@Injectable()`)
- Public package APIs (check package.json exports)

---

## Project Overview
<!-- Describe your project here -->

## Development Workflow
<!-- How to build, test, and deploy -->
"#;

/// Template for .cursorrules - Cursor editor rules
pub const CURSORRULES_TEMPLATE: &str = r#"# Cursor Rules

## Dead Code Cleanup (clrd)

When asked to clean up dead code or unused files:

1. Run: `npx clrd scan --format json`
2. Review the JSON output - focus on items with confidence >= 0.8
3. Remove dead code carefully, checking for dynamic usage
4. Run tests after removals: `npm test`

Dead code types:
- unused_export: Remove the export/function
- unused_import: Remove the import line
- zombie_file: Ask user before deleting entire file
- unreachable_function: Remove the function

Always run tests after cleanup to verify nothing broke.

---

# Code Style
# - Follow existing patterns in the codebase
# - Use TypeScript strict mode
# - Prefer functional patterns where appropriate

# Testing
# - Write tests for new functionality
# - Follow existing test patterns
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
