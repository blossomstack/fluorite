//! Lexer for the Fluorite IDL using logos

use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq, Eq, Hash)]
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
    #[token(".")]
    Dot,
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
            Token::Dot => write!(f, "."),
            Token::Comma => write!(f, ","),
            Token::Eq => write!(f, "="),
            Token::Hash => write!(f, "#"),
            Token::Ident(s) => write!(f, "{}", s),
            Token::StringLit(s) => write!(f, "\"{}\"", s),
            Token::DocComment(s) => write!(f, "/// {}", s),
            Token::Comment => write!(f, "// ..."),
            Token::TyString
            | Token::TyBool
            | Token::TyI32
            | Token::TyI64
            | Token::TyU32
            | Token::TyU64
            | Token::TyF32
            | Token::TyF64
            | Token::TyOption
            | Token::TyVec
            | Token::TyMap
            | Token::TyAny
            | Token::TyUuid
            | Token::TyDecimal
            | Token::TyBytes
            | Token::TyUrl
            | Token::TyDateTime
            | Token::TyDateTimeUtc
            | Token::TyDateTimeTz
            | Token::TyDate
            | Token::TyTime
            | Token::TyDuration
            | Token::TyTimestamp
            | Token::TyTimestampMillis => write!(f, "{:?}", self),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;

    fn lex(input: &str) -> Vec<Token> {
        Token::lexer(input).filter_map(|r| r.ok()).collect()
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
        let tokens = lex("{ } ( ) < > [ ] ; : . , = #");
        assert_eq!(
            tokens,
            vec![
                Token::LBrace,
                Token::RBrace,
                Token::LParen,
                Token::RParen,
                Token::LAngle,
                Token::RAngle,
                Token::LBracket,
                Token::RBracket,
                Token::Semi,
                Token::Colon,
                Token::Dot,
                Token::Comma,
                Token::Eq,
                Token::Hash,
            ]
        );
    }

    #[test]
    fn test_dot_token() {
        assert_eq!(lex("."), vec![Token::Dot]);
    }

    #[test]
    fn test_dotted_identifier() {
        let tokens = lex("foo.bar");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("foo".to_string()),
                Token::Dot,
                Token::Ident("bar".to_string()),
            ]
        );
    }

    #[test]
    fn test_dotted_path() {
        let tokens = lex("com.example.users");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("com".to_string()),
                Token::Dot,
                Token::Ident("example".to_string()),
                Token::Dot,
                Token::Ident("users".to_string()),
            ]
        );
    }

    #[test]
    fn test_identifier() {
        assert_eq!(lex("foo"), vec![Token::Ident("foo".to_string())]);
        assert_eq!(lex("_bar"), vec![Token::Ident("_bar".to_string())]);
        assert_eq!(
            lex("MyType123"),
            vec![Token::Ident("MyType123".to_string())]
        );
    }

    #[test]
    fn test_string_literal() {
        assert_eq!(
            lex(r#""hello""#),
            vec![Token::StringLit("hello".to_string())]
        );
        assert_eq!(
            lex(r#""camelCase""#),
            vec![Token::StringLit("camelCase".to_string())]
        );
    }

    #[test]
    fn test_doc_comment() {
        assert_eq!(
            lex("/// This is a doc"),
            vec![Token::DocComment("This is a doc".to_string())]
        );
    }

    #[test]
    fn test_package_statement() {
        let tokens = lex("package orders;");
        assert_eq!(
            tokens,
            vec![
                Token::Package,
                Token::Ident("orders".to_string()),
                Token::Semi,
            ]
        );
    }

    #[test]
    fn test_use_statement() {
        let tokens = lex("use com.example.users.User;");
        assert_eq!(
            tokens,
            vec![
                Token::Use,
                Token::Ident("com".to_string()),
                Token::Dot,
                Token::Ident("example".to_string()),
                Token::Dot,
                Token::Ident("users".to_string()),
                Token::Dot,
                Token::Ident("User".to_string()),
                Token::Semi,
            ]
        );
    }
}
