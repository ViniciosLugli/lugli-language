use lugli_ast::{Expr, LiteralValue, Stmt};
use lugli_parser::Parser;

fn parse_expr(source: &str) -> Expr {
    let mut parser = Parser::new(source).unwrap();
    let (program, _span_map) = parser.parse().unwrap();
    match &program.statements[0] {
        Stmt::Expression {
            expr, ..
        } => expr.clone(),
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_basic_list_comprehension() {
    let source = "[x for x in numbers]";
    let expr = parse_expr(source);

    if let Expr::ListComprehension {
        data, ..
    } = expr
    {
        assert!(matches!(data.element, Expr::Identifier { .. }));
        assert_eq!(data.variable, "x");
        assert!(matches!(data.iterable, Expr::Identifier { .. }));
        assert!(data.condition.is_none());
    } else {
        panic!("Expected list comprehension, got: {:?}", expr);
    }
}

#[test]
fn test_list_comprehension_with_expression() {
    let source = "[x * 2 for x in numbers]";
    let expr = parse_expr(source);

    if let Expr::ListComprehension {
        data, ..
    } = expr
    {
        assert!(matches!(data.element, Expr::Binary { .. }));
        assert_eq!(data.variable, "x");
        assert!(matches!(data.iterable, Expr::Identifier { .. }));
        assert!(data.condition.is_none());
    } else {
        panic!("Expected list comprehension, got: {:?}", expr);
    }
}

#[test]
fn test_list_comprehension_with_condition() {
    let source = "[x for x in numbers if x > 0]";
    let expr = parse_expr(source);

    if let Expr::ListComprehension {
        data, ..
    } = expr
    {
        assert!(matches!(data.element, Expr::Identifier { .. }));
        assert_eq!(data.variable, "x");
        assert!(matches!(data.iterable, Expr::Identifier { .. }));
        assert!(data.condition.is_some());
        if let Some(ref cond) = data.condition {
            assert!(matches!(*cond, Expr::Binary { .. }));
        }
    } else {
        panic!("Expected list comprehension, got: {:?}", expr);
    }
}

#[test]
fn test_list_comprehension_complex_expression() {
    let source = "[x * x + 1 for x in numbers if x % 2 == 0]";
    let expr = parse_expr(source);

    if let Expr::ListComprehension {
        data, ..
    } = expr
    {
        assert!(matches!(data.element, Expr::Binary { .. }));
        assert_eq!(data.variable, "x");
        assert!(matches!(data.iterable, Expr::Identifier { .. }));
        assert!(data.condition.is_some());
    } else {
        panic!("Expected list comprehension, got: {:?}", expr);
    }
}

#[test]
fn test_list_comprehension_list_literal() {
    let source = "[x for x in [1, 2, 3]]";
    let expr = parse_expr(source);

    if let Expr::ListComprehension {
        data, ..
    } = expr
    {
        assert!(matches!(data.element, Expr::Identifier { .. }));
        assert_eq!(data.variable, "x");
        assert!(matches!(data.iterable, Expr::List { .. }));
        assert!(data.condition.is_none());
    } else {
        panic!("Expected list comprehension, got: {:?}", expr);
    }
}

#[test]
fn test_list_comprehension_method_call() {
    let source = "[x.upper() for x in names if x.len() > 3]";
    let expr = parse_expr(source);

    if let Expr::ListComprehension {
        data, ..
    } = expr
    {
        assert!(matches!(data.element, Expr::Call { .. }));
        assert_eq!(data.variable, "x");
        assert!(matches!(data.iterable, Expr::Identifier { .. }));
        assert!(data.condition.is_some());
    } else {
        panic!("Expected list comprehension, got: {:?}", expr);
    }
}

#[test]
fn test_regular_list_still_works() {
    let source = "[1, 2, 3]";
    let expr = parse_expr(source);

    if let Expr::List {
        elements, ..
    } = expr
    {
        assert_eq!(elements.len(), 3);
        for (i, elem) in elements.iter().enumerate() {
            if let Expr::Literal {
                value: LiteralValue::Number(n), ..
            } = elem
            {
                assert_eq!(*n, (i + 1) as f64);
            } else {
                panic!("Expected number literal");
            }
        }
    } else {
        panic!("Expected list literal, got: {:?}", expr);
    }
}

#[test]
fn test_empty_list() {
    let source = "[]";
    let expr = parse_expr(source);

    if let Expr::List {
        elements, ..
    } = expr
    {
        assert_eq!(elements.len(), 0);
    } else {
        panic!("Expected empty list, got: {:?}", expr);
    }
}
