//! AST Analyzer - Oxc-based JavaScript/TypeScript analysis
//!
//! Extracts exports, imports, and internal references from source files
//! using the ultra-fast Oxc parser.

use crate::types::*;
use anyhow::{Context, Result};
use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::fs;
use std::path::Path;

/// Analyzes a single source file and extracts reference information
pub struct AstAnalyzer;

impl AstAnalyzer {
    /// Analyze a file and return its reference node
    pub fn analyze_file(path: &Path) -> Result<ReferenceNode> {
        let source = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {:?}", path))?;

        Self::analyze_source(path, &source)
    }

    /// Analyze source code directly
    pub fn analyze_source(path: &Path, source: &str) -> Result<ReferenceNode> {
        let allocator = Allocator::default();
        let source_type = Self::get_source_type(path);

        let parser = Parser::new(&allocator, source, source_type);
        let result = parser.parse();

        if !result.errors.is_empty() {
            tracing::debug!(
                "Parse errors in {:?}: {:?}",
                path,
                result
                    .errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
            );
        }

        let mut exports = Vec::new();
        let mut imports = Vec::new();
        let internal_refs = Vec::new();

        // Process statements directly
        for stmt in &result.program.body {
            Self::process_statement(stmt, source, &mut exports, &mut imports);
        }

        Ok(ReferenceNode {
            file_path: path.to_path_buf(),
            exports,
            imports,
            internal_refs,
        })
    }

    fn process_statement(
        stmt: &Statement,
        source: &str,
        exports: &mut Vec<ExportedSymbol>,
        imports: &mut Vec<ImportedSymbol>,
    ) {
        match stmt {
            Statement::ImportDeclaration(decl) => {
                Self::process_import(decl, source, imports);
            }
            Statement::ExportNamedDeclaration(decl) => {
                Self::process_export_named(decl, source, exports);
            }
            Statement::ExportDefaultDeclaration(decl) => {
                Self::process_export_default(decl, source, exports);
            }
            Statement::ExportAllDeclaration(decl) => {
                Self::process_export_all(decl, source, exports);
            }
            _ => {}
        }
    }

    fn process_import(decl: &ImportDeclaration, source: &str, imports: &mut Vec<ImportedSymbol>) {
        let import_source = decl.source.value.to_string();
        let is_type_only = decl.import_kind.is_type();
        let span = Self::span_to_code_span(decl.span, source);

        if let Some(specifiers) = &decl.specifiers {
            for spec in specifiers {
                match spec {
                    ImportDeclarationSpecifier::ImportSpecifier(s) => {
                        imports.push(ImportedSymbol {
                            name: s.imported.name().to_string(),
                            alias: if s.local.name != s.imported.name() {
                                Some(s.local.name.to_string())
                            } else {
                                None
                            },
                            source: import_source.clone(),
                            is_type_only: is_type_only || s.import_kind.is_type(),
                            span,
                        });
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                        imports.push(ImportedSymbol {
                            name: "default".to_string(),
                            alias: Some(s.local.name.to_string()),
                            source: import_source.clone(),
                            is_type_only,
                            span,
                        });
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                        imports.push(ImportedSymbol {
                            name: "*".to_string(),
                            alias: Some(s.local.name.to_string()),
                            source: import_source.clone(),
                            is_type_only,
                            span,
                        });
                    }
                }
            }
        }
    }

    fn process_export_named(
        decl: &ExportNamedDeclaration,
        source: &str,
        exports: &mut Vec<ExportedSymbol>,
    ) {
        let span = Self::span_to_code_span(decl.span, source);
        let is_reexport = decl.source.is_some();

        // Handle export specifiers: export { foo, bar }
        for spec in &decl.specifiers {
            exports.push(ExportedSymbol {
                name: spec.exported.name().to_string(),
                kind: SymbolKind::Variable,
                span,
                is_default: false,
                is_reexport,
            });
        }

        // Handle declaration exports: export const foo = ...
        if let Some(declaration) = &decl.declaration {
            match declaration {
                Declaration::VariableDeclaration(var_decl) => {
                    for declarator in &var_decl.declarations {
                        if let Some(name) = Self::get_binding_name(&declarator.id) {
                            let kind = match var_decl.kind {
                                VariableDeclarationKind::Const => SymbolKind::Const,
                                VariableDeclarationKind::Let => SymbolKind::Let,
                                _ => SymbolKind::Variable,
                            };
                            exports.push(ExportedSymbol {
                                name,
                                kind,
                                span,
                                is_default: false,
                                is_reexport: false,
                            });
                        }
                    }
                }
                Declaration::FunctionDeclaration(func) => {
                    if let Some(id) = &func.id {
                        exports.push(ExportedSymbol {
                            name: id.name.to_string(),
                            kind: SymbolKind::Function,
                            span,
                            is_default: false,
                            is_reexport: false,
                        });
                    }
                }
                Declaration::ClassDeclaration(class) => {
                    if let Some(id) = &class.id {
                        exports.push(ExportedSymbol {
                            name: id.name.to_string(),
                            kind: SymbolKind::Class,
                            span,
                            is_default: false,
                            is_reexport: false,
                        });
                    }
                }
                Declaration::TSTypeAliasDeclaration(type_alias) => {
                    exports.push(ExportedSymbol {
                        name: type_alias.id.name.to_string(),
                        kind: SymbolKind::Type,
                        span,
                        is_default: false,
                        is_reexport: false,
                    });
                }
                Declaration::TSInterfaceDeclaration(interface) => {
                    exports.push(ExportedSymbol {
                        name: interface.id.name.to_string(),
                        kind: SymbolKind::Interface,
                        span,
                        is_default: false,
                        is_reexport: false,
                    });
                }
                Declaration::TSEnumDeclaration(enum_decl) => {
                    exports.push(ExportedSymbol {
                        name: enum_decl.id.name.to_string(),
                        kind: SymbolKind::Enum,
                        span,
                        is_default: false,
                        is_reexport: false,
                    });
                }
                _ => {}
            }
        }
    }

    fn process_export_default(
        decl: &ExportDefaultDeclaration,
        source: &str,
        exports: &mut Vec<ExportedSymbol>,
    ) {
        let span = Self::span_to_code_span(decl.span, source);

        let (name, kind) = match &decl.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(func) => (
                func.id
                    .as_ref()
                    .map(|id| id.name.to_string())
                    .unwrap_or_else(|| "default".to_string()),
                SymbolKind::Function,
            ),
            ExportDefaultDeclarationKind::ClassDeclaration(class) => (
                class
                    .id
                    .as_ref()
                    .map(|id| id.name.to_string())
                    .unwrap_or_else(|| "default".to_string()),
                SymbolKind::Class,
            ),
            _ => ("default".to_string(), SymbolKind::Variable),
        };

        exports.push(ExportedSymbol {
            name,
            kind,
            span,
            is_default: true,
            is_reexport: false,
        });
    }

    fn process_export_all(
        decl: &ExportAllDeclaration,
        source: &str,
        exports: &mut Vec<ExportedSymbol>,
    ) {
        let span = Self::span_to_code_span(decl.span, source);

        exports.push(ExportedSymbol {
            name: "*".to_string(),
            kind: SymbolKind::Variable,
            span,
            is_default: false,
            is_reexport: true,
        });
    }

    fn span_to_code_span(span: oxc_span::Span, source: &str) -> CodeSpan {
        let start_line = source[..span.start as usize]
            .chars()
            .filter(|&c| c == '\n')
            .count() as u32
            + 1;
        let end_line = source[..span.end as usize]
            .chars()
            .filter(|&c| c == '\n')
            .count() as u32
            + 1;

        CodeSpan {
            start: start_line,
            end: end_line,
            col_start: 0,
            col_end: 0,
        }
    }

    fn get_source_type(path: &Path) -> SourceType {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match ext {
            "ts" | "mts" => SourceType::ts(),
            "tsx" => SourceType::tsx(),
            "cts" => SourceType::cjs().with_typescript(true),
            "jsx" => SourceType::jsx(),
            "mjs" => SourceType::mjs(),
            "cjs" => SourceType::cjs(),
            _ => SourceType::mjs(),
        }
    }

    fn get_binding_name(pattern: &BindingPattern) -> Option<String> {
        match &pattern.kind {
            BindingPatternKind::BindingIdentifier(id) => Some(id.name.to_string()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_analyze_simple_export() {
        let source = r#"
            export const foo = 42;
            export function bar() {}
            export class Baz {}
        "#;

        let path = PathBuf::from("test.ts");
        let result = AstAnalyzer::analyze_source(&path, source).unwrap();

        assert_eq!(result.exports.len(), 3);
        assert!(result.exports.iter().any(|e| e.name == "foo"));
        assert!(result.exports.iter().any(|e| e.name == "bar"));
        assert!(result.exports.iter().any(|e| e.name == "Baz"));
    }

    #[test]
    fn test_analyze_imports() {
        let source = r#"
            import { foo, bar as baz } from './module';
            import Default from './default';
            import * as All from './all';
        "#;

        let path = PathBuf::from("test.ts");
        let result = AstAnalyzer::analyze_source(&path, source).unwrap();

        assert_eq!(result.imports.len(), 4);
    }
}
