use crate::error::Reporter;
use crate::frontend::lexer::Lexer;
use crate::frontend::parser::Parser;
use crate::frontend::semantic::SemanticAnalyzer;
use codespan::Files;

fn analyze_source(source: &str) -> (crate::core::ast::Ast, Reporter) {
    let mut files = Files::new();
    let file_id = files.add("test.em", source.to_string());
    let mut reporter = Reporter::new();
    let source_str = files.source(file_id).to_string();
    let mut lexer = Lexer::new(&source_str, file_id, &mut reporter);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens, file_id, &mut reporter);
    let ast = parser.parse();
    
    if !reporter.has_errors() {
        let mut analyzer = SemanticAnalyzer::new(&mut reporter, file_id);
        analyzer.analyze(&ast);
    }
    
    if reporter.has_errors() {
        for diag in reporter.diagnostics() {
            eprintln!("[{:?}] {:?}: {}", diag.kind, diag.severity, diag.message);
        }
    }
    
    (ast, reporter)
}

#[test]
fn test_pointer_operations() {
    let source = r#"
def main
  x : int = 10
  ptr : ref int = @x
  value : int = ptr.value
  ptr.value = 20
end
"#;
    let (_ast, reporter) = analyze_source(source);
    assert!(!reporter.has_errors());
}

#[test]
fn test_nullable_pointer() {
    let source = r#"
def main
  ptr : ref? int = null
  if ptr.exists?
    value : int = ptr.value
    ptr.value = 10
  end
end
"#;
    let (_ast, reporter) = analyze_source(source);
    assert!(!reporter.has_errors());
}

#[test]
fn test_pointer_assignment() {
    let source = r#"
def main
  x : int = 10
  y : int = 20
  ptr : ref int = @x
  ptr.value = y
end
"#;
    let (_ast, reporter) = analyze_source(source);
    assert!(!reporter.has_errors());
}

#[test]
fn test_mutability_tracking() {
    let source = r#"
def main
  x : int = 10
  x = 20
  x = x + 5
end
"#;
    let (_ast, reporter) = analyze_source(source);
    assert!(!reporter.has_errors());
}

#[test]
fn test_immutable_reassignment_error() {
    let source = r#"
def main
  let x : int = 10
  x = 20
end
"#;
    let (_ast, reporter) = analyze_source(source);
    assert!(reporter.has_errors());
}
