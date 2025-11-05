use lugli_ast::Stmt;
use lugli_parser::Parser;

#[test]
fn test_simple_import() {
    let source = "import math";
    let mut parser = Parser::new(source).expect("Parser creation should succeed");
    let (program, _span_map) = parser.parse().expect("Parse should succeed");

    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Stmt::Import {
            module_path,
            items,
            alias,
            ..
        } => {
            assert_eq!(module_path, &vec!["math".to_string()]);
            assert_eq!(*items, None);
            assert_eq!(*alias, None);
        }
        _ => panic!("Expected Import statement"),
    }
}

#[test]
fn test_import_with_dots() {
    let source = "import stdlib.collections.list";
    let mut parser = Parser::new(source).expect("Parser creation should succeed");
    let (program, _span_map) = parser.parse().expect("Parse should succeed");

    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Stmt::Import {
            module_path, ..
        } => {
            assert_eq!(module_path, &vec!["stdlib".to_string(), "collections".to_string(), "list".to_string()]);
        }
        _ => panic!("Expected Import statement"),
    }
}

#[test]
fn test_import_with_alias() {
    let source = "import math as m";
    let mut parser = Parser::new(source).expect("Parser creation should succeed");
    let (program, _span_map) = parser.parse().expect("Parse should succeed");

    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Stmt::Import {
            module_path,
            alias,
            ..
        } => {
            assert_eq!(module_path, &vec!["math".to_string()]);
            assert_eq!(alias, &Some("m".to_string()));
        }
        _ => panic!("Expected Import statement"),
    }
}

#[test]
fn test_from_import_single_item() {
    let source = "from math import pi";
    let mut parser = Parser::new(source).expect("Parser creation should succeed");
    let (program, _span_map) = parser.parse().expect("Parse should succeed");

    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Stmt::Import {
            module_path,
            items,
            ..
        } => {
            assert_eq!(module_path, &vec!["math".to_string()]);
            assert_eq!(items, &Some(vec!["pi".to_string()]));
        }
        _ => panic!("Expected Import statement"),
    }
}

#[test]
fn test_from_import_multiple_items() {
    let source = "from math import pi, sqrt, cos";
    let mut parser = Parser::new(source).expect("Parser creation should succeed");
    let (program, _span_map) = parser.parse().expect("Parse should succeed");

    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Stmt::Import {
            module_path,
            items,
            ..
        } => {
            assert_eq!(module_path, &vec!["math".to_string()]);
            assert_eq!(items, &Some(vec!["pi".to_string(), "sqrt".to_string(), "cos".to_string()]));
        }
        _ => panic!("Expected Import statement"),
    }
}

#[test]
fn test_from_import_wildcard() {
    let source = "from math import *";
    let mut parser = Parser::new(source).expect("Parser creation should succeed");
    let (program, _span_map) = parser.parse().expect("Parse should succeed");

    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Stmt::Import {
            module_path,
            items,
            ..
        } => {
            assert_eq!(module_path, &vec!["math".to_string()]);
            assert_eq!(*items, None); // Wildcard represented as None
        }
        _ => panic!("Expected Import statement"),
    }
}

#[test]
fn test_export_function() {
    let source = "export fn add(a, b) { return a + b }";
    let mut parser = Parser::new(source).expect("Parser creation should succeed");
    let (program, _span_map) = parser.parse().expect("Parse should succeed");

    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Stmt::Export {
            item, ..
        } => match &**item {
            Stmt::FnDecl {
                name,
                params,
                ..
            } => {
                assert_eq!(name, "add");
                let param_names: Vec<String> = params.iter().map(|(n, _)| n.clone()).collect();
                assert_eq!(param_names, vec!["a".to_string(), "b".to_string()]);
            }
            _ => panic!("Expected FnDecl inside Export"),
        },
        _ => panic!("Expected Export statement"),
    }
}

#[test]
fn test_export_variable() {
    let source = "export let x = 42";
    let mut parser = Parser::new(source).expect("Parser creation should succeed");
    let (program, _span_map) = parser.parse().expect("Parse should succeed");

    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Stmt::Export {
            item, ..
        } => match &**item {
            Stmt::VarDecl {
                name, ..
            } => {
                assert_eq!(name, "x");
            }
            _ => panic!("Expected VarDecl inside Export"),
        },
        _ => panic!("Expected Export statement"),
    }
}

#[test]
fn test_multiple_imports() {
    let source = r#"
        import math
        from collections import List, Dict
        import io as stdio
    "#;
    let mut parser = Parser::new(source).expect("Parser creation should succeed");
    let (program, _span_map) = parser.parse().expect("Parse should succeed");

    assert_eq!(program.statements.len(), 3);

    // First import
    match &program.statements[0] {
        Stmt::Import {
            module_path,
            items,
            alias,
            ..
        } => {
            assert_eq!(module_path, &vec!["math".to_string()]);
            assert_eq!(*items, None);
            assert_eq!(*alias, None);
        }
        _ => panic!("Expected Import statement"),
    }

    // Second import (from)
    match &program.statements[1] {
        Stmt::Import {
            module_path,
            items,
            ..
        } => {
            assert_eq!(module_path, &vec!["collections".to_string()]);
            assert_eq!(items, &Some(vec!["List".to_string(), "Dict".to_string()]));
        }
        _ => panic!("Expected Import statement"),
    }

    // Third import (with alias)
    match &program.statements[2] {
        Stmt::Import {
            module_path,
            alias,
            ..
        } => {
            assert_eq!(module_path, &vec!["io".to_string()]);
            assert_eq!(alias, &Some("stdio".to_string()));
        }
        _ => panic!("Expected Import statement"),
    }
}
