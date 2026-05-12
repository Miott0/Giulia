use logos::Logos;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
    pub line: usize,
    pub column: usize,
}

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r]+")]
#[logos(skip r"/:[^\n]*")]
#[logos(skip r"/:([^:]|:[^/])*:/")]
pub enum Token {
    // ---- PALAVRAS-CHAVE ----
    #[token("agent")]       Agent,
    #[token("on")]          On,
    #[token("fn")]          Fn,
    #[token("let")]         Let,
    #[token("return")]      Return,
    #[token("if")]          If,
    #[token("else")]        Else,
    #[token("while")]       While,
    #[token("for")]         For,
    #[token("in")]          In,
    #[token("use")]         Use,
    #[token("do")]          Do,
    #[token("send")]        Send,
    #[token("and")]         And,
    #[token("or")]          Or,
    #[token("not")]         Not,
    #[token("true")]        True,
    #[token("false")]       False,
    #[token("null")]        Null,
    #[token("channel")]     Channel,
    #[token("priority")]    Priority,
    #[token("concurrent")]  Concurrent,

    // ---- EVENTOS BUILT-IN ----
    #[token("start")]       Start,
    #[token("stop")]        Stop,
    #[token("on_error")]    OnError,
    #[token("event_policy")] EventPolicy,
    #[token("ai_policy")]   AiPolicy,

    // ---- NÍVEIS DE PRIORIDADE ----
    #[token("critical")]    PriorityCritical,
    #[token("high")]        PriorityHigh,
    #[token("normal")]      PriorityNormal,
    #[token("low")]         PriorityLow,

    // ---- OPERADORES ----
    #[token("==")]  EqEq,
    #[token("!=")]  NotEq,
    #[token("<=")]  LtEq,
    #[token(">=")]  GtEq,
    #[token("->")]  Arrow,
    #[token("<")]   Lt,
    #[token(">")]   Gt,
    #[token("=")]   Eq,
    #[token("+")]   Plus,
    #[token("-")]   Minus,
    #[token("*")]   Star,
    #[token("/")]   Slash,
    #[token("%")]   Percent,

    // ---- DELIMITADORES ----
    #[token("{")]   LBrace,
    #[token("}")]   RBrace,
    #[token("(")]   LParen,
    #[token(")")]   RParen,
    #[token("[")]   LBracket,
    #[token("]")]   RBracket,
    #[token(",")]   Comma,
    #[token(".")]   Dot,
    #[token(":")]   Colon,
    #[token("\n")]  Newline,

    // ---- LITERAIS ----
    #[regex(r"[0-9]+(?:\.[0-9]+)?[eE][+-]?[0-9]+",
        |lex| lex.slice().parse::<f64>().ok())]
    Scientific(f64),

    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    Float(f64),

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Integer(i64),

    #[regex(r#""[^"]*""#, |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string())
    })]
    StringLit(String),

    /// f-string: f"..." — conteúdo bruto entre aspas (sem o prefixo f).
    #[regex(r#"f"[^"]*""#, |lex| {
        let s = lex.slice();
        Some(s[2..s.len()-1].to_string())
    })]
    FString(String),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    Eof,
}

