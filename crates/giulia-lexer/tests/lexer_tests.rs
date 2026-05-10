use giulia_lexer::{lex, token::Token};

fn tokens(source: &str) -> Vec<Token> {
    lex(source)
        .expect("lex unexpectedly failed")
        .into_iter()
        .map(|st| st.token)
        .filter(|t| !matches!(t, Token::Newline | Token::Eof)) // Ignore Newline and Eof for tests
        .collect()
}

#[test]
fn keywords_recognized() {
    let result = tokens("agent on fn let return if else while for in use do");
    assert_eq!(result[0],  Token::Agent);
    assert_eq!(result[1],  Token::On);
    assert_eq!(result[2],  Token::Fn);
    assert_eq!(result[3],  Token::Let);
    assert_eq!(result[4],  Token::Return);
    assert_eq!(result[5],  Token::If);
    assert_eq!(result[6],  Token::Else);
    assert_eq!(result[7],  Token::While);
    assert_eq!(result[8],  Token::For);
    assert_eq!(result[9],  Token::In);
    assert_eq!(result[10], Token::Use);
    assert_eq!(result[11], Token::Do);
}

#[test]
fn identifier_not_confused_with_keyword() {
    let result = tokens("agent_name agenter");
    assert_eq!(result[0], Token::Identifier("agent_name".into()));
    assert_eq!(result[1], Token::Identifier("agenter".into()));
}

#[test]
fn numeric_literals_basic() {
    let result = tokens("42 3.14 0");
    assert_eq!(result[0], Token::Integer(42));
    assert_eq!(result[1], Token::Float(3.14));
    assert_eq!(result[2], Token::Integer(0));
}

#[test]
fn scientific_numbers() {
    let toks = tokens("42 3.14 1e4 1.23E-4");
    match &toks[0] { Token::Integer(n) => assert_eq!(*n, 42), _ => panic!("expected Integer") }
    match &toks[1] { Token::Float(f) => assert!((f - 3.14).abs() < 1e-12), _ => panic!("expected Float") }
    match &toks[2] { Token::Scientific(f) => assert!((f - 1e4).abs() < 1e-9), _ => panic!("expected Scientific") }
    match &toks[3] { Token::Scientific(f) => assert!((f - 1.23e-4).abs() < 1e-15), _ => panic!("expected Scientific") }
}

#[test]
fn string_literal() {
    let result = tokens(r#""hello world""#);
    assert_eq!(result[0], Token::StringLit("hello world".into()));
}

#[test]
fn two_char_operators_before_one_char() {
    let result = tokens("== != <= >= < > =");
    assert_eq!(result[0], Token::EqEq);
    assert_eq!(result[1], Token::NotEq);
    assert_eq!(result[2], Token::LtEq);
    assert_eq!(result[3], Token::GtEq);
    assert_eq!(result[4], Token::Lt);
    assert_eq!(result[5], Token::Gt);
    assert_eq!(result[6], Token::Eq);
}

#[test]
fn comments_ignored_single_line() {
    let result = tokens("let x = 42 /: comment\nlet y = 2");
    assert_eq!(result[0], Token::Let);
    assert_eq!(result[1], Token::Identifier("x".into()));
    assert_eq!(result[2], Token::Eq);
    assert_eq!(result[3], Token::Integer(42));
    assert_eq!(result[4], Token::Let);
    assert_eq!(result[5], Token::Identifier("y".into()));
    assert_eq!(result[6], Token::Eq);
    assert_eq!(result[7], Token::Integer(2));
}

#[test]
fn comments_ignored_multi_line() {
    let src = "let x = 1 /: comment\nmultiline\nuntil here :/ let y = 2";
    let result = tokens(src);
    assert_eq!(result[0], Token::Let);
    assert_eq!(result[1], Token::Identifier("x".into()));
    assert_eq!(result[2], Token::Eq);
    assert_eq!(result[3], Token::Integer(1));
    assert_eq!(result[4], Token::Let);
    assert_eq!(result[5], Token::Identifier("y".into()));
    assert_eq!(result[6], Token::Eq);
    assert_eq!(result[7], Token::Integer(2));
}

#[test]
fn invalid_character_returns_error() {
    let result = lex("let x = @invalid");
    assert!(result.is_err());
}

#[test]
fn line_and_column_correct() {
    let result = lex("let x = 1\nlet y = 2").unwrap();
    let let_tokens: Vec<_> = result.iter().filter(|st| st.token == Token::Let).collect();
    assert_eq!(let_tokens[0].line, 1);
    assert_eq!(let_tokens[1].line, 2);
}