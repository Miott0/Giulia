use giulia_lexer::{lex, token::Token};

fn tokens(source: &str) -> Vec<Token> {
    lex(source)
        .expect("lex unexpectedly failed")
        .into_iter()
        .map(|st| st.token)
        .filter(|t| !matches!(t, Token::Newline | Token::Eof))
        .collect()
}

#[test]
fn keywords_reconhecidos() {
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
fn novos_keywords_fase2_parseados() {
    let result = tokens("send channel priority concurrent");
    assert_eq!(result[0], Token::Send);
    assert_eq!(result[1], Token::Channel);
    assert_eq!(result[2], Token::Priority);
    assert_eq!(result[3], Token::Concurrent);
}

#[test]
fn priority_levels_reconhecidos() {
    let result = tokens("critical high normal low");
    assert_eq!(result[0], Token::PriorityCritical);
    assert_eq!(result[1], Token::PriorityHigh);
    assert_eq!(result[2], Token::PriorityNormal);
    assert_eq!(result[3], Token::PriorityLow);
}

#[test]
fn identifier_nao_confundido_com_keyword() {
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
fn literais_numericos_com_prioridade_correta() {
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
fn operadores_dois_chars_antes_de_um() {
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
fn comentario_single_line_ignorado() {
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
fn comentario_multi_line_ignorado() {
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
fn linha_e_coluna_corretos() {
    let result = lex("let x = 1\nlet y = 2").unwrap();
    let let_tokens: Vec<_> = result.iter().filter(|st| st.token == Token::Let).collect();
    assert_eq!(let_tokens[0].line, 1);
    assert_eq!(let_tokens[1].line, 2);
}

#[test]
fn erro_com_posicao_correta() {
    let errs = lex("let x = @bad").unwrap_err();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].to_string(), "Unexpected character '@' at line 1, column 9");
}

#[test]
fn multiplos_erros_coletados() {
    let errs = lex("@a @b @c").unwrap_err();
    assert_eq!(errs.len(), 3);
}
