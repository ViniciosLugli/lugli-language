use lugli_ast::{Expr, Stmt};
use lugli_parser::parse;

fn parse_expr(source: &str) -> Expr {
    // Wrap in a variable declaration to get a valid statement
    let wrapped = format!("let x = {}", source);
    let (program, _span_map) = parse(&wrapped).unwrap();
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Stmt::VarDecl {
            initializer, ..
        } => initializer.as_ref().unwrap().clone(),
        _ => panic!("Expected VarDecl statement"),
    }
}

#[test]
fn test_simple_function_literal() {
    let source = "fn(x) { return x * 2 }";
    let expr = parse_expr(source);

    match expr {
        Expr::Function {
            params,
            body,
            ..
        } => {
            assert_eq!(params.len(), 1);
            assert_eq!(params[0], "x");
            assert_eq!(body.len(), 1);
        }
        _ => panic!("Expected Function expression, got {:?}", expr),
    }
}

#[test]
fn test_function_literal_no_params() {
    let source = "fn() { return 42 }";
    let expr = parse_expr(source);

    match expr {
        Expr::Function {
            params,
            body,
            ..
        } => {
            assert_eq!(params.len(), 0);
            assert_eq!(body.len(), 1);
        }
        _ => panic!("Expected Function expression, got {:?}", expr),
    }
}

#[test]
fn test_function_literal_multiple_params() {
    let source = "fn(a, b, c) { return a + b + c }";
    let expr = parse_expr(source);

    match expr {
        Expr::Function {
            params,
            body,
            ..
        } => {
            assert_eq!(params.len(), 3);
            assert_eq!(params[0], "a");
            assert_eq!(params[1], "b");
            assert_eq!(params[2], "c");
            assert_eq!(body.len(), 1);
        }
        _ => panic!("Expected Function expression, got {:?}", expr),
    }
}

#[test]
fn test_function_literal_with_type_hints() {
    let source = "fn(x: num) -> num { return x * 2 }";
    let expr = parse_expr(source);

    match expr {
        Expr::Function {
            params,
            body,
            ..
        } => {
            assert_eq!(params.len(), 1);
            assert_eq!(params[0], "x");
            assert_eq!(body.len(), 1);
        }
        _ => panic!("Expected Function expression, got {:?}", expr),
    }
}

#[test]
fn test_function_literal_in_call() {
    let source = "list.filter(fn(x) { return x > 0 })";
    let expr = parse_expr(source);

    match expr {
        Expr::Call { data, .. } => {
            assert_eq!(data.arguments.len(), 1);
            match &data.arguments[0] {
                Expr::Function {
                    params, ..
                } => {
                    assert_eq!(params.len(), 1);
                    assert_eq!(params[0], "x");
                }
                _ => panic!("Expected Function expression as argument"),
            }
        }
        _ => panic!("Expected Call expression, got {:?}", expr),
    }
}

#[test]
fn test_function_literal_multiline_body() {
    let source = "fn(x) { let doubled = x * 2; let result = doubled + 1; return result }";
    let expr = parse_expr(source);

    match expr {
        Expr::Function {
            params,
            body,
            ..
        } => {
            assert_eq!(params.len(), 1);
            assert_eq!(params[0], "x");
            assert_eq!(body.len(), 3);
        }
        _ => panic!("Expected Function expression, got {:?}", expr),
    }
}
