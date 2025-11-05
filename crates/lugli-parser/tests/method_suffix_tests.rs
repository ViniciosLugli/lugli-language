use lugli_parser::Parser;

#[test]
fn test_parse_query_method_syntax() {
    let source = r#"
        struct Counter {
            value

            fn is_zero?(self) {
                return self.value == 0
            }
        }

        let c = Counter { value: 0 }
        c.is_zero?()
    "#;

    let mut parser = Parser::new(source).unwrap();
    let ast = parser.parse();
    assert!(ast.is_ok(), "Parser should handle ? suffix in methods");
}

#[test]
fn test_parse_mutating_method_syntax() {
    let source = r#"
        struct Counter {
            value

            fn increment!(self) {
                self.value = self.value + 1
            }
        }

        let c = Counter { value: 0 }
        c.increment!()
    "#;

    let mut parser = Parser::new(source).unwrap();
    let ast = parser.parse();
    assert!(ast.is_ok(), "Parser should handle ! suffix in methods");
}

#[test]
fn test_parse_both_suffix_types() {
    let source = r#"
        struct List {
            items

            fn push(self, item) {
                self.items.push(item)
            }

            fn is_empty?(self) {
                return self.items.len() == 0
            }
        }

        let list = List { items: [] }
        list.push(42)
        let empty = list.is_empty?()
    "#;

    let mut parser = Parser::new(source).unwrap();
    let ast = parser.parse();
    assert!(ast.is_ok(), "Parser should handle both ! and ? suffixes");
}

#[test]
fn test_method_call_without_suffix() {
    let source = r#"
        struct Person {
            name

            fn greet(self) {
                print(f"Hello, {self.name}")
            }
        }

        let p = Person { name: "Alice" }
        p.greet()
    "#;

    let mut parser = Parser::new(source).unwrap();
    let ast = parser.parse();
    assert!(ast.is_ok(), "Parser should handle methods without suffix");
}
