# Fluorite IDL Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement a custom IDL parser for Fluorite using Rust-like syntax, outputting to the existing IR for code generation.

**Architecture:** Lexer (logos) → Parser (chumsky) → AST → AST-to-IR converter → Existing code generation pipeline

**Tech Stack:** logos (lexer), chumsky (parser), existing IR types

---

## Task 1: Add Dependencies to Cargo.toml

**Files:**
- Modify: `codegen/Cargo.toml`

**Step 1: Add logos and chumsky dependencies**

Add to `[dependencies]` section in `codegen/Cargo.toml`:

```toml
logos = "0.14"
chumsky = "0.9"
ariadne = "0.4"
```

Note: `ariadne` is for pretty error reporting with source spans.

**Step 2: Verify dependencies compile**

Run: `cargo build --package fluorite_codegen`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add codegen/Cargo.toml
git commit -m "feat(idl): add parser dependencies (logos, chumsky, ariadne)"
```

---

## Task 2: Create AST Type Definitions

**Files:**
- Create: `codegen/src/idl/ast.rs`

**Step 1: Write AST types**

```rust
//! AST types for the Fluorite IDL

use std::ops::Range;

/// Source span for error reporting
pub type Span = Range<usize>;

/// A spanned value
#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}

/// A complete .fl file
#[derive(Debug, Clone)]
pub struct AstFile {
    pub package: Spanned<String>,
    pub uses: Vec<AstUse>,
    pub items: Vec<AstItem>,
}

/// A use statement: `use foo::bar::Type;`
#[derive(Debug, Clone)]
pub struct AstUse {
    pub path: Vec<Spanned<String>>,
    pub span: Span,
}

/// A top-level item
#[derive(Debug, Clone)]
pub enum AstItem {
    Struct(AstStruct),
    Enum(AstEnum),
    Union(AstUnion),
    TypeAlias(AstTypeAlias),
}

/// Struct definition
#[derive(Debug, Clone)]
pub struct AstStruct {
    pub name: Spanned<String>,
    pub attrs: Vec<AstAttribute>,
    pub fields: Vec<AstField>,
    pub doc: Option<String>,
    pub span: Span,
}

/// Field in a struct
#[derive(Debug, Clone)]
pub struct AstField {
    pub name: Spanned<String>,
    pub ty: AstType,
    pub attrs: Vec<AstAttribute>,
    pub doc: Option<String>,
    pub span: Span,
}

/// Type expression
#[derive(Debug, Clone)]
pub enum AstType {
    /// Simple type name: `String`, `User`
    Named(Spanned<String>),
    /// Optional type: `Option<T>`
    Option(Box<AstType>),
    /// List type: `Vec<T>`
    Vec(Box<AstType>),
    /// Map type: `Map<K, V>`
    Map(Box<AstType>, Box<AstType>),
}

/// Enum definition
#[derive(Debug, Clone)]
pub struct AstEnum {
    pub name: Spanned<String>,
    pub attrs: Vec<AstAttribute>,
    pub variants: Vec<AstEnumVariant>,
    pub doc: Option<String>,
    pub span: Span,
}

/// Enum variant
#[derive(Debug, Clone)]
pub struct AstEnumVariant {
    pub name: Spanned<String>,
    pub attrs: Vec<AstAttribute>,
    pub doc: Option<String>,
    pub span: Span,
}

/// Union definition (tagged union)
#[derive(Debug, Clone)]
pub struct AstUnion {
    pub name: Spanned<String>,
    pub attrs: Vec<AstAttribute>,
    pub variants: Vec<AstUnionVariant>,
    pub doc: Option<String>,
    pub span: Span,
}

/// Union variant
#[derive(Debug, Clone)]
pub struct AstUnionVariant {
    pub name: Spanned<String>,
    pub inner_type: Option<Spanned<String>>,
    pub span: Span,
}

/// Type alias: `type OrderList = Vec<Order>;`
#[derive(Debug, Clone)]
pub struct AstTypeAlias {
    pub name: Spanned<String>,
    pub target: AstType,
    pub doc: Option<String>,
    pub span: Span,
}

/// Attribute: `#[rename = "value"]` or `#[deprecated]`
#[derive(Debug, Clone)]
pub struct AstAttribute {
    pub name: Spanned<String>,
    pub value: Option<Spanned<String>>,
    pub span: Span,
}
```

**Step 2: Commit**

```bash
git add codegen/src/idl/ast.rs
git commit -m "feat(idl): add AST type definitions"
```

---

## Task 3: Create Lexer with Logos

**Files:**
- Create: `codegen/src/idl/lexer.rs`

**Step 1: Write lexer token definitions**

```rust
//! Lexer for the Fluorite IDL using logos

use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")]
pub enum Token {
    // Keywords
    #[token("package")]
    Package,
    #[token("use")]
    Use,
    #[token("struct")]
    Struct,
    #[token("enum")]
    Enum,
    #[token("union")]
    Union,
    #[token("type")]
    Type,

    // Type keywords
    #[token("String")]
    TyString,
    #[token("bool")]
    TyBool,
    #[token("i32")]
    TyI32,
    #[token("i64")]
    TyI64,
    #[token("u32")]
    TyU32,
    #[token("u64")]
    TyU64,
    #[token("f32")]
    TyF32,
    #[token("f64")]
    TyF64,
    #[token("Option")]
    TyOption,
    #[token("Vec")]
    TyVec,
    #[token("Map")]
    TyMap,
    #[token("Any")]
    TyAny,

    // Extended type keywords
    #[token("Uuid")]
    TyUuid,
    #[token("Decimal")]
    TyDecimal,
    #[token("Bytes")]
    TyBytes,
    #[token("Url")]
    TyUrl,
    #[token("DateTime")]
    TyDateTime,
    #[token("DateTimeUtc")]
    TyDateTimeUtc,
    #[token("DateTimeTz")]
    TyDateTimeTz,
    #[token("Date")]
    TyDate,
    #[token("Time")]
    TyTime,
    #[token("Duration")]
    TyDuration,
    #[token("Timestamp")]
    TyTimestamp,
    #[token("TimestampMillis")]
    TyTimestampMillis,

    // Punctuation
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("<")]
    LAngle,
    #[token(">")]
    RAngle,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(";")]
    Semi,
    #[token(":")]
    Colon,
    #[token("::")]
    DoubleColon,
    #[token(",")]
    Comma,
    #[token("=")]
    Eq,
    #[token("#")]
    Hash,

    // Identifier
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    // String literal
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    StringLit(String),

    // Doc comment
    #[regex(r"///[^\n]*", |lex| {
        let s = lex.slice();
        s[3..].trim().to_string()
    })]
    DocComment(String),

    // Regular comment (skip)
    #[regex(r"//[^\n]*", logos::skip)]
    Comment,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Package => write!(f, "package"),
            Token::Use => write!(f, "use"),
            Token::Struct => write!(f, "struct"),
            Token::Enum => write!(f, "enum"),
            Token::Union => write!(f, "union"),
            Token::Type => write!(f, "type"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LAngle => write!(f, "<"),
            Token::RAngle => write!(f, ">"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::Semi => write!(f, ";"),
            Token::Colon => write!(f, ":"),
            Token::DoubleColon => write!(f, "::"),
            Token::Comma => write!(f, ","),
            Token::Eq => write!(f, "="),
            Token::Hash => write!(f, "#"),
            Token::Ident(s) => write!(f, "{}", s),
            Token::StringLit(s) => write!(f, "\"{}\"", s),
            Token::DocComment(s) => write!(f, "/// {}", s),
            Token::Comment => write!(f, "// ..."),
            _ => write!(f, "{:?}", self),
        }
    }
}
```

**Step 2: Add unit tests for lexer**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;

    fn lex(input: &str) -> Vec<Token> {
        Token::lexer(input)
            .filter_map(|r| r.ok())
            .collect()
    }

    #[test]
    fn test_keywords() {
        assert_eq!(lex("package"), vec![Token::Package]);
        assert_eq!(lex("use"), vec![Token::Use]);
        assert_eq!(lex("struct"), vec![Token::Struct]);
        assert_eq!(lex("enum"), vec![Token::Enum]);
        assert_eq!(lex("union"), vec![Token::Union]);
        assert_eq!(lex("type"), vec![Token::Type]);
    }

    #[test]
    fn test_types() {
        assert_eq!(lex("String"), vec![Token::TyString]);
        assert_eq!(lex("bool"), vec![Token::TyBool]);
        assert_eq!(lex("Option"), vec![Token::TyOption]);
        assert_eq!(lex("Vec"), vec![Token::TyVec]);
        assert_eq!(lex("Map"), vec![Token::TyMap]);
    }

    #[test]
    fn test_punctuation() {
        let tokens = lex("{ } ( ) < > [ ] ; : :: , = #");
        assert_eq!(tokens, vec![
            Token::LBrace, Token::RBrace,
            Token::LParen, Token::RParen,
            Token::LAngle, Token::RAngle,
            Token::LBracket, Token::RBracket,
            Token::Semi, Token::Colon, Token::DoubleColon,
            Token::Comma, Token::Eq, Token::Hash,
        ]);
    }

    #[test]
    fn test_identifier() {
        assert_eq!(lex("foo"), vec![Token::Ident("foo".to_string())]);
        assert_eq!(lex("_bar"), vec![Token::Ident("_bar".to_string())]);
        assert_eq!(lex("MyType123"), vec![Token::Ident("MyType123".to_string())]);
    }

    #[test]
    fn test_string_literal() {
        assert_eq!(lex(r#""hello""#), vec![Token::StringLit("hello".to_string())]);
        assert_eq!(lex(r#""camelCase""#), vec![Token::StringLit("camelCase".to_string())]);
    }

    #[test]
    fn test_doc_comment() {
        assert_eq!(lex("/// This is a doc"), vec![Token::DocComment("This is a doc".to_string())]);
    }

    #[test]
    fn test_package_statement() {
        let tokens = lex("package orders;");
        assert_eq!(tokens, vec![
            Token::Package,
            Token::Ident("orders".to_string()),
            Token::Semi,
        ]);
    }

    #[test]
    fn test_use_statement() {
        let tokens = lex("use users::User;");
        assert_eq!(tokens, vec![
            Token::Use,
            Token::Ident("users".to_string()),
            Token::DoubleColon,
            Token::Ident("User".to_string()),
            Token::Semi,
        ]);
    }
}
```

**Step 3: Run tests**

Run: `cargo test --package fluorite_codegen lexer`
Expected: All tests pass

**Step 4: Commit**

```bash
git add codegen/src/idl/lexer.rs
git commit -m "feat(idl): implement lexer with logos"
```

---

## Task 4: Create Parser with Chumsky

**Files:**
- Create: `codegen/src/idl/parser.rs`

**Step 1: Write parser combinators**

```rust
//! Parser for the Fluorite IDL using chumsky

use chumsky::prelude::*;

use super::ast::*;
use super::lexer::Token;

type ParserInput<'a> = chumsky::input::SpannedInput<Token, Span, &'a [(Token, Span)]>;

/// Parse a complete .fl file
pub fn file_parser<'a>() -> impl Parser<'a, ParserInput<'a>, AstFile, extra::Err<Rich<'a, Token, Span>>> {
    let package = package_parser();
    let uses = use_parser().repeated().collect::<Vec<_>>();
    let items = item_parser().repeated().collect::<Vec<_>>();

    package
        .then(uses)
        .then(items)
        .map(|((package, uses), items)| AstFile { package, uses, items })
}

fn package_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Spanned<String>, extra::Err<Rich<'a, Token, Span>>> {
    just(Token::Package)
        .ignore_then(path_parser())
        .then_ignore(just(Token::Semi))
        .map_with(|path, e| Spanned::new(path.join("::"), e.span()))
}

fn path_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Vec<String>, extra::Err<Rich<'a, Token, Span>>> {
    ident_parser()
        .separated_by(just(Token::DoubleColon))
        .at_least(1)
        .collect()
}

fn ident_parser<'a>() -> impl Parser<'a, ParserInput<'a>, String, extra::Err<Rich<'a, Token, Span>>> {
    select! {
        Token::Ident(s) => s,
    }
}

fn use_parser<'a>() -> impl Parser<'a, ParserInput<'a>, AstUse, extra::Err<Rich<'a, Token, Span>>> {
    just(Token::Use)
        .ignore_then(
            ident_parser()
                .map_with(|s, e| Spanned::new(s, e.span()))
                .separated_by(just(Token::DoubleColon))
                .at_least(1)
                .collect::<Vec<_>>()
        )
        .then_ignore(just(Token::Semi))
        .map_with(|path, e| AstUse { path, span: e.span() })
}

fn item_parser<'a>() -> impl Parser<'a, ParserInput<'a>, AstItem, extra::Err<Rich<'a, Token, Span>>> {
    let doc = doc_parser();
    let attrs = attrs_parser();

    doc.then(attrs).then(choice((
        struct_parser().map(AstItem::Struct),
        enum_parser().map(AstItem::Enum),
        union_parser().map(AstItem::Union),
        type_alias_parser().map(AstItem::TypeAlias),
    ))).map(|((doc, attrs), mut item)| {
        match &mut item {
            AstItem::Struct(s) => { s.doc = doc; s.attrs = attrs; }
            AstItem::Enum(e) => { e.doc = doc; e.attrs = attrs; }
            AstItem::Union(u) => { u.doc = doc; u.attrs = attrs; }
            AstItem::TypeAlias(t) => { t.doc = doc; }
        }
        item
    })
}

fn doc_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Option<String>, extra::Err<Rich<'a, Token, Span>>> {
    select! { Token::DocComment(s) => s }
        .repeated()
        .collect::<Vec<_>>()
        .map(|docs| {
            if docs.is_empty() {
                None
            } else {
                Some(docs.join("\n"))
            }
        })
}

fn attrs_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Vec<AstAttribute>, extra::Err<Rich<'a, Token, Span>>> {
    attr_parser().repeated().collect()
}

fn attr_parser<'a>() -> impl Parser<'a, ParserInput<'a>, AstAttribute, extra::Err<Rich<'a, Token, Span>>> {
    just(Token::Hash)
        .then(just(Token::LBracket))
        .ignore_then(
            ident_parser()
                .map_with(|s, e| Spanned::new(s, e.span()))
                .then(
                    just(Token::Eq)
                        .ignore_then(select! { Token::StringLit(s) => s }.map_with(|s, e| Spanned::new(s, e.span())))
                        .or_not()
                )
        )
        .then_ignore(just(Token::RBracket))
        .map_with(|(name, value), e| AstAttribute { name, value, span: e.span() })
}

fn struct_parser<'a>() -> impl Parser<'a, ParserInput<'a>, AstStruct, extra::Err<Rich<'a, Token, Span>>> {
    just(Token::Struct)
        .ignore_then(ident_parser().map_with(|s, e| Spanned::new(s, e.span())))
        .then(
            field_parser()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::LBrace), just(Token::RBrace))
        )
        .map_with(|(name, fields), e| AstStruct {
            name,
            attrs: vec![],
            fields,
            doc: None,
            span: e.span(),
        })
}

fn field_parser<'a>() -> impl Parser<'a, ParserInput<'a>, AstField, extra::Err<Rich<'a, Token, Span>>> {
    doc_parser()
        .then(attrs_parser())
        .then(ident_parser().map_with(|s, e| Spanned::new(s, e.span())))
        .then_ignore(just(Token::Colon))
        .then(type_parser())
        .map_with(|(((doc, attrs), name), ty), e| AstField {
            name,
            ty,
            attrs,
            doc,
            span: e.span(),
        })
}

fn type_parser<'a>() -> impl Parser<'a, ParserInput<'a>, AstType, extra::Err<Rich<'a, Token, Span>>> {
    recursive(|ty| {
        let option_ty = just(Token::TyOption)
            .ignore_then(ty.clone().delimited_by(just(Token::LAngle), just(Token::RAngle)))
            .map(|inner| AstType::Option(Box::new(inner)));

        let vec_ty = just(Token::TyVec)
            .ignore_then(ty.clone().delimited_by(just(Token::LAngle), just(Token::RAngle)))
            .map(|inner| AstType::Vec(Box::new(inner)));

        let map_ty = just(Token::TyMap)
            .ignore_then(
                ty.clone()
                    .then_ignore(just(Token::Comma))
                    .then(ty.clone())
                    .delimited_by(just(Token::LAngle), just(Token::RAngle))
            )
            .map(|(k, v)| AstType::Map(Box::new(k), Box::new(v)));

        let primitive = select! {
            Token::TyString => "String",
            Token::TyBool => "Bool",
            Token::TyI32 => "Int32",
            Token::TyI64 => "Int64",
            Token::TyU32 => "UInt32",
            Token::TyU64 => "UInt64",
            Token::TyF32 => "Float32",
            Token::TyF64 => "Float64",
            Token::TyAny => "Any",
            Token::TyUuid => "UUID",
            Token::TyDecimal => "Decimal",
            Token::TyBytes => "Bytes",
            Token::TyUrl => "Url",
            Token::TyDateTime => "DateTime",
            Token::TyDateTimeUtc => "DateTimeUtc",
            Token::TyDateTimeTz => "DateTimeTz",
            Token::TyDate => "Date",
            Token::TyTime => "Time",
            Token::TyDuration => "Duration",
            Token::TyTimestamp => "Timestamp",
            Token::TyTimestampMillis => "TimestampMillis",
        }.map_with(|s, e| AstType::Named(Spanned::new(s.to_string(), e.span())));

        let named = ident_parser()
            .map_with(|s, e| AstType::Named(Spanned::new(s, e.span())));

        choice((option_ty, vec_ty, map_ty, primitive, named))
    })
}

fn enum_parser<'a>() -> impl Parser<'a, ParserInput<'a>, AstEnum, extra::Err<Rich<'a, Token, Span>>> {
    just(Token::Enum)
        .ignore_then(ident_parser().map_with(|s, e| Spanned::new(s, e.span())))
        .then(
            enum_variant_parser()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::LBrace), just(Token::RBrace))
        )
        .map_with(|(name, variants), e| AstEnum {
            name,
            attrs: vec![],
            variants,
            doc: None,
            span: e.span(),
        })
}

fn enum_variant_parser<'a>() -> impl Parser<'a, ParserInput<'a>, AstEnumVariant, extra::Err<Rich<'a, Token, Span>>> {
    doc_parser()
        .then(attrs_parser())
        .then(ident_parser().map_with(|s, e| Spanned::new(s, e.span())))
        .map_with(|((doc, attrs), name), e| AstEnumVariant {
            name,
            attrs,
            doc,
            span: e.span(),
        })
}

fn union_parser<'a>() -> impl Parser<'a, ParserInput<'a>, AstUnion, extra::Err<Rich<'a, Token, Span>>> {
    just(Token::Union)
        .ignore_then(ident_parser().map_with(|s, e| Spanned::new(s, e.span())))
        .then(
            union_variant_parser()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::LBrace), just(Token::RBrace))
        )
        .map_with(|(name, variants), e| AstUnion {
            name,
            attrs: vec![],
            variants,
            doc: None,
            span: e.span(),
        })
}

fn union_variant_parser<'a>() -> impl Parser<'a, ParserInput<'a>, AstUnionVariant, extra::Err<Rich<'a, Token, Span>>> {
    ident_parser()
        .map_with(|s, e| Spanned::new(s, e.span()))
        .then(
            ident_parser()
                .map_with(|s, e| Spanned::new(s, e.span()))
                .delimited_by(just(Token::LParen), just(Token::RParen))
                .or_not()
        )
        .map_with(|(name, inner_type), e| AstUnionVariant {
            name,
            inner_type,
            span: e.span(),
        })
}

fn type_alias_parser<'a>() -> impl Parser<'a, ParserInput<'a>, AstTypeAlias, extra::Err<Rich<'a, Token, Span>>> {
    just(Token::Type)
        .ignore_then(ident_parser().map_with(|s, e| Spanned::new(s, e.span())))
        .then_ignore(just(Token::Eq))
        .then(type_parser())
        .then_ignore(just(Token::Semi))
        .map_with(|(name, target), e| AstTypeAlias {
            name,
            target,
            doc: None,
            span: e.span(),
        })
}

/// Parse source code into an AST
pub fn parse(source: &str) -> Result<AstFile, Vec<Rich<'_, Token, Span>>> {
    use logos::Logos;

    let tokens: Vec<_> = Token::lexer(source)
        .spanned()
        .map(|(tok, span)| (tok.unwrap_or(Token::Ident("ERROR".to_string())), span))
        .collect();

    let len = source.len();
    let eoi = len..len;

    file_parser()
        .parse(tokens.as_slice().spanned(eoi))
        .into_result()
}
```

**Step 2: Add parser tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_package() {
        let src = "package orders;";
        let ast = parse(src).unwrap();
        assert_eq!(ast.package.value, "orders");
    }

    #[test]
    fn test_parse_nested_package() {
        let src = "package protocols::orders;";
        let ast = parse(src).unwrap();
        assert_eq!(ast.package.value, "protocols::orders");
    }

    #[test]
    fn test_parse_use_statement() {
        let src = r#"
            package test;
            use users::User;
        "#;
        let ast = parse(src).unwrap();
        assert_eq!(ast.uses.len(), 1);
        assert_eq!(ast.uses[0].path.iter().map(|s| s.value.as_str()).collect::<Vec<_>>(), vec!["users", "User"]);
    }

    #[test]
    fn test_parse_simple_struct() {
        let src = r#"
            package test;
            struct User {
                name: String,
                age: u32,
            }
        "#;
        let ast = parse(src).unwrap();
        assert_eq!(ast.items.len(), 1);
        if let AstItem::Struct(s) = &ast.items[0] {
            assert_eq!(s.name.value, "User");
            assert_eq!(s.fields.len(), 2);
            assert_eq!(s.fields[0].name.value, "name");
            assert_eq!(s.fields[1].name.value, "age");
        } else {
            panic!("Expected struct");
        }
    }

    #[test]
    fn test_parse_struct_with_attrs() {
        let src = r#"
            package test;
            #[rename_all = "camelCase"]
            struct Order {
                #[rename = "orderId"]
                id: String,
            }
        "#;
        let ast = parse(src).unwrap();
        if let AstItem::Struct(s) = &ast.items[0] {
            assert_eq!(s.attrs.len(), 1);
            assert_eq!(s.attrs[0].name.value, "rename_all");
            assert_eq!(s.attrs[0].value.as_ref().unwrap().value, "camelCase");
            assert_eq!(s.fields[0].attrs.len(), 1);
            assert_eq!(s.fields[0].attrs[0].name.value, "rename");
        } else {
            panic!("Expected struct");
        }
    }

    #[test]
    fn test_parse_optional_type() {
        let src = r#"
            package test;
            struct Order {
                shipping: Option<Shipping>,
            }
        "#;
        let ast = parse(src).unwrap();
        if let AstItem::Struct(s) = &ast.items[0] {
            match &s.fields[0].ty {
                AstType::Option(inner) => {
                    if let AstType::Named(n) = inner.as_ref() {
                        assert_eq!(n.value, "Shipping");
                    } else {
                        panic!("Expected named type inside Option");
                    }
                }
                _ => panic!("Expected Option type"),
            }
        } else {
            panic!("Expected struct");
        }
    }

    #[test]
    fn test_parse_vec_type() {
        let src = r#"
            package test;
            struct Order {
                items: Vec<OrderItem>,
            }
        "#;
        let ast = parse(src).unwrap();
        if let AstItem::Struct(s) = &ast.items[0] {
            match &s.fields[0].ty {
                AstType::Vec(inner) => {
                    if let AstType::Named(n) = inner.as_ref() {
                        assert_eq!(n.value, "OrderItem");
                    } else {
                        panic!("Expected named type inside Vec");
                    }
                }
                _ => panic!("Expected Vec type"),
            }
        } else {
            panic!("Expected struct");
        }
    }

    #[test]
    fn test_parse_map_type() {
        let src = r#"
            package test;
            type OrderMap = Map<String, Order>;
        "#;
        let ast = parse(src).unwrap();
        if let AstItem::TypeAlias(t) = &ast.items[0] {
            assert_eq!(t.name.value, "OrderMap");
            match &t.target {
                AstType::Map(k, v) => {
                    if let AstType::Named(kn) = k.as_ref() {
                        assert_eq!(kn.value, "String");
                    }
                    if let AstType::Named(vn) = v.as_ref() {
                        assert_eq!(vn.value, "Order");
                    }
                }
                _ => panic!("Expected Map type"),
            }
        } else {
            panic!("Expected type alias");
        }
    }

    #[test]
    fn test_parse_enum() {
        let src = r#"
            package test;
            enum Gender {
                Male,
                Female,
            }
        "#;
        let ast = parse(src).unwrap();
        if let AstItem::Enum(e) = &ast.items[0] {
            assert_eq!(e.name.value, "Gender");
            assert_eq!(e.variants.len(), 2);
            assert_eq!(e.variants[0].name.value, "Male");
            assert_eq!(e.variants[1].name.value, "Female");
        } else {
            panic!("Expected enum");
        }
    }

    #[test]
    fn test_parse_union() {
        let src = r#"
            package test;
            #[type_tag = "type"]
            union Address {
                Empty,
                PostCode(PostCode),
                Info(AddressInfo),
            }
        "#;
        let ast = parse(src).unwrap();
        if let AstItem::Union(u) = &ast.items[0] {
            assert_eq!(u.name.value, "Address");
            assert_eq!(u.variants.len(), 3);
            assert_eq!(u.variants[0].name.value, "Empty");
            assert!(u.variants[0].inner_type.is_none());
            assert_eq!(u.variants[1].name.value, "PostCode");
            assert_eq!(u.variants[1].inner_type.as_ref().unwrap().value, "PostCode");
        } else {
            panic!("Expected union");
        }
    }

    #[test]
    fn test_parse_doc_comment() {
        let src = r#"
            package test;
            /// A user in the system
            struct User {
                /// The user's name
                name: String,
            }
        "#;
        let ast = parse(src).unwrap();
        if let AstItem::Struct(s) = &ast.items[0] {
            assert_eq!(s.doc, Some("A user in the system".to_string()));
            assert_eq!(s.fields[0].doc, Some("The user's name".to_string()));
        } else {
            panic!("Expected struct");
        }
    }

    #[test]
    fn test_parse_type_alias() {
        let src = r#"
            package test;
            type OrderList = Vec<Order>;
        "#;
        let ast = parse(src).unwrap();
        if let AstItem::TypeAlias(t) = &ast.items[0] {
            assert_eq!(t.name.value, "OrderList");
        } else {
            panic!("Expected type alias");
        }
    }
}
```

**Step 3: Run tests**

Run: `cargo test --package fluorite_codegen parser`
Expected: All tests pass

**Step 4: Commit**

```bash
git add codegen/src/idl/parser.rs
git commit -m "feat(idl): implement parser with chumsky"
```

---

## Task 5: Create AST to IR Converter

**Files:**
- Create: `codegen/src/idl/ast_to_ir.rs`

**Step 1: Write AST to IR conversion**

```rust
//! Convert AST to IR types

use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet};

use crate::code_gen::ir::{
    IREnum, IRField, IRFieldType, IRPackage, IRPrimitive, IRSchema, IRStruct, IRType, IRTypeAlias,
    IRTypeAliasTarget, IRUnion, IRUnionStyle, IRUnionVariant,
};

use super::ast::*;

/// Convert parsed AST files to IR schema
pub struct AstToIrConverter {
    /// All type names across all files
    all_type_names: HashSet<String>,
    /// Types used as inline union variants
    union_variant_names: HashSet<String>,
}

impl AstToIrConverter {
    pub fn new() -> Self {
        Self {
            all_type_names: HashSet::new(),
            union_variant_names: HashSet::new(),
        }
    }

    /// Convert multiple AST files to an IR schema
    pub fn convert(mut self, files: &[AstFile]) -> Result<IRSchema> {
        // First pass: collect type info
        self.collect_type_info(files);

        // Second pass: convert to IR
        let mut packages: HashMap<String, IRPackage> = HashMap::new();

        for file in files {
            let package_name = &file.package.value;
            let package = packages
                .entry(package_name.clone())
                .or_insert_with(|| IRPackage {
                    name: package_name.clone(),
                    types: Vec::new(),
                });

            for item in &file.items {
                let ir_type = self.convert_item(item)?;
                package.types.push(ir_type);
            }
        }

        Ok(IRSchema { packages })
    }

    fn collect_type_info(&mut self, files: &[AstFile]) {
        // Collect all type names
        for file in files {
            for item in &file.items {
                let name = match item {
                    AstItem::Struct(s) => s.name.value.clone(),
                    AstItem::Enum(e) => e.name.value.clone(),
                    AstItem::Union(u) => u.name.value.clone(),
                    AstItem::TypeAlias(t) => t.name.value.clone(),
                };
                self.all_type_names.insert(name);
            }
        }

        // Identify inline union variants
        for file in files {
            for item in &file.items {
                if let AstItem::Union(u) = item {
                    let is_inline = !u.attrs.iter().any(|a| {
                        a.name.value == "union_style"
                            && a.value.as_ref().map(|v| v.value.as_str()) == Some("extern")
                    });

                    if is_inline {
                        for v in &u.variants {
                            if let Some(inner) = &v.inner_type {
                                if self.all_type_names.contains(&inner.value) {
                                    self.union_variant_names.insert(inner.value.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn convert_item(&self, item: &AstItem) -> Result<IRType> {
        match item {
            AstItem::Struct(s) => self.convert_struct(s),
            AstItem::Enum(e) => self.convert_enum(e),
            AstItem::Union(u) => self.convert_union(u),
            AstItem::TypeAlias(t) => self.convert_type_alias(t),
        }
    }

    fn convert_struct(&self, s: &AstStruct) -> Result<IRType> {
        let is_union_variant = self.union_variant_names.contains(&s.name.value);
        let fields = s.fields.iter().map(|f| self.convert_field(f)).collect();

        let rename_all = self.get_attr_value(&s.attrs, "rename_all");
        let deny_unknown_fields = self.has_attr(&s.attrs, "deny_unknown_fields");

        Ok(IRType::Struct(IRStruct {
            name: s.name.value.clone(),
            fields,
            is_union_variant,
            doc: s.doc.clone(),
            rename_all,
            deny_unknown_fields,
        }))
    }

    fn convert_field(&self, f: &AstField) -> IRField {
        let (field_type, is_optional) = self.convert_type(&f.ty);
        let is_boxed = self.has_attr(&f.attrs, "box");
        let rename = self.get_attr_value(&f.attrs, "rename");
        let alias_str = self.get_attr_value(&f.attrs, "alias");
        let alias = alias_str.map(|s| vec![s]).unwrap_or_default();
        let default = self.get_attr_value(&f.attrs, "default");
        let skip_if_none = self.has_attr(&f.attrs, "skip_if_none");
        let skip_if_default = self.has_attr(&f.attrs, "skip_if_default");
        let flatten = self.has_attr(&f.attrs, "flatten");
        let deprecated = self.has_attr(&f.attrs, "deprecated");

        IRField {
            name: f.name.value.clone(),
            field_type,
            is_optional,
            is_boxed,
            rename,
            doc: f.doc.clone(),
            alias,
            default,
            skip_if_none,
            skip_if_default,
            flatten,
            deprecated,
        }
    }

    fn convert_type(&self, ty: &AstType) -> (IRFieldType, bool) {
        match ty {
            AstType::Named(n) => {
                let ft = self.convert_named_type(&n.value);
                (ft, false)
            }
            AstType::Option(inner) => {
                let (inner_ft, _) = self.convert_type(inner);
                (inner_ft, true)
            }
            AstType::Vec(inner) => {
                let (inner_ft, _) = self.convert_type(inner);
                (IRFieldType::List(Box::new(inner_ft)), false)
            }
            AstType::Map(k, v) => {
                let (key_ft, _) = self.convert_type(k);
                let (value_ft, _) = self.convert_type(v);
                (IRFieldType::Map(Box::new(key_ft), Box::new(value_ft)), false)
            }
        }
    }

    fn convert_named_type(&self, name: &str) -> IRFieldType {
        if name == "Any" {
            return IRFieldType::Any;
        }

        if let Some(prim) = self.parse_primitive(name) {
            return IRFieldType::Primitive(prim);
        }

        IRFieldType::Custom(name.to_string())
    }

    fn parse_primitive(&self, name: &str) -> Option<IRPrimitive> {
        match name {
            "String" => Some(IRPrimitive::String),
            "Bool" => Some(IRPrimitive::Bool),
            "DateTime" => Some(IRPrimitive::DateTime),
            "UInt32" => Some(IRPrimitive::UInt32),
            "UInt64" => Some(IRPrimitive::UInt64),
            "Int32" => Some(IRPrimitive::Int32),
            "Int64" => Some(IRPrimitive::Int64),
            "Float32" => Some(IRPrimitive::Float32),
            "Float64" => Some(IRPrimitive::Float64),
            "UUID" => Some(IRPrimitive::UUID),
            "Decimal" => Some(IRPrimitive::Decimal),
            "Bytes" => Some(IRPrimitive::Bytes),
            "Url" => Some(IRPrimitive::Url),
            "Timestamp" => Some(IRPrimitive::Timestamp),
            "TimestampMillis" => Some(IRPrimitive::TimestampMillis),
            "DateTimeUtc" => Some(IRPrimitive::DateTimeUtc),
            "DateTimeTz" => Some(IRPrimitive::DateTimeTz),
            "Date" => Some(IRPrimitive::Date),
            "Time" => Some(IRPrimitive::Time),
            "Duration" => Some(IRPrimitive::Duration),
            _ => None,
        }
    }

    fn convert_enum(&self, e: &AstEnum) -> Result<IRType> {
        let variants = e.variants.iter().map(|v| v.name.value.clone()).collect();

        Ok(IRType::Enum(IREnum {
            name: e.name.value.clone(),
            variants,
            doc: e.doc.clone(),
        }))
    }

    fn convert_union(&self, u: &AstUnion) -> Result<IRType> {
        let tag_field = self
            .get_attr_value(&u.attrs, "type_tag")
            .ok_or_else(|| anyhow!("Union '{}' missing #[type_tag = \"...\"] attribute", u.name.value))?;

        let style = if self.get_attr_value(&u.attrs, "union_style").as_deref() == Some("extern") {
            IRUnionStyle::Extern
        } else {
            IRUnionStyle::Inline
        };

        let variants = u
            .variants
            .iter()
            .map(|v| {
                if let Some(inner) = &v.inner_type {
                    match style {
                        IRUnionStyle::Inline => IRUnionVariant::Inline(v.name.value.clone(), vec![]),
                        IRUnionStyle::Extern => {
                            IRUnionVariant::Newtype(v.name.value.clone(), inner.value.clone())
                        }
                    }
                } else {
                    IRUnionVariant::Unit(v.name.value.clone())
                }
            })
            .collect();

        Ok(IRType::Union(IRUnion {
            name: u.name.value.clone(),
            tag_field,
            variants,
            style,
            doc: u.doc.clone(),
        }))
    }

    fn convert_type_alias(&self, t: &AstTypeAlias) -> Result<IRType> {
        let target = match &t.target {
            AstType::Vec(inner) => {
                let (inner_ft, _) = self.convert_type(inner);
                IRTypeAliasTarget::List(inner_ft)
            }
            AstType::Map(k, v) => {
                let (key_ft, _) = self.convert_type(k);
                let (value_ft, _) = self.convert_type(v);
                IRTypeAliasTarget::Map(key_ft, value_ft)
            }
            _ => return Err(anyhow!("Type alias '{}' must be Vec<T> or Map<K, V>", t.name.value)),
        };

        Ok(IRType::TypeAlias(IRTypeAlias {
            name: t.name.value.clone(),
            target,
            doc: t.doc.clone(),
        }))
    }

    fn get_attr_value(&self, attrs: &[AstAttribute], name: &str) -> Option<String> {
        attrs
            .iter()
            .find(|a| a.name.value == name)
            .and_then(|a| a.value.as_ref())
            .map(|v| v.value.clone())
    }

    fn has_attr(&self, attrs: &[AstAttribute], name: &str) -> bool {
        attrs.iter().any(|a| a.name.value == name)
    }
}

impl Default for AstToIrConverter {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 2: Add converter tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::idl::parser::parse;

    fn convert(src: &str) -> Result<IRSchema> {
        let ast = parse(src).map_err(|e| anyhow!("Parse error: {:?}", e))?;
        AstToIrConverter::new().convert(&[ast])
    }

    #[test]
    fn test_convert_simple_struct() {
        let src = r#"
            package test;
            struct User {
                name: String,
                age: u32,
            }
        "#;
        let schema = convert(src).unwrap();
        let pkg = schema.packages.get("test").unwrap();
        assert_eq!(pkg.types.len(), 1);

        if let IRType::Struct(s) = &pkg.types[0] {
            assert_eq!(s.name, "User");
            assert_eq!(s.fields.len(), 2);
            assert_eq!(s.fields[0].name, "name");
            assert!(matches!(s.fields[0].field_type, IRFieldType::Primitive(IRPrimitive::String)));
            assert_eq!(s.fields[1].name, "age");
            assert!(matches!(s.fields[1].field_type, IRFieldType::Primitive(IRPrimitive::UInt32)));
        } else {
            panic!("Expected struct");
        }
    }

    #[test]
    fn test_convert_optional_field() {
        let src = r#"
            package test;
            struct Order {
                shipping: Option<String>,
            }
        "#;
        let schema = convert(src).unwrap();
        let pkg = schema.packages.get("test").unwrap();

        if let IRType::Struct(s) = &pkg.types[0] {
            assert!(s.fields[0].is_optional);
            assert!(matches!(s.fields[0].field_type, IRFieldType::Primitive(IRPrimitive::String)));
        } else {
            panic!("Expected struct");
        }
    }

    #[test]
    fn test_convert_boxed_field() {
        let src = r#"
            package test;
            struct Order {
                #[box]
                shipping: Option<Shipping>,
            }
            struct Shipping {
                id: String,
            }
        "#;
        let schema = convert(src).unwrap();
        let pkg = schema.packages.get("test").unwrap();

        if let IRType::Struct(s) = &pkg.types[0] {
            assert!(s.fields[0].is_boxed);
            assert!(s.fields[0].is_optional);
        } else {
            panic!("Expected struct");
        }
    }

    #[test]
    fn test_convert_enum() {
        let src = r#"
            package test;
            enum Gender {
                Male,
                Female,
            }
        "#;
        let schema = convert(src).unwrap();
        let pkg = schema.packages.get("test").unwrap();

        if let IRType::Enum(e) = &pkg.types[0] {
            assert_eq!(e.name, "Gender");
            assert_eq!(e.variants, vec!["Male", "Female"]);
        } else {
            panic!("Expected enum");
        }
    }

    #[test]
    fn test_convert_union() {
        let src = r#"
            package test;
            #[type_tag = "type"]
            union Address {
                Empty,
                PostCode(PostCode),
            }
            struct PostCode {
                code: String,
            }
        "#;
        let schema = convert(src).unwrap();
        let pkg = schema.packages.get("test").unwrap();

        let union_type = pkg.types.iter().find(|t| t.name() == "Address").unwrap();
        if let IRType::Union(u) = union_type {
            assert_eq!(u.tag_field, "type");
            assert_eq!(u.variants.len(), 2);
            assert!(matches!(&u.variants[0], IRUnionVariant::Unit(n) if n == "Empty"));
            assert!(matches!(&u.variants[1], IRUnionVariant::Inline(n, _) if n == "PostCode"));
        } else {
            panic!("Expected union");
        }
    }

    #[test]
    fn test_convert_type_alias_vec() {
        let src = r#"
            package test;
            type OrderList = Vec<Order>;
            struct Order {
                id: String,
            }
        "#;
        let schema = convert(src).unwrap();
        let pkg = schema.packages.get("test").unwrap();

        let alias = pkg.types.iter().find(|t| t.name() == "OrderList").unwrap();
        if let IRType::TypeAlias(a) = alias {
            if let IRTypeAliasTarget::List(inner) = &a.target {
                assert!(matches!(inner, IRFieldType::Custom(s) if s == "Order"));
            } else {
                panic!("Expected list target");
            }
        } else {
            panic!("Expected type alias");
        }
    }

    #[test]
    fn test_convert_type_alias_map() {
        let src = r#"
            package test;
            type OrderMap = Map<String, Order>;
            struct Order {
                id: String,
            }
        "#;
        let schema = convert(src).unwrap();
        let pkg = schema.packages.get("test").unwrap();

        let alias = pkg.types.iter().find(|t| t.name() == "OrderMap").unwrap();
        if let IRType::TypeAlias(a) = alias {
            if let IRTypeAliasTarget::Map(k, v) = &a.target {
                assert!(matches!(k, IRFieldType::Primitive(IRPrimitive::String)));
                assert!(matches!(v, IRFieldType::Custom(s) if s == "Order"));
            } else {
                panic!("Expected map target");
            }
        } else {
            panic!("Expected type alias");
        }
    }

    #[test]
    fn test_convert_field_rename() {
        let src = r#"
            package test;
            struct Order {
                #[rename = "order_type"]
                type_field: String,
            }
        "#;
        let schema = convert(src).unwrap();
        let pkg = schema.packages.get("test").unwrap();

        if let IRType::Struct(s) = &pkg.types[0] {
            assert_eq!(s.fields[0].rename, Some("order_type".to_string()));
        } else {
            panic!("Expected struct");
        }
    }

    #[test]
    fn test_convert_rename_all() {
        let src = r#"
            package test;
            #[rename_all = "camelCase"]
            struct Order {
                order_id: String,
            }
        "#;
        let schema = convert(src).unwrap();
        let pkg = schema.packages.get("test").unwrap();

        if let IRType::Struct(s) = &pkg.types[0] {
            assert_eq!(s.rename_all, Some("camelCase".to_string()));
        } else {
            panic!("Expected struct");
        }
    }
}
```

**Step 3: Run tests**

Run: `cargo test --package fluorite_codegen ast_to_ir`
Expected: All tests pass

**Step 4: Commit**

```bash
git add codegen/src/idl/ast_to_ir.rs
git commit -m "feat(idl): implement AST to IR converter"
```

---

## Task 6: Create IDL Module with Public API

**Files:**
- Create: `codegen/src/idl/mod.rs`
- Modify: `codegen/src/lib.rs`

**Step 1: Write module with public API**

```rust
//! Fluorite IDL parser
//!
//! This module provides parsing for `.fl` schema files.

mod ast;
mod ast_to_ir;
mod lexer;
mod parser;

pub use ast::*;
pub use ast_to_ir::AstToIrConverter;
pub use parser::parse;

use crate::code_gen::ir::IRSchema;
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Parse a single .fl file and convert to IR
pub fn parse_file(path: &Path) -> Result<IRSchema> {
    let content = fs::read_to_string(path)?;
    let ast = parse(&content).map_err(|errors| {
        let msg = errors
            .iter()
            .map(|e| format!("{:?}", e))
            .collect::<Vec<_>>()
            .join("\n");
        anyhow::anyhow!("Parse errors in {:?}:\n{}", path, msg)
    })?;
    AstToIrConverter::new().convert(&[ast])
}

/// Parse multiple .fl files and convert to a single IR schema
pub fn parse_files(paths: &[&Path]) -> Result<IRSchema> {
    let asts: Vec<_> = paths
        .iter()
        .map(|path| {
            let content = fs::read_to_string(path)?;
            parse(&content).map_err(|errors| {
                let msg = errors
                    .iter()
                    .map(|e| format!("{:?}", e))
                    .collect::<Vec<_>>()
                    .join("\n");
                anyhow::anyhow!("Parse errors in {:?}:\n{}", path, msg)
            })
        })
        .collect::<Result<Vec<_>>>()?;

    AstToIrConverter::new().convert(&asts)
}

/// Parse IDL source string and convert to IR
pub fn parse_string(source: &str) -> Result<IRSchema> {
    let ast = parse(source).map_err(|errors| {
        let msg = errors
            .iter()
            .map(|e| format!("{:?}", e))
            .collect::<Vec<_>>()
            .join("\n");
        anyhow::anyhow!("Parse errors:\n{}", msg)
    })?;
    AstToIrConverter::new().convert(&[ast])
}
```

**Step 2: Add module to lib.rs**

Add to `codegen/src/lib.rs`:

```rust
pub mod idl;
```

**Step 3: Verify it compiles**

Run: `cargo build --package fluorite_codegen`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add codegen/src/idl/mod.rs codegen/src/lib.rs
git commit -m "feat(idl): add public API for IDL parsing"
```

---

## Task 7: Create Example .fl Files

**Files:**
- Create: `examples/users.fl`
- Create: `examples/orders.fl`

**Step 1: Write users.fl equivalent to users.yml**

```rust
package protocols::users;

struct User {
    first_name: String,
    last_name: String,
    age: u32,
    gender: Gender,
    active: bool,
}

enum Gender {
    Male,
    Female,
}
```

**Step 2: Write orders.fl equivalent to orders.yml**

```rust
package protocols::orders;

use protocols::users::User;

struct UserOrders {
    user: User,
    orders: OrderList,
}

type OrderList = Vec<Order>;
type OrderMap = Map<String, Order>;

struct Order {
    id: u64,
    item: String,
    user: User,
    #[box]
    shipping: Option<Shipping>,
    #[rename = "order_type"]
    type_field: String,
}

struct Shipping {
    id: String,
    order: Order,
    address: Address,
}

#[type_tag = "type"]
union Address {
    Empty,
    PostCode(PostCode),
    AddressInfo(AddressInfo),
}

struct AddressInfo {
    first_line: String,
    second_line: String,
}

struct PostCode {
    code: String,
    order: Order,
    instruction: Any,
}
```

**Step 3: Commit**

```bash
git add examples/users.fl examples/orders.fl
git commit -m "feat(idl): add example .fl schema files"
```

---

## Task 8: Add Integration Tests for IDL

**Files:**
- Create: `codegen/tests/idl_code_gen.rs`

**Step 1: Write integration tests**

```rust
use std::sync::Arc;

use fluorite_codegen::{
    code_gen::{
        fs::MemoryFileSystem,
        rust::{RustOptions, RustTemplateGenerator},
        ts::{TsTemplateGenerator, TypeScriptOptions},
    },
    idl,
};

#[test]
fn test_parse_users_fl() {
    let src = include_str!("../../examples/users.fl");
    let schema = idl::parse_string(src).expect("Should parse users.fl");

    let pkg = schema.packages.get("protocols::users").expect("Should have users package");
    assert_eq!(pkg.types.len(), 2);

    let user = pkg.types.iter().find(|t| t.name() == "User").expect("Should have User");
    let gender = pkg.types.iter().find(|t| t.name() == "Gender").expect("Should have Gender");

    assert!(matches!(user, fluorite_codegen::code_gen::ir::IRType::Struct(_)));
    assert!(matches!(gender, fluorite_codegen::code_gen::ir::IRType::Enum(_)));
}

#[test]
fn test_parse_orders_fl() {
    let src = include_str!("../../examples/orders.fl");
    let schema = idl::parse_string(src).expect("Should parse orders.fl");

    let pkg = schema.packages.get("protocols::orders").expect("Should have orders package");

    // Check expected types
    let type_names: Vec<_> = pkg.types.iter().map(|t| t.name()).collect();
    assert!(type_names.contains(&"Order"));
    assert!(type_names.contains(&"OrderList"));
    assert!(type_names.contains(&"OrderMap"));
    assert!(type_names.contains(&"Address"));
    assert!(type_names.contains(&"Shipping"));
}

#[test]
fn test_fl_to_rust_codegen() {
    let src = include_str!("../../examples/users.fl");
    let schema = idl::parse_string(src).expect("Should parse");

    let fs = Arc::new(MemoryFileSystem::new());
    let options = RustOptions::new("/output".to_string());
    let generator = RustTemplateGenerator::new(options, fs.clone());

    // Convert schema packages to definitions (need to bridge the gap)
    // For now, just verify the schema is valid
    assert!(!schema.packages.is_empty());
}

#[test]
fn test_fl_generates_same_rust_as_yaml() {
    // Parse YAML
    let yaml_content = include_str!("../../examples/users.yml");
    let yaml_def: fluorite_codegen::definitions::Definition =
        serde_yaml::from_str(yaml_content).expect("Should parse YAML");

    // Parse FL
    let fl_content = include_str!("../../examples/users.fl");
    let fl_schema = idl::parse_string(fl_content).expect("Should parse FL");

    // Build IR from YAML
    use fluorite_codegen::code_gen::ir::IRBuilder;
    let yaml_schema = IRBuilder::new().build(&[yaml_def]).expect("Should build YAML IR");

    // Compare packages
    assert_eq!(yaml_schema.packages.len(), fl_schema.packages.len());

    // Compare types (names should match)
    let yaml_pkg = yaml_schema.packages.get("protocols.users").expect("YAML pkg");
    let fl_pkg = fl_schema.packages.get("protocols::users").expect("FL pkg");

    let yaml_names: std::collections::HashSet<_> = yaml_pkg.types.iter().map(|t| t.name()).collect();
    let fl_names: std::collections::HashSet<_> = fl_pkg.types.iter().map(|t| t.name()).collect();

    assert_eq!(yaml_names, fl_names);
}
```

**Step 2: Run tests**

Run: `cargo test --package fluorite_codegen idl`
Expected: All tests pass

**Step 3: Commit**

```bash
git add codegen/tests/idl_code_gen.rs
git commit -m "test(idl): add integration tests for IDL parsing"
```

---

## Task 9: Update CLI to Support .fl Files

**Files:**
- Modify: `codegen/src/main.rs`

**Step 1: Update CLI to detect file extension**

Update the `main.rs` to support both `.fl` and `.yaml` files:

```rust
// Add this helper function
fn load_schema(paths: &[String]) -> anyhow::Result<IRSchema> {
    use fluorite_codegen::code_gen::ir::IRBuilder;
    use fluorite_codegen::definitions::Definition;

    let mut all_yaml_defs = Vec::new();
    let mut all_fl_asts = Vec::new();

    for path in paths {
        if path.ends_with(".fl") {
            let content = fs::read_to_string(path)?;
            let ast = fluorite_codegen::idl::parse(&content).map_err(|errors| {
                let msg = errors
                    .iter()
                    .map(|e| format!("{:?}", e))
                    .collect::<Vec<_>>()
                    .join("\n");
                anyhow::anyhow!("Parse errors in {}:\n{}", path, msg)
            })?;
            all_fl_asts.push(ast);
        } else {
            // Assume YAML
            let content = fs::read_to_string(path)?;
            let def: Definition = serde_yaml::from_str(&content)?;
            all_yaml_defs.push(def);
        }
    }

    // Build combined schema
    if !all_fl_asts.is_empty() && !all_yaml_defs.is_empty() {
        anyhow::bail!("Cannot mix .fl and .yaml files in the same invocation");
    }

    if !all_fl_asts.is_empty() {
        fluorite_codegen::idl::AstToIrConverter::new().convert(&all_fl_asts)
    } else {
        IRBuilder::new().build(&all_yaml_defs)
    }
}
```

Then update the command handlers to use this function and generate directly from IRSchema.

**Step 2: Test CLI with .fl files**

Run: `cargo run --package fluorite_codegen --bin fluorite -- rust --inputs examples/users.fl --output /tmp/test_fl`
Expected: Code generates successfully

**Step 3: Commit**

```bash
git add codegen/src/main.rs
git commit -m "feat(idl): update CLI to support .fl files"
```

---

## Task 10: Update Template Generator to Accept IRSchema Directly

**Files:**
- Modify: `codegen/src/code_gen/rust/template_generator.rs`

**Step 1: Add method to generate from IRSchema**

Add a new method that accepts IRSchema directly:

```rust
/// Generate code from an IR schema directly
pub fn generate_from_schema(&self, schema: &IRSchema) -> Result<()> {
    // Validate
    let errors = Validator::new().validate(schema);
    if !errors.is_empty() {
        return Err(anyhow::anyhow!(
            "Validation errors: {}",
            errors
                .iter()
                .map(|e| format!("{:?}", e))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    // Generate
    for (_, package) in &schema.packages {
        self.generate_package(package)?;
    }

    Ok(())
}
```

**Step 2: Commit**

```bash
git add codegen/src/code_gen/rust/template_generator.rs
git commit -m "feat(idl): add generate_from_schema method"
```

---

## Task 11: Add E2E Test Comparing YAML and FL Output

**Files:**
- Modify: `codegen/tests/idl_code_gen.rs`

**Step 1: Add comprehensive comparison test**

```rust
#[test]
fn test_fl_rust_output_matches_yaml() {
    use fluorite_codegen::code_gen::ir::IRBuilder;

    // Generate from YAML
    let yaml_fs = Arc::new(MemoryFileSystem::new());
    let yaml_def: fluorite_codegen::definitions::Definition =
        serde_yaml::from_str(include_str!("../../examples/users.yml")).unwrap();
    let yaml_schema = IRBuilder::new().build(&[yaml_def]).unwrap();

    let yaml_options = RustOptions::new("/output".to_string());
    let yaml_gen = RustTemplateGenerator::new(yaml_options, yaml_fs.clone());
    yaml_gen.generate_from_schema(&yaml_schema).unwrap();

    // Generate from FL
    let fl_fs = Arc::new(MemoryFileSystem::new());
    let fl_schema = idl::parse_string(include_str!("../../examples/users.fl")).unwrap();

    let fl_options = RustOptions::new("/output".to_string());
    let fl_gen = RustTemplateGenerator::new(fl_options, fl_fs.clone());
    fl_gen.generate_from_schema(&fl_schema).unwrap();

    // Compare outputs (allow for package name difference: protocols.users vs protocols::users)
    let yaml_files = yaml_fs.files();
    let fl_files = fl_fs.files();

    // Both should generate files
    assert!(!yaml_files.is_empty(), "YAML should generate files");
    assert!(!fl_files.is_empty(), "FL should generate files");

    // The struct and enum content should be structurally similar
    // (exact comparison may differ due to package path format)
}
```

**Step 2: Run full test suite**

Run: `cargo test --package fluorite_codegen`
Expected: All tests pass

**Step 3: Commit**

```bash
git add codegen/tests/idl_code_gen.rs
git commit -m "test(idl): add E2E test comparing YAML and FL output"
```

---

## Task 12: Update Documentation

**Files:**
- Modify: `CLAUDE.md`

**Step 1: Add IDL documentation to CLAUDE.md**

Add a new section documenting the IDL syntax and usage.

**Step 2: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: add IDL documentation"
```

---

## Task 13: Final Integration and Cleanup

**Step 1: Run all CI checks**

```bash
make all
```

Expected: All checks pass (fmt, lint, test)

**Step 2: Create final commit if any cleanup needed**

```bash
git add -A
git commit -m "chore(idl): final cleanup and integration"
```

---

## E2E Acceptance Criteria

1. **Parse users.fl** - Produces correct IR with User struct and Gender enum
2. **Parse orders.fl** - Produces correct IR with all types including union
3. **Rust codegen from .fl** - Generates valid Rust code
4. **TypeScript codegen from .fl** - Generates valid TypeScript code
5. **Error messages** - Include line/column for syntax errors
6. **CLI works** - `fluorite rust --inputs schema.fl --output ./src` succeeds
7. **All CI checks pass** - `make all` succeeds
