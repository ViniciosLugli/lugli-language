//! Tests for context-aware keyword parsing
//! Keywords can be used as method names and property names in specific contexts

use lugli_parser::parse;

#[test]
fn test_from_as_method_name() {
    let source = r#"
        struct Builder {
            fn from(self, x) {
                return self
            }
        }
        let b = Builder {}
        b.from(10)
    "#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Should parse 'from' as method name: {:?}", result.err());
}

#[test]
fn test_where_as_method_name() {
    let source = r#"
        struct Query {
            fn where(self, condition) {
                return self
            }
        }
        let q = Query {}
        q.where("x > 5")
    "#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Should parse 'where' as method name: {:?}", result.err());
}

#[test]
fn test_match_as_method_name() {
    let source = r#"
        struct Router {
            fn match(self, pattern) {
                return pattern
            }
        }
        let r = Router {}
        r.match("/users")
    "#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Should parse 'match' as method name: {:?}", result.err());
}

#[test]
fn test_if_as_method_name() {
    let source = r#"
        struct Conditional {
            fn if(self, cond) {
                return cond
            }
        }
        let c = Conditional {}
        c.if(true)
    "#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Should parse 'if' as method name: {:?}", result.err());
}

#[test]
fn test_for_as_method_name() {
    let source = r#"
        struct Iterator {
            fn for(self, item) {
                return item
            }
        }
        let i = Iterator {}
        i.for(42)
    "#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Should parse 'for' as method name: {:?}", result.err());
}

#[test]
fn test_while_as_method_name() {
    let source = r#"
        struct Looper {
            fn while(self, condition) {
                return condition
            }
        }
        let l = Looper {}
        l.while(true)
    "#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Should parse 'while' as method name: {:?}", result.err());
}

#[test]
fn test_loop_as_method_name() {
    let source = r#"
        struct Repeater {
            fn loop(self, times) {
                return times
            }
        }
        let r = Repeater {}
        r.loop(5)
    "#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Should parse 'loop' as method name: {:?}", result.err());
}

#[test]
fn test_return_as_method_name() {
    let source = r#"
        struct Returner {
            fn return(self, value) {
                return value
            }
        }
        let r = Returner {}
        r.return(100)
    "#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Should parse 'return' as method name: {:?}", result.err());
}

#[test]
fn test_import_as_method_name() {
    let source = r#"
        struct Importer {
            fn import(self, module) {
                return module
            }
        }
        let i = Importer {}
        i.import("math")
    "#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Should parse 'import' as method name: {:?}", result.err());
}

#[test]
fn test_export_as_method_name() {
    let source = r#"
        struct Exporter {
            fn export(self, data) {
                return data
            }
        }
        let e = Exporter {}
        e.export("result")
    "#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Should parse 'export' as method name: {:?}", result.err());
}

#[test]
fn test_async_as_method_name() {
    let source = r#"
        struct AsyncOp {
            fn async(self, func) {
                return func
            }
        }
        let a = AsyncOp {}
        a.async(null)
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Should parse 'async' as method name: {:?}", result.err());
}

#[test]
fn test_await_as_method_name() {
    let source = r#"
        struct Promise {
            fn await(self) {
                return null
            }
        }
        let p = Promise {}
        p.await()
    "#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Should parse 'await' as method name: {:?}", result.err());
}

#[test]
fn test_try_as_method_name() {
    let source = r#"
        struct ErrorHandler {
            fn try(self, operation) {
                return operation
            }
        }
        let e = ErrorHandler {}
        e.try(fn() { return 1 })
    "#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Should parse 'try' as method name: {:?}", result.err());
}

#[test]
fn test_catch_as_method_name() {
    let source = r#"
        struct ErrorHandler {
            fn catch(self, handler) {
                return handler
            }
        }
        let e = ErrorHandler {}
        e.catch(fn() { return 0 })
    "#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Should parse 'catch' as method name: {:?}", result.err());
}

#[test]
fn test_struct_as_method_name() {
    let source = r#"
        struct Builder {
            fn struct(self, name) {
                return name
            }
        }
        let b = Builder {}
        b.struct("User")
    "#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Should parse 'struct' as method name: {:?}", result.err());
}

#[test]
fn test_fn_as_method_name() {
    let source = r#"
        struct FunctionBuilder {
            fn fn(self, body) {
                return body
            }
        }
        let f = FunctionBuilder {}
        f.fn("return 42")
    "#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Should parse 'fn' as method name: {:?}", result.err());
}

#[test]
fn test_let_as_method_name() {
    let source = r#"
        struct Variable {
            fn let(self, value) {
                return value
            }
        }
        let v = Variable {}
        v.let(10)
    "#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Should parse 'let' as method name: {:?}", result.err());
}

#[test]
fn test_multiple_keyword_methods_in_chain() {
    let source = r#"
        struct FluentAPI {
            fn from(self, source) {
                return self
            }
            
            fn where(self, condition) {
                return self
            }
            
            fn select(self, fields) {
                return self
            }
            
            fn import(self, module) {
                return self
            }
        }
        
        let api = FluentAPI {}
        api.from("table").where("id > 0").select("*").import("helpers")
    "#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Should parse multiple keyword methods in chain: {:?}", result.err());
}

#[test]
fn test_keyword_stays_reserved_in_language_context() {
    // This should fail - 'from' is still reserved for imports
    let source = r#"
        from math import pi
    "#;
    
    let result = parse(source);
    // This should parse successfully as an import statement
    assert!(result.is_ok(), "Should parse 'from' as import keyword");
}

#[test]
fn test_keyword_stays_reserved_for_control_flow() {
    // This should parse correctly - keywords work in language context
    let source = r#"
        if true {
            for x in [1, 2, 3] {
                let y = x
            }
        }
    "#;
    
    let result = parse(source);
    assert!(result.is_ok(), "Keywords should still work in language context");
}
