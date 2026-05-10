use logos::Logos;

// Posição de um token no source code. Pequeno (dois usize) — Copy é conveniente.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// Token com span, linha e coluna
#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
    pub line: usize,
    pub column: usize,
}

/// Tokens.
/// - `Newline` como token (por isso o skip ignora só espaços/tabs/carriage-return).
/// - Comentário single-line: `/: ...`
/// - Comentário multi-line: `/: ... :/` 
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r]+")] // ignora espaços, tabs e CR (mas não \n)
#[logos(skip r"/:([^\n])*")] // comentário single-line
// comentário multi-line: /: ... :/ (dot matches newline)
#[logos(skip r"/:([^:]|:[^/])*:/")]
pub enum Token {
    //palavras-chave
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
    #[token("and")]         And,
    #[token("or")]          Or,
    #[token("not")]         Not,
    #[token("true")]        True,
    #[token("false")]       False,
    #[token("null")]        Null,
    

    //enventos built-in
    #[token("start")]       Start,
    #[token("stop")]      Stop,

    //Operadores
    #[token("==")]  EqEq,
    #[token("!=")]  NotEq,
    #[token("<=")]  LtEq,
    #[token(">=")]  GtEq,
    #[token("<")]   Lt,
    #[token(">")]   Gt,
    #[token("=")]   Eq,
    #[token("+")]   Plus,
    #[token("-")]   Minus,
    #[token("*")]   Star,
    #[token("/")]   Slash,
    #[token("%")]   Percent,
    #[token("->")]  Arrow,

    //Delimatadores
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

    // Literais numéricos
    /// Número com expoente
    #[regex(r"[0-9]+(?:\.[0-9]+)?[eE][+-]?[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    Scientific(f64),

    /// Float
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    Float(f64),

    /// Inteiro
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Integer(i64),

    //string
    //fase 1: apenas string entre aspas duplas, sem suporte a escape sequences
    #[regex(r#""[^"]*""#, |lex| {
        let slice = lex.slice();
        //remover as aspas duplas
        Some(slice[1..slice.len()-1].to_string())
    })]
    StringLit(String),


    // identificador: variáveis, funções, agentes, nomes de evento
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // EOF sintético (opcional na pipeline)
    Eof,
}

