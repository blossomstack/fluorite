//! Parser for the Fluorite IDL using chumsky

// chumsky's `Rich<Token>` error is large by design and pervades every parser
// combinator here; boxing it isn't warranted for this internal parser.
#![allow(clippy::result_large_err)]

use chumsky::input::{MappedInput, Stream};
use chumsky::prelude::*;
use logos::Logos;

use crate::idl::ast::{
    AstAttribute, AstEnum, AstEnumVariant, AstField, AstFile, AstItem, AstStruct, AstType,
    AstTypeAlias, AstUnion, AstUnionVariant, AstUse, Span, Spanned,
};
use crate::idl::lexer::Token;

/// Parse error type
pub type ParseError = Rich<'static, Token, Span>;

/// The error configuration every parser in this module carries.
type Extra = extra::Err<ParseError>;

/// The parser input: the lexer's `(token, span)` pairs, presented to chumsky as
/// a stream of tokens that reports the lexer's real byte spans.
///
/// Naming the input concretely — rather than making every combinator generic
/// over `I: ValueInput` — keeps the signatures below readable. It costs nothing:
/// this parser only ever runs over one kind of input.
type TokenStream =
    MappedInput<'static, Token, Span, Stream<std::vec::IntoIter<(Token, Span)>>, SplitFn>;

/// The `(token, span)` splitter chumsky's [`Input::map`] takes. Spelled as a
/// function pointer so [`TokenStream`] stays nameable — a closure's type isn't.
type SplitFn = fn((Token, Span)) -> (Token, Span);

/// Parse a complete .fl file from source string
pub fn parse_file(source: &str) -> Result<AstFile, Vec<ParseError>> {
    let tokens = tokenize(source)?;
    let eoi = source.len()..source.len();
    file_parser().parse(token_stream(tokens, eoi)).into_result()
}

/// Wrap lexed tokens as parser input, with `eoi` as the span reported at the
/// end of input.
fn token_stream(tokens: Vec<(Token, Span)>, eoi: Span) -> TokenStream {
    Stream::from_iter(tokens).map(eoi, (|pair| pair) as SplitFn)
}

/// Tokenize source string, keeping each token's byte span.
///
/// The span matters: parsing a bare token slice makes chumsky number positions
/// by token index, so a reported span like `19..20` looks like a byte offset but
/// points somewhere unrelated in the file. Carrying the lexer's real spans
/// through means error positions map back to actual source offsets.
///
/// Input the lexer cannot recognise is an error rather than something to skip —
/// dropping it silently would let a stray character vanish and surface later as
/// a confusing structural parse failure.
fn tokenize(source: &str) -> Result<Vec<(Token, Span)>, Vec<ParseError>> {
    let mut tokens = Vec::new();
    let mut errors = Vec::new();

    for (result, span) in Token::lexer(source).spanned() {
        match result {
            Ok(token) => tokens.push((token, span)),
            Err(()) => errors.push(Rich::custom(span, "unrecognized input")),
        }
    }

    if errors.is_empty() {
        Ok(tokens)
    } else {
        Err(errors)
    }
}

/// Parser for a complete file
fn file_parser() -> impl Parser<'static, TokenStream, AstFile, Extra> {
    // Skip any leading doc comments (file-level documentation)
    doc_comment()
        .repeated()
        .ignore_then(package_stmt())
        .then(use_stmt().repeated().collect::<Vec<_>>())
        .then(item().repeated().collect::<Vec<_>>())
        .map(|((package, uses), items)| AstFile {
            package,
            uses,
            items,
        })
        .then_ignore(end())
}

/// Parser for dotted path: `foo.bar.baz`
fn dotted_path() -> impl Parser<'static, TokenStream, Vec<Spanned<String>>, Extra> {
    ident()
        .separated_by(just(Token::Dot))
        .at_least(1)
        .collect::<Vec<_>>()
}

/// Parser for package statement: `package com.example.users;`
fn package_stmt() -> impl Parser<'static, TokenStream, Vec<Spanned<String>>, Extra> {
    just(Token::Package)
        .ignore_then(dotted_path())
        .then_ignore(just(Token::Semi))
}

/// Parser for use statement: `use com.example.users.User;`
fn use_stmt() -> impl Parser<'static, TokenStream, AstUse, Extra> {
    just(Token::Use)
        .ignore_then(dotted_path())
        .then_ignore(just(Token::Semi))
        .map_with(|path, e| AstUse {
            path,
            span: e.span(),
        })
}

/// Parser for any top-level item
fn item() -> impl Parser<'static, TokenStream, AstItem, Extra> {
    choice((
        struct_def().map(AstItem::Struct),
        enum_def().map(AstItem::Enum),
        union_def().map(AstItem::Union),
        type_alias().map(AstItem::TypeAlias),
    ))
}

/// Parser for struct definition
fn struct_def() -> impl Parser<'static, TokenStream, AstStruct, Extra> {
    doc_comments()
        .then(attributes())
        .then_ignore(just(Token::Struct))
        .then(ident())
        .then(struct_body())
        .map_with(|(((doc, attrs), name), fields), e| AstStruct {
            name,
            attrs,
            fields,
            doc,
            span: e.span(),
        })
}

/// Parser for struct body: `{ fields }`
fn struct_body() -> impl Parser<'static, TokenStream, Vec<AstField>, Extra> {
    just(Token::LBrace)
        .ignore_then(
            field()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<Vec<_>>(),
        )
        .then_ignore(just(Token::RBrace))
}

/// Parser for a field
fn field() -> impl Parser<'static, TokenStream, AstField, Extra> {
    doc_comments()
        .then(attributes())
        .then(ident())
        .then_ignore(just(Token::Colon))
        .then(ty())
        .map_with(|(((doc, attrs), name), ty), e| AstField {
            name,
            ty,
            attrs,
            doc,
            span: e.span(),
        })
}

/// Parser for primitive type tokens (String, bool, i32, etc.)
fn primitive_type() -> impl Parser<'static, TokenStream, AstType, Extra> + Clone {
    let primitives = choice((
        just(Token::TyString).to("String".to_string()),
        just(Token::TyBool).to("bool".to_string()),
        just(Token::TyI32).to("i32".to_string()),
        just(Token::TyI64).to("i64".to_string()),
        just(Token::TyU32).to("u32".to_string()),
        just(Token::TyU64).to("u64".to_string()),
        just(Token::TyF32).to("f32".to_string()),
        just(Token::TyF64).to("f64".to_string()),
        just(Token::TyAny).to("Any".to_string()),
        // Extended types
        just(Token::TyUuid).to("Uuid".to_string()),
        just(Token::TyDecimal).to("Decimal".to_string()),
        just(Token::TyBytes).to("Bytes".to_string()),
        just(Token::TyUrl).to("Url".to_string()),
        just(Token::TyDateTime).to("DateTime".to_string()),
        just(Token::TyDateTimeUtc).to("DateTimeUtc".to_string()),
        just(Token::TyDateTimeTz).to("DateTimeTz".to_string()),
        just(Token::TyDate).to("Date".to_string()),
        just(Token::TyTime).to("Time".to_string()),
        just(Token::TyDuration).to("Duration".to_string()),
        just(Token::TyTimestamp).to("Timestamp".to_string()),
        just(Token::TyTimestampMillis).to("TimestampMillis".to_string()),
    ));

    primitives.map_with(|name, e| AstType::Named(Spanned::new(name, e.span())))
}

/// Parser for type expression
fn ty() -> impl Parser<'static, TokenStream, AstType, Extra> + Clone {
    recursive(|ty| {
        // Generic types must be tried first since they start with specific keywords
        let option = just(Token::TyOption)
            .ignore_then(just(Token::LAngle))
            .ignore_then(ty.clone())
            .then_ignore(just(Token::RAngle))
            .map(|inner| AstType::Option(Box::new(inner)));

        let vec = just(Token::TyVec)
            .ignore_then(just(Token::LAngle))
            .ignore_then(ty.clone())
            .then_ignore(just(Token::RAngle))
            .map(|inner| AstType::Vec(Box::new(inner)));

        let map = just(Token::TyMap)
            .ignore_then(just(Token::LAngle))
            .ignore_then(ty.clone())
            .then_ignore(just(Token::Comma))
            .then(ty.clone())
            .then_ignore(just(Token::RAngle))
            .map(|(key, value)| AstType::Map(Box::new(key), Box::new(value)));

        // Primitive types (String, bool, i32, Uuid, etc.)
        let primitive = primitive_type();

        // Custom named types (User, Order, etc.)
        let custom = ident().map(AstType::Named);

        choice((option, vec, map, primitive, custom))
    })
}

/// Parser for enum definition
fn enum_def() -> impl Parser<'static, TokenStream, AstEnum, Extra> {
    doc_comments()
        .then(attributes())
        .then_ignore(just(Token::Enum))
        .then(ident())
        .then(enum_body())
        .map_with(|(((doc, attrs), name), variants), e| AstEnum {
            name,
            attrs,
            variants,
            doc,
            span: e.span(),
        })
}

/// Parser for enum body: `{ variants }`
fn enum_body() -> impl Parser<'static, TokenStream, Vec<AstEnumVariant>, Extra> {
    just(Token::LBrace)
        .ignore_then(
            enum_variant()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<Vec<_>>(),
        )
        .then_ignore(just(Token::RBrace))
}

/// Parser for enum variant
fn enum_variant() -> impl Parser<'static, TokenStream, AstEnumVariant, Extra> {
    doc_comments()
        .then(attributes())
        .then(ident())
        .map_with(|((doc, attrs), name), e| AstEnumVariant {
            name,
            attrs,
            doc,
            span: e.span(),
        })
}

/// Parser for union definition
fn union_def() -> impl Parser<'static, TokenStream, AstUnion, Extra> {
    doc_comments()
        .then(attributes())
        .then_ignore(just(Token::Union))
        .then(ident())
        .then(union_body())
        .map_with(|(((doc, attrs), name), variants), e| AstUnion {
            name,
            attrs,
            variants,
            doc,
            span: e.span(),
        })
}

/// Parser for union body: `{ variants }`
fn union_body() -> impl Parser<'static, TokenStream, Vec<AstUnionVariant>, Extra> {
    just(Token::LBrace)
        .ignore_then(
            union_variant()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<Vec<_>>(),
        )
        .then_ignore(just(Token::RBrace))
}

/// Parser for union variant: `Variant` or `Variant(Type)`, optionally preceded
/// by a doc comment (as on enum variants and struct fields).
fn union_variant() -> impl Parser<'static, TokenStream, AstUnionVariant, Extra> {
    doc_comments()
        .then(ident())
        .then(
            just(Token::LParen)
                .ignore_then(ident())
                .then_ignore(just(Token::RParen))
                .or_not(),
        )
        .map_with(|((doc, name), inner_type), e| AstUnionVariant {
            name,
            inner_type,
            doc,
            span: e.span(),
        })
}

/// Parser for type alias: `type Name = Target;`
fn type_alias() -> impl Parser<'static, TokenStream, AstTypeAlias, Extra> {
    doc_comments()
        .then_ignore(just(Token::Type))
        .then(ident())
        .then_ignore(just(Token::Eq))
        .then(ty())
        .then_ignore(just(Token::Semi))
        .map_with(|((doc, name), target), e| AstTypeAlias {
            name,
            target,
            doc,
            span: e.span(),
        })
}

/// Parser for attributes: `#[attr]` or `#[attr = "value"]`
fn attributes() -> impl Parser<'static, TokenStream, Vec<AstAttribute>, Extra> {
    attribute().repeated().collect::<Vec<_>>()
}

fn attribute() -> impl Parser<'static, TokenStream, AstAttribute, Extra> {
    just(Token::Hash)
        .ignore_then(just(Token::LBracket))
        .ignore_then(ident())
        .then(just(Token::Eq).ignore_then(string_lit()).or_not())
        .then_ignore(just(Token::RBracket))
        .map_with(|(name, value), e| AstAttribute {
            name,
            value,
            span: e.span(),
        })
}

/// Parser for a single `///` line as a string
fn doc_comment() -> impl Parser<'static, TokenStream, String, Extra> {
    select! {
        Token::DocComment(s) => s,
    }
}

/// Parser for a run of consecutive `///` lines, joined into one doc comment.
///
/// The lexer emits one token per `///` line, so a wrapped comment arrives here
/// as several strings. They are joined with newlines and kept as written —
/// keeping only the first would cut the prose wherever the author happened to
/// wrap it.
fn doc_comments() -> impl Parser<'static, TokenStream, Option<String>, Extra> {
    doc_comment()
        .repeated()
        .collect::<Vec<_>>()
        .map(|lines: Vec<String>| {
            if lines.is_empty() {
                None
            } else {
                Some(lines.join("\n"))
            }
        })
}

/// Parser for identifier (including type keywords when used as names)
fn ident() -> impl Parser<'static, TokenStream, Spanned<String>, Extra> + Clone {
    // Accept both Ident tokens and type keywords (which can be used as field names)
    let ident_token = select! {
        Token::Ident(s) => s,
    };

    let type_as_ident = choice((
        just(Token::TyString).to("String".to_string()),
        just(Token::TyBool).to("bool".to_string()),
        just(Token::TyI32).to("i32".to_string()),
        just(Token::TyI64).to("i64".to_string()),
        just(Token::TyU32).to("u32".to_string()),
        just(Token::TyU64).to("u64".to_string()),
        just(Token::TyF32).to("f32".to_string()),
        just(Token::TyF64).to("f64".to_string()),
        just(Token::TyOption).to("Option".to_string()),
        just(Token::TyVec).to("Vec".to_string()),
        just(Token::TyMap).to("Map".to_string()),
        just(Token::TyAny).to("Any".to_string()),
        just(Token::TyUuid).to("Uuid".to_string()),
        just(Token::TyDecimal).to("Decimal".to_string()),
        just(Token::TyBytes).to("Bytes".to_string()),
        just(Token::TyUrl).to("Url".to_string()),
        just(Token::TyDateTime).to("DateTime".to_string()),
        just(Token::TyDateTimeUtc).to("DateTimeUtc".to_string()),
        just(Token::TyDateTimeTz).to("DateTimeTz".to_string()),
        just(Token::TyDate).to("Date".to_string()),
        just(Token::TyTime).to("Time".to_string()),
        just(Token::TyDuration).to("Duration".to_string()),
        just(Token::TyTimestamp).to("Timestamp".to_string()),
        just(Token::TyTimestampMillis).to("TimestampMillis".to_string()),
    ));

    ident_token
        .or(type_as_ident)
        .map_with(|name, e| Spanned::new(name, e.span()))
}

/// Parser for string literal
fn string_lit() -> impl Parser<'static, TokenStream, Spanned<String>, Extra> {
    select! {
        Token::StringLit(s) => s,
    }
    .map_with(|lit, e| Spanned::new(lit, e.span()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_package() {
        let source = "package orders;";
        let result = parse_file(source);
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert_eq!(ast.package.len(), 1);
        assert_eq!(ast.package[0].value, "orders");
    }

    #[test]
    fn test_parse_dotted_package() {
        let source = "package com.example.users;";
        let result = parse_file(source);
        assert!(result.is_ok(), "{:?}", result.err());
        let ast = result.unwrap();
        assert_eq!(ast.package.len(), 3);
        assert_eq!(ast.package[0].value, "com");
        assert_eq!(ast.package[1].value, "example");
        assert_eq!(ast.package[2].value, "users");
    }

    #[test]
    fn test_parse_deep_dotted_path() {
        let source = "package a.b.c.d.e.f;";
        let result = parse_file(source);
        assert!(result.is_ok(), "{:?}", result.err());
        let ast = result.unwrap();
        assert_eq!(ast.package.len(), 6);
        assert_eq!(ast.package[0].value, "a");
        assert_eq!(ast.package[5].value, "f");
    }

    #[test]
    fn test_parse_use() {
        let source = r#"
            package test;
            use com.example.users.User;
        "#;
        let result = parse_file(source);
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert_eq!(ast.uses.len(), 1);
        assert_eq!(ast.uses[0].path.len(), 4);
        assert_eq!(ast.uses[0].path[0].value, "com");
        assert_eq!(ast.uses[0].path[1].value, "example");
        assert_eq!(ast.uses[0].path[2].value, "users");
        assert_eq!(ast.uses[0].path[3].value, "User");
    }

    #[test]
    fn test_parse_dotted_use() {
        let source = r#"
            package test;
            use com.example.users.User;
            use com.example.orders.Order;
        "#;
        let result = parse_file(source);
        assert!(result.is_ok(), "{:?}", result.err());
        let ast = result.unwrap();
        assert_eq!(ast.uses.len(), 2);
        assert_eq!(ast.uses[0].path.len(), 4);
        assert_eq!(ast.uses[1].path.len(), 4);
    }

    #[test]
    fn test_parse_struct() {
        let source = r#"
            package test;
            struct User {
                name: String,
                age: u32,
            }
        "#;
        let result = parse_file(source);
        assert!(result.is_ok(), "{:?}", result.err());
        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 1);
        match &ast.items[0] {
            AstItem::Struct(s) => {
                assert_eq!(s.name.value, "User");
                assert_eq!(s.fields.len(), 2);
                assert_eq!(s.fields[0].name.value, "name");
                assert_eq!(s.fields[1].name.value, "age");
            }
            _ => panic!("Expected struct"),
        }
    }

    #[test]
    fn test_parse_enum() {
        let source = r#"
            package test;
            enum Status {
                Active,
                Inactive,
            }
        "#;
        let result = parse_file(source);
        assert!(result.is_ok(), "{:?}", result.err());
        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 1);
        match &ast.items[0] {
            AstItem::Enum(e) => {
                assert_eq!(e.name.value, "Status");
                assert_eq!(e.variants.len(), 2);
                assert_eq!(e.variants[0].name.value, "Active");
                assert_eq!(e.variants[1].name.value, "Inactive");
            }
            _ => panic!("Expected enum"),
        }
    }

    #[test]
    fn test_parse_union() {
        let source = r#"
            package test;
            union Event {
                UserCreated(User),
                OrderPlaced(Order),
            }
        "#;
        let result = parse_file(source);
        assert!(result.is_ok(), "{:?}", result.err());
        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 1);
        match &ast.items[0] {
            AstItem::Union(u) => {
                assert_eq!(u.name.value, "Event");
                assert_eq!(u.variants.len(), 2);
                assert_eq!(u.variants[0].name.value, "UserCreated");
                assert!(u.variants[0].inner_type.is_some());
                assert_eq!(u.variants[0].inner_type.as_ref().unwrap().value, "User");
            }
            _ => panic!("Expected union"),
        }
    }

    #[test]
    fn test_parse_type_alias() {
        let source = r#"
            package test;
            type OrderList = Vec<Order>;
        "#;
        let result = parse_file(source);
        assert!(result.is_ok(), "{:?}", result.err());
        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 1);
        match &ast.items[0] {
            AstItem::TypeAlias(t) => {
                assert_eq!(t.name.value, "OrderList");
            }
            _ => panic!("Expected type alias"),
        }
    }

    #[test]
    fn test_parse_with_doc_comment() {
        let source = r#"
            package test;
            /// A user in the system
            struct User {
                /// The user's name
                name: String,
            }
        "#;
        let result = parse_file(source);
        assert!(result.is_ok(), "{:?}", result.err());
        let ast = result.unwrap();
        match &ast.items[0] {
            AstItem::Struct(s) => {
                assert_eq!(s.doc.as_ref().unwrap(), "A user in the system");
                assert_eq!(s.fields[0].doc.as_ref().unwrap(), "The user's name");
            }
            _ => panic!("Expected struct"),
        }
    }

    #[test]
    fn test_parse_joins_consecutive_doc_comment_lines() {
        let source = r#"
            package test;
            /// A user in the system. This sentence wraps across
            /// three source lines, and every one of them
            /// belongs to the comment.
            struct User {
                /// The user's name,
                /// as they wrote it.
                name: String,
            }
        "#;
        let result = parse_file(source);
        assert!(result.is_ok(), "{:?}", result.err());
        let ast = result.unwrap();
        match &ast.items[0] {
            AstItem::Struct(s) => {
                assert_eq!(
                    s.doc.as_deref().unwrap(),
                    "A user in the system. This sentence wraps across\n\
                     three source lines, and every one of them\n\
                     belongs to the comment."
                );
                assert_eq!(
                    s.fields[0].doc.as_deref().unwrap(),
                    "The user's name,\nas they wrote it."
                );
            }
            _ => panic!("Expected struct"),
        }
    }

    #[test]
    fn test_parse_joins_doc_comment_lines_on_every_item_kind() {
        let source = r#"
            package test;
            /// An enum.
            /// Second line.
            enum Status {
                /// A variant.
                /// Second line.
                Active,
            }
            /// A union.
            /// Second line.
            union Event {
                /// A union variant.
                /// Second line.
                Created,
            }
            /// An alias.
            /// Second line.
            type Statuses = Vec<Status>;
        "#;
        let result = parse_file(source);
        assert!(result.is_ok(), "{:?}", result.err());
        let ast = result.unwrap();

        let docs: Vec<&str> = ast
            .items
            .iter()
            .map(|item| match item {
                AstItem::Struct(s) => s.doc.as_deref().unwrap(),
                AstItem::Enum(e) => e.doc.as_deref().unwrap(),
                AstItem::Union(u) => u.doc.as_deref().unwrap(),
                AstItem::TypeAlias(a) => a.doc.as_deref().unwrap(),
            })
            .collect();
        assert_eq!(
            docs,
            vec![
                "An enum.\nSecond line.",
                "A union.\nSecond line.",
                "An alias.\nSecond line.",
            ]
        );

        match &ast.items[0] {
            AstItem::Enum(e) => assert_eq!(
                e.variants[0].doc.as_deref().unwrap(),
                "A variant.\nSecond line."
            ),
            _ => panic!("Expected enum"),
        }
        match &ast.items[1] {
            AstItem::Union(u) => assert_eq!(
                u.variants[0].doc.as_deref().unwrap(),
                "A union variant.\nSecond line."
            ),
            _ => panic!("Expected union"),
        }
    }

    #[test]
    fn test_parse_with_attributes() {
        let source = r#"
            package test;
            #[rename = "user_name"]
            struct User {
                #[deprecated]
                name: String,
            }
        "#;
        let result = parse_file(source);
        assert!(result.is_ok(), "{:?}", result.err());
        let ast = result.unwrap();
        match &ast.items[0] {
            AstItem::Struct(s) => {
                assert_eq!(s.attrs.len(), 1);
                assert_eq!(s.attrs[0].name.value, "rename");
                assert_eq!(s.attrs[0].value.as_ref().unwrap().value, "user_name");
                assert_eq!(s.fields[0].attrs.len(), 1);
                assert_eq!(s.fields[0].attrs[0].name.value, "deprecated");
                assert!(s.fields[0].attrs[0].value.is_none());
            }
            _ => panic!("Expected struct"),
        }
    }

    #[test]
    fn test_parse_complex_types() {
        let source = r#"
            package test;
            struct Data {
                items: Vec<String>,
                maybe: Option<i32>,
                mapping: Map<String, User>,
            }
        "#;
        let result = parse_file(source);
        assert!(result.is_ok(), "{:?}", result.err());
    }
}
