//! Templates for AI context files

/// Template for claude.md - Claude Code context file
pub const CLAUDE_MD_TEMPLATE: &str = r#"# Project Context for Claude

## Overview
This file provides context for Claude Code to understand this project.

## Project Structure
<!-- Add your project structure description here -->

## Key Files
<!-- List important files and their purposes -->

## Conventions
<!-- Document coding conventions and patterns -->

## Commands
<!-- Document common commands for this project -->

## Notes
<!-- Any additional context for AI assistance -->
"#;

/// Template for agent.md - Universal AI agent guide
pub const AGENT_MD_TEMPLATE: &str = r#"# Agent Guide

## Project Overview
This document helps AI agents understand and work with this codebase.

## Architecture
<!-- Describe the overall architecture -->

## Entry Points
<!-- List main entry points -->

## Dependencies
<!-- Key dependencies and their purposes -->

## Development Workflow
<!-- How to build, test, and deploy -->

## Code Style
<!-- Coding conventions to follow -->

## Common Tasks
<!-- Frequently needed modifications -->
"#;

/// Template for .cursorrules - Cursor editor rules
pub const CURSORRULES_TEMPLATE: &str = r#"# Cursor Rules

# Project-specific rules for Cursor AI

# Code Style
# - Follow existing patterns in the codebase
# - Use TypeScript strict mode
# - Prefer functional patterns where appropriate

# Testing
# - Write tests for new functionality
# - Follow existing test patterns

# Documentation
# - Add JSDoc comments for public APIs
# - Update README for significant changes
"#;

/// JSON Schema for LLM communication
pub const DEAD_CODE_JSON_SCHEMA: &str = r##"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ClrScanOutput",
  "description": "Output from clr dead code scanner",
  "type": "object",
  "required": ["version", "root", "timestamp", "dead_code", "summary"],
  "properties": {
    "version": {
      "type": "string",
      "description": "clr version that generated this output"
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
