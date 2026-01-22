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
fn test_foreign_function_declaration() {
    let source = r#"
foreign "C" libc
  def printf(format : ref char) returns int
end

def main
  result : int = printf(null)
end
"#;
    let (_ast, reporter) = analyze_source(source);
    assert!(!reporter.has_errors());
}

#[test]
fn test_foreign_with_primitive_types() {
    let source = r#"
foreign "C" math
  def sin(x : float) returns float
  def cos(x : float) returns float
  def sqrt(x : float) returns float
end

def main
  s : float = sin(3.14)
  c : float = cos(1.57)
end
"#;
    let (_ast, reporter) = analyze_source(source);
    assert!(!reporter.has_errors());
}

#[test]
fn test_foreign_with_pointers() {
    let source = r#"
foreign "C" string
  def strlen(s : ref char) returns int
end

def main
  len : int = strlen(null)
end
"#;
    let (_ast, reporter) = analyze_source(source);
    assert!(!reporter.has_errors());
}

#[test]
fn test_foreign_c_incompatible_type() {
    let source = r#"
foreign "C" test
  def bad_func(x : string) returns int
end
"#;
    let (_ast, reporter) = analyze_source(source);
    assert!(reporter.has_errors());
}

#[test]
fn test_foreign_variadic_function() {
    let source = r#"
foreign "C" stdio
  def printf(format : ref char) returns int
  def sprintf(buffer : ref char, format : ref char) returns int
end

def main
  result : int = printf(null)
end
"#;
    let (_ast, reporter) = analyze_source(source);
    assert!(!reporter.has_errors());
}
