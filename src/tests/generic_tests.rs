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
fn test_generic_struct_instantiation() {
    let source = r#"
struct List [ Type T ]
  data : ref T
  size : int
end

def main
  int_list : List[int]
  float_list : List[float]
end
"#;
    let (_ast, reporter) = analyze_source(source);
    assert!(!reporter.has_errors());
}

#[test]
fn test_generic_function_instantiation() {
    let source = r#"
def identity [ Type T ](x : T) returns T
  return x
end

def main
  a : int = identity(10)
  b : float = identity(3.14)
end
"#;
    let (_ast, reporter) = analyze_source(source);
    assert!(!reporter.has_errors());
}

#[test]
fn test_generic_with_constraints() {
    let source = r#"
trait Addable
  def add(self) returns int
end

def sum [ Type T ](a : T, b : T) returns int
  return 0
end
"#;
    let (_ast, reporter) = analyze_source(source);
    assert!(!reporter.has_errors());
}

#[test]
fn test_nested_generics() {
    let source = r#"
struct Pair [ Type A, Type B ]
  first : A
  second : B
end

struct Container [ Type T ]
  items : Pair[T, int]
end

def main
  c : Container[string]
end
"#;
    let (_ast, reporter) = analyze_source(source);
    assert!(!reporter.has_errors());
}

#[test]
fn test_generic_type_mismatch() {
    let source = r#"
def identity [ Type T ](x : T) returns T
  return x
end

def main
  a : int = identity(3.14)
end
"#;
    let (_ast, reporter) = analyze_source(source);
    assert!(reporter.has_errors());
}
