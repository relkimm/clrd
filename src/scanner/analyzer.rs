//! AST Analyzer - Oxc-based JavaScript/TypeScript analysis
//!
//! Extracts exports, imports, and internal references from source files
//! using the ultra-fast Oxc parser.

use crate::types::*;
use anyhow::{Context, Result};
use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast::visit::walk;
use oxc_ast::Visit;
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

        let mut visitor = ReferenceVisitor::new(path, source);
        visitor.visit_program(&result.program);

        Ok(ReferenceNode {
            file_path: path.to_path_buf(),
            exports: visitor.exports,
            imports: visitor.imports,
            internal_refs: visitor.internal_refs,
        })
    }

    fn get_source_type(path: &Path) -> SourceType {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match ext {
            "ts" => SourceType::ts(),
            "tsx" => SourceType::tsx(),
            "mts" => SourceType::mts(),
            "cts" => SourceType::cts(),
            "jsx" => SourceType::jsx(),
            "mjs" => SourceType::mjs(),
            "cjs" => SourceType::cjs(),
            _ => SourceType::mjs(), // Default to ESM
        }
    }
}

/// Visitor that collects exports, imports, and references
struct ReferenceVisitor<'a> {
    path: &'a Path,
    source: &'a str,
    exports: Vec<ExportedSymbol>,
    imports: Vec<ImportedSymbol>,
    internal_refs: Vec<String>,
}

impl<'a> ReferenceVisitor<'a> {
    fn new(path: &'a Path, source: &'a str) -> Self {
        Self {
            path,
            source,
            exports: Vec::new(),
            imports: Vec::new(),
            internal_refs: Vec::new(),
        }
    }

    fn span_to_code_span(&self, span: oxc_span::Span) -> CodeSpan {
        let start_line = self.source[..span.start as usize]
            .chars()
            .filter(|&c| c == '\n')
            .count() as u32
            + 1;
        let end_line = self.source[..span.end as usize]
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
}

impl<'a> Visit<'a> for ReferenceVisitor<'a> {
    fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        let source = decl.source.value.to_string();
        let is_type_only = decl.import_kind.is_type();
        let span = self.span_to_code_span(decl.span);

        if let Some(specifiers) = &decl.specifiers {
            for spec in specifiers {
                match spec {
                    ImportDeclarationSpecifier::ImportSpecifier(s) => {
                        self.imports.push(ImportedSymbol {
                            name: s.imported.name().to_string(),
                            alias: if s.local.name != s.imported.name() {
                                Some(s.local.name.to_string())
                            } else {
                                None
                            },
                            source: source.clone(),
                            is_type_only: is_type_only || s.import_kind.is_type(),
                            span,
                        });
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                        self.imports.push(ImportedSymbol {
                            name: "default".to_string(),
                            alias: Some(s.local.name.to_string()),
                            source: source.clone(),
                            is_type_only,
                            span,
                        });
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                        self.imports.push(ImportedSymbol {
                            name: "*".to_string(),
                            alias: Some(s.local.name.to_string()),
                            source: source.clone(),
                            is_type_only,
                            span,
                        });
                    }
                }
            }
        }

        walk::walk_import_declaration(self, decl);
    }

    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        let span = self.span_to_code_span(decl.span);
        let is_reexport = decl.source.is_some();

        // Handle export specifiers: export { foo, bar }
        for spec in &decl.specifiers {
            self.exports.push(ExportedSymbol {
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
                        if let Some(name) = self.get_binding_name(&declarator.id) {
                            let kind = match var_decl.kind {
                                VariableDeclarationKind::Const => SymbolKind::Const,
                                VariableDeclarationKind::Let => SymbolKind::Let,
                                _ => SymbolKind::Variable,
                            };
                            self.exports.push(ExportedSymbol {
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
                        self.exports.push(ExportedSymbol {
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
                        self.exports.push(ExportedSymbol {
                            name: id.name.to_string(),
                            kind: SymbolKind::Class,
                            span,
                            is_default: false,
                            is_reexport: false,
                        });
                    }
                }
                Declaration::TSTypeAliasDeclaration(type_alias) => {
                    self.exports.push(ExportedSymbol {
                        name: type_alias.id.name.to_string(),
                        kind: SymbolKind::Type,
                        span,
                        is_default: false,
                        is_reexport: false,
                    });
                }
                Declaration::TSInterfaceDeclaration(interface) => {
                    self.exports.push(ExportedSymbol {
                        name: interface.id.name.to_string(),
                        kind: SymbolKind::Interface,
                        span,
                        is_default: false,
                        is_reexport: false,
                    });
                }
                Declaration::TSEnumDeclaration(enum_decl) => {
                    self.exports.push(ExportedSymbol {
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

        walk::walk_export_named_declaration(self, decl);
    }

    fn visit_export_default_declaration(&mut self, decl: &ExportDefaultDeclaration<'a>) {
        let span = self.span_to_code_span(decl.span);

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

        self.exports.push(ExportedSymbol {
            name,
            kind,
            span,
            is_default: true,
            is_reexport: false,
        });

        walk::walk_export_default_declaration(self, decl);
    }

    fn visit_export_all_declaration(&mut self, decl: &ExportAllDeclaration<'a>) {
        let span = self.span_to_code_span(decl.span);

        // export * from 'module'
        self.exports.push(ExportedSymbol {
            name: "*".to_string(),
            kind: SymbolKind::Variable,
            span,
            is_default: false,
            is_reexport: true,
        });

        walk::walk_export_all_declaration(self, decl);
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        self.internal_refs.push(ident.name.to_string());
        walk::walk_identifier_reference(self, ident);
    }
}

impl<'a> ReferenceVisitor<'a> {
    fn get_binding_name(&self, pattern: &BindingPattern<'a>) -> Option<String> {
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
