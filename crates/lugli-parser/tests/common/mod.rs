use lugli_parser::parse;

pub fn assert_parse_success(source: &str) {
    match parse(source) {
        Ok(_) => {},
        Err(e) => panic!("Expected parse success for '{}', got: {:?}", source, e),
    }
}