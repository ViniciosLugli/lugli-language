use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeHint {
    Simple(String),
    Generic { base: String, args: Vec<TypeHint> },
    Function { params: Vec<TypeHint>, return_type: Box<TypeHint> },
    Optional(Box<TypeHint>),
    Union(Vec<TypeHint>),
}

impl TypeHint {
    pub fn simple(name: impl Into<String>) -> Self {
        TypeHint::Simple(name.into())
    }

    pub fn list(element: TypeHint) -> Self {
        TypeHint::Generic {
            base: "List".to_string(),
            args: vec![element],
        }
    }

    pub fn dict(key: TypeHint, value: TypeHint) -> Self {
        TypeHint::Generic {
            base: "Dict".to_string(),
            args: vec![key, value],
        }
    }

    pub fn optional(inner: TypeHint) -> Self {
        TypeHint::Optional(Box::new(inner))
    }
}

impl fmt::Display for TypeHint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeHint::Simple(name) => write!(f, "{}", name),
            TypeHint::Generic { base, args } => {
                write!(f, "{}<", base)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ">")
            }
            TypeHint::Function { params, return_type } => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", return_type)
            }
            TypeHint::Optional(inner) => write!(f, "{}?", inner),
            TypeHint::Union(types) => {
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{}", ty)?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_type() {
        let hint = TypeHint::simple("num");
        assert_eq!(hint.to_string(), "num");
    }

    #[test]
    fn test_generic_type() {
        let hint = TypeHint::list(TypeHint::simple("str"));
        assert_eq!(hint.to_string(), "List<str>");
    }

    #[test]
    fn test_nested_generic() {
        let hint = TypeHint::dict(TypeHint::simple("str"), TypeHint::list(TypeHint::simple("num")));
        assert_eq!(hint.to_string(), "Dict<str, List<num>>");
    }

    #[test]
    fn test_optional_type() {
        let hint = TypeHint::optional(TypeHint::simple("num"));
        assert_eq!(hint.to_string(), "num?");
    }

    #[test]
    fn test_union_type() {
        let hint = TypeHint::Union(vec![
            TypeHint::simple("num"),
            TypeHint::simple("str"),
            TypeHint::simple("null"),
        ]);
        assert_eq!(hint.to_string(), "num | str | null");
    }

    #[test]
    fn test_function_type() {
        let hint = TypeHint::Function {
            params: vec![TypeHint::simple("num"), TypeHint::simple("str")],
            return_type: Box::new(TypeHint::simple("bool")),
        };
        assert_eq!(hint.to_string(), "fn(num, str) -> bool");
    }
}
