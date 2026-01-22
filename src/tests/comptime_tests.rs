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
fn test_comptime_constant_evaluation() {
    let source = r#"
def main
  x = comptime 2 + 3
  y = comptime 10 * 5
  z = comptime 100 / 4
end
"#;
    let (_ast, reporter) = analyze_source(source);
    assert!(!reporter.has_errors());
}

#[test]
fn test_comptime_complex_expression() {
    let source = r#"
def main
  result = comptime (10 + 5) * 3 - 7
  nested = comptime 2 * (3 + comptime 4 * 5)
end
"#;
    let (_ast, reporter) = analyze_source(source);
    assert!(!reporter.has_errors());
}

#[test]
fn test_comptime_comparison() {
    let source = r#"
def main
  is_greater = comptime 10 > 5
  is_equal = comptime 5 == 5
  is_less = comptime 3 < 7
end
"#;
    let (_ast, reporter) = analyze_source(source);
    assert!(!reporter.has_errors());
}

#[test]
fn test_comptime_variable() {
    let source = r#"
def main
  x = comptime 10
  y = comptime 5 + 3
end
"#;
    let (_ast, reporter) = analyze_source(source);
    assert!(!reporter.has_errors());
}

#[test]
fn test_comptime_if_condition() {
    let source = r#"
def main
  if comptime 2 + 2 == 4
    x = 10
  end
end
"#;
    let (_ast, reporter) = analyze_source(source);
    assert!(!reporter.has_errors());
}

#[test]
fn test_comptime_non_constant_error() {
    let source = r#"
def main
  x = 10
  y = comptime x + 5
end
"#;
    let (_ast, reporter) = analyze_source(source);
    assert!(reporter.has_errors());
}
