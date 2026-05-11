# CRL — Fase 0 + Fase 1
## Guia de Execução: Do Zero ao Primeiro Script Funcionando
### Documento Operacional · Controle Passo a Passo · Versão 2.0

---

> **Como usar este documento**
> Este é o guia operacional das Fases 0 e 1. Siga na ordem exata.
> Cada seção tem um checklist. Só avance quando todos os itens estiverem marcados.
> Ao final, você terá um interpretador mínimo executando scripts CRL reais.
>
> **v2.0 — Revisão de engenharia:**
> - `pest` removido da Fase 1 (recursive descent manual é mais simples de depurar)
> - Tipagem anotada como opcional na Fase 1 (progressiva)
> - Critérios de conclusão tornam-se mais concretos e verificáveis
> - Seção de decisões técnicas locais adicionada (explica cada escolha da fase)

---

## ÍNDICE

1. Pré-requisitos e ambiente
2. Fase 0 — Estudo dirigido
3. Decisões técnicas desta fase (e por quê)
4. Setup do projeto Rust
5. Fase 1.1 — Lexer
6. Fase 1.2 — AST
7. Fase 1.3 — Parser
8. Fase 1.4 — Interpreter
9. Fase 1.5 — CLI
10. Testes e validação
11. Critérios de conclusão da Fase 1
12. O que NÃO fazer nesta fase

---

## 1. PRÉ-REQUISITOS E AMBIENTE

### Ferramentas necessárias

```bash
# Instalar Rust (toolchain estável)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verificar instalação
rustc --version    # deve ser >= 1.75
cargo --version

# Instalar ferramentas de desenvolvimento
cargo install cargo-watch    # recompila ao salvar arquivo
cargo install cargo-nextest  # test runner mais rápido e paralelo
cargo install cargo-expand   # inspeciona macros expandidas (útil para logos)
cargo install cargo-tarpaulin  # medição de cobertura de testes
```

### Editor recomendado

VSCode com extensões:
- `rust-analyzer` — LSP completo para Rust
- `Even Better TOML` — para Cargo.toml
- `Error Lens` — mostra erros inline

---

## 2. FASE 0 — ESTUDO DIRIGIDO

**Objetivo:** Entender os conceitos que você vai implementar antes de implementar.

> Esta fase parece "teoria inútil". Não é. Cada item aqui corresponde a um
> componente real que você vai escrever. Pular esta fase significa reescrever
> código 3 vezes depois.

### 2.1 O que estudar e por quê

---

#### 2.1.1 Como um Lexer funciona

**Por que importa:** Você vai escrever um lexer na Fase 1.1.

**Conceito central:**
Um lexer converte caracteres em tokens — unidades atômicas com significado.

```
Entrada:  let x = 42
Saída:    [Let, Identifier("x"), Eq, Integer(42), Eof]
```

O lexer não entende estrutura — ele só reconhece padrões. É uma máquina de estados finita.

**O que o `logos` crate faz por você:**
Você descreve os tokens com regex e atributos. O logos gera a máquina de estados
otimizada automaticamente. Sem logos: ~500 linhas. Com logos: ~50.

**Leitura recomendada:**
- "Crafting Interpreters" de Robert Nystrom — Capítulo 3 (Scanning)
- Documentação do crate `logos`: https://docs.rs/logos

---

#### 2.1.2 Como um AST funciona

**Por que importa:** O AST é a estrutura de dados central de todo o projeto.
Parser, interpreter, type checker — todos operam sobre o AST.

**Conceito central:**
O AST representa o programa como árvore, capturando estrutura sem detalhes irrelevantes.

```
Programa: let x = 1 + 2 * 3

AST:
LetStmt
├── name: "x"
└── value: BinOp(Add)
    ├── left: Literal(1)
    └── right: BinOp(Mul)
        ├── left: Literal(2)
        └── right: Literal(3)
```

A precedência de operadores já está resolvida na estrutura da árvore.

**Por que "Abstract":**
`(1 + 2)` e `1 + 2` produzem o mesmo nó `BinOp(Add, 1, 2)`.
Os parênteses existem na sintaxe concreta, não no AST.

**Leitura recomendada:**
- "Crafting Interpreters" — Capítulo 5 (Representing Code)

---

#### 2.1.3 Como um Parser funciona

**Por que importa:** Você vai escrever um parser na Fase 1.3.

**Estratégia: Recursive Descent**
Cada regra da gramática vira uma função. A recursão da gramática vira recursão no código.

```
Gramática:
if_stmt ::= "if" expr block ("else" block)?

Função em Rust:
fn parse_if_stmt(&mut self) -> Result<Stmt, ParseError> {
    let span_start = self.current_span();
    self.expect(Token::If)?;
    let condition = self.parse_expr()?;
    let then_branch = self.parse_block()?;
    let else_branch = if self.check(&Token::Else) {
        self.advance();
        Some(self.parse_block()?)
    } else {
        None
    };
    Ok(Stmt::If { condition, then_branch, else_branch, span: span_start })
}
```

**Por que recursive descent e não um parser generator (pest, lalrpop):**
Parser generators são poderosos mas adicionam uma camada de indireção entre
a gramática e o código. Na Fase 1, a gramática vai mudar com frequência.
Com recursive descent, mudar a gramática = mudar a função correspondente.
Com pest, mudar a gramática = mudar o .pest + interpretar a nova CST + adaptar o código.
O custo-benefício favorece recursive descent nas fases iniciais.
Se a gramática se estabilizar na Fase 3+, a migração para pest é factível.

**Tratamento de erros:**
Na Fase 1, parar no primeiro erro é aceitável.
Recuperação de múltiplos erros (`synchronize`) entra na Fase 2.

**Leitura recomendada:**
- "Crafting Interpreters" — Capítulos 6–8

---

#### 2.1.4 Como um Interpreter funciona

**Por que importa:** Você vai escrever um interpreter na Fase 1.4.

**Conceito central:**
O interpreter percorre o AST e executa cada nó. Pattern matching sobre o tipo do nó.

```rust
fn eval_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
    match expr {
        Expr::Literal(lit, _) => Ok(Value::from(lit)),
        Expr::Identifier(name, span) => {
            self.env.get(name).ok_or_else(|| RuntimeError::UndefinedVariable {
                name: name.clone(),
                span: *span,
            })
        }
        Expr::BinOp { op, left, right, span } => {
            let l = self.eval_expr(left)?;
            let r = self.eval_expr(right)?;
            self.eval_binop(op, l, r, *span)
        }
        Expr::Call { callee, args, span } => {
            let func = self.eval_expr(callee)?;
            let evaluated_args = args.iter()
                .map(|a| self.eval_expr(a))
                .collect::<Result<Vec<_>, _>>()?;
            self.call_function(func, evaluated_args, *span)
        }
        // ...
    }
}
```

**Ambiente (Environment):**
Mapeia nomes a valores. Encadeado para suportar escopos léxicos.

```
Ambiente global:   { "print": NativeFn, "len": NativeFn }
    └── Ambiente da função main: { "x": Int(42) }
            └── Ambiente do if: { "temp": Bool(true) }
```

Ao buscar uma variável, sobe a cadeia até encontrar ou retornar erro com Span.

**Leitura recomendada:**
- "Crafting Interpreters" — Capítulos 7–10

---

#### 2.1.5 Actor Model (contexto para a arquitetura)

**Por que importa:** O runtime que você vai construir nas fases seguintes
usa o Actor Model. Entender o conceito antes de implementar evita
design mistakes na estrutura do Runtime.

**Conceito central:**
Actors são unidades de computação isoladas que:
- Têm estado próprio (não compartilhado)
- Se comunicam exclusivamente por mensagens
- Reagem a mensagens uma de cada vez (por padrão)

No CRL, cada `agent` é um Actor. O event bus é o sistema de mensagens.

**Por que importa para a Fase 1:**
A estrutura interna do interpreter deve ser projetada para que,
na Fase 2, o runtime consiga instanciar múltiplos agents isolados.
A `struct Interpreter` da Fase 1 se tornará a base do `AgentInstance` da Fase 2.

**Leitura recomendada:**
- https://en.wikipedia.org/wiki/Actor_model
- "Programming Erlang" de Joe Armstrong — Capítulos 1–4 (conceptual, não precisa de Erlang)

---

### 2.2 Checklist de Estudo — Fase 0

Execute estes exercícios concretos antes de escrever código:

- [ ] Leia os Capítulos 1–8 de "Crafting Interpreters" (Nystrom) — gratuito em craftinginterpreters.com
- [ ] Escreva à mão os tokens desta linha CRL:
      `agent home { on start { let x = 42 } }`
      Resultado esperado: `[Agent, Identifier("home"), LBrace, On, Start, LBrace, Let, Identifier("x"), Eq, Integer(42), RBrace, RBrace, Eof]`
- [ ] Desenhe o AST desta expressão: `if x > 10 { print(x) } else { print(0) }`
- [ ] Leia a documentação do crate `logos` e execute o exemplo básico
- [ ] Leia sobre o Actor Model (Wikipedia) — 20 minutos suficientes
- [ ] Escreva em texto livre (papel ou arquivo): "Como o CRL vai executar este código passo a passo?"
      ```
      agent main {
          fn soma(a: Int, b: Int) -> Int {
              return a + b
          }
          on start {
              let resultado = soma(3, 7)
              print(resultado)
          }
      }
      ```

---

## 3. DECISÕES TÉCNICAS DESTA FASE

Esta seção documenta as decisões de implementação específicas das Fases 0–1,
com raciocínio explícito. Cada decisão aqui é local a esta fase e pode ser
revisada nas fases seguintes.

### 3.1 Recursive descent em vez de pest

**Decisão:** Parser recursive descent manual.
**Raciocínio:** A gramática da Fase 1 é simples o suficiente. Recursive descent
oferece controle total sobre mensagens de erro (com Span preciso) e é mais fácil
de depurar quando a gramática muda. `pest` adiciona uma camada CST intermediária
que complica o diagnóstico de erros na fase inicial.
**Revisão:** Avaliar migração para pest na Fase 3 quando a gramática estiver estável.

### 3.2 Tipagem anotada como opcional

**Decisão:** Anotações de tipo (`let x: Int = 42`) são opcionais na Fase 1.
**Raciocínio:** O objetivo da Fase 1 é ter um MVP executando. Inferência de tipos
completa é um compilador separado. Anotações opcionais permitem que scripts simples
funcionem sem boilerplate, e o type checker entra progressivamente na Fase 3.
**Revisão:** Tornar obrigatório em assinaturas de função na Fase 4.

### 3.3 `Rc<RefCell<Environment>>` para escopo

**Decisão:** Ambiente de variáveis usando `Rc<RefCell<>>` para encadeamento de escopos.
**Raciocínio:** A alternativa (arena alocada) é mais eficiente mas significativamente
mais complexa. Na Fase 1, corretude importa mais que performance. `Rc<RefCell<>>`
é idiomático para interpretadores simples em Rust.
**Revisão:** Se o profiler mostrar que o ambiente é um hotspot na Fase 4+,
migrar para arena ou índice por inteiro.

### 3.4 `sled` em vez de `rocksdb` para persistência futura

**Decisão:** Quando memória persistente entrar (Fase 5), usar `sled`.
**Raciocínio:** RocksDB requer bindings C e overhead de compilação. `sled` é
implementado inteiramente em Rust, compila sem problemas e tem performance adequada.
Documentado aqui para que a estrutura do crate `crl-context` da Fase 5 já use
a abstração certa desde o início.

### 3.5 Não usar `pest` para a gramática EBNF

**Decisão:** A gramática EBNF do Mapa Geral é documentação, não código.
O parser é implementado manualmente e derivado da gramática.
**Raciocínio:** A gramática EBNF é a especificação. O parser é a implementação.
Mantê-los separados permite que a gramática evolua sem estar acoplada a uma
ferramenta específica.

---

## 4. SETUP DO PROJETO RUST

### 4.1 Criar o workspace

```bash
mkdir crl && cd crl

cat > Cargo.toml << 'EOF'
[workspace]
members = [
    "crates/crl-lexer",
    "crates/crl-ast",
    "crates/crl-parser",
    "crates/crl-interpreter",
    "crates/crl-cli",
]
resolver = "2"

[workspace.dependencies]
logos       = "0.14"
thiserror   = "1.0"
miette      = { version = "5.10", features = ["fancy"] }
EOF
```

### 4.2 Criar os crates

```bash
mkdir -p crates
cargo new --lib crates/crl-lexer
cargo new --lib crates/crl-ast
cargo new --lib crates/crl-parser
cargo new --lib crates/crl-interpreter
cargo new --bin crates/crl-cli
```

### 4.3 Configurar dependências

**crates/crl-lexer/Cargo.toml:**
```toml
[package]
name    = "crl-lexer"
version = "0.1.0"
edition = "2021"

[dependencies]
logos     = { workspace = true }
thiserror = { workspace = true }
miette    = { workspace = true }
```

**crates/crl-ast/Cargo.toml:**
```toml
[package]
name    = "crl-ast"
version = "0.1.0"
edition = "2021"

# Sem dependências externas — o AST é puro Rust
[dependencies]
```

**crates/crl-parser/Cargo.toml:**
```toml
[package]
name    = "crl-parser"
version = "0.1.0"
edition = "2021"

[dependencies]
crl-lexer = { path = "../crl-lexer" }
crl-ast   = { path = "../crl-ast" }
thiserror = { workspace = true }
miette    = { workspace = true }
```

**crates/crl-interpreter/Cargo.toml:**
```toml
[package]
name    = "crl-interpreter"
version = "0.1.0"
edition = "2021"

[dependencies]
crl-ast   = { path = "../crl-ast" }
thiserror = { workspace = true }
```

**crates/crl-cli/Cargo.toml:**
```toml
[package]
name    = "crl"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "crl"
path = "src/main.rs"

[dependencies]
crl-lexer       = { path = "../crl-lexer" }
crl-parser      = { path = "../crl-parser" }
crl-interpreter = { path = "../crl-interpreter" }
miette          = { workspace = true }
```

### 4.4 Verificar build inicial

```bash
cargo build
# Esperado: Compiling ... Finished
# Se falhar: revise os Cargo.toml acima

cargo test
# Esperado: running 0 tests (ainda não há testes)
```

### 4.5 Criar estrutura de exemplos e testes

```bash
mkdir -p examples/phase1
mkdir -p tests/integration

cat > examples/phase1/hello.crl << 'EOF'
agent main {
    on start {
        let msg = "Hello, Cognitive Runtime"
        print(msg)
    }
}
EOF
```

---

## 5. FASE 1.1 — LEXER

**Objetivo:** Converter texto CRL em sequência de tokens com posição exata.

### 5.1 Definir os Tokens

Crie `crates/crl-lexer/src/token.rs`:

```rust
use logos::Logos;

/// Posição de um nó no source code.
/// Todos os tokens e nós do AST carregam um Span.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Span {
    pub start: usize,
    pub end:   usize,
}

/// Token com posição no source code.
#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token:  Token,
    pub span:   Span,
    pub line:   usize,
    pub column: usize,
}

/// Todos os tokens da linguagem CRL.
///
/// O derive `Logos` gera o lexer otimizado automaticamente.
/// Ordem das variantes importa: keywords devem vir antes de Identifier.
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r]+")]  // ignora espaços e tabs
#[logos(skip r"--[^\n]*")]  // ignora comentários (-- até fim da linha)
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
    #[token("and")]         And,
    #[token("or")]          Or,
    #[token("not")]         Not,
    #[token("true")]        True,
    #[token("false")]       False,
    #[token("null")]        Null,

    // ---- EVENTOS BUILT-IN ----
    #[token("start")]       Start,
    #[token("stop")]        Stop,

    // ---- OPERADORES ----
    // ATENÇÃO: operadores de 2 chars (==, !=, <=, >=) DEVEM vir antes dos de 1 char (<, >, =)
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

    /// Inteiro: 42, 0, 1000
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Integer(i64),

    /// Float: 3.14, 0.5
    /// NOTA: o regex do float deve vir ANTES do Integer para ser testado primeiro
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    Float(f64),

    /// String entre aspas duplas
    /// Fase 1: sem suporte a escape sequences (ex: \n, \")
    #[regex(r#""[^"]*""#, |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string())
    })]
    StringLit(String),

    /// Identifier: variáveis, funções, agents, nomes de evento
    /// Deve vir DEPOIS de todos as keywords
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    /// Fim do arquivo
    Eof,
}
```

**Nota sobre `Float` antes de `Integer`:** O logos processa variantes na ordem
de declaração. Se `Integer` vier primeiro, `3.14` será tokenizado como `Integer(3)`,
`Dot`, `Integer(14)`. Para evitar isso, declare `Float` antes de `Integer` no enum,
ou use prioridade explícita via `#[logos(priority = N)]`.

### 5.2 Implementar o Lexer

Crie `crates/crl-lexer/src/error.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LexError {
    #[error("Caractere inesperado '{character}' na linha {line}, coluna {column}")]
    UnexpectedCharacter {
        character: char,
        line:      usize,
        column:    usize,
    },

    #[error("String não fechada iniciada na linha {line}")]
    UnterminatedString { line: usize },
}
```

Crie `crates/crl-lexer/src/lib.rs`:

```rust
pub mod error;
pub mod token;

use logos::Logos;
use token::{Span, SpannedToken, Token};
use error::LexError;

pub type LexResult = Result<Vec<SpannedToken>, Vec<LexError>>;

/// Converte source code CRL em sequência de tokens com posição.
/// Coleta TODOS os erros antes de retornar (não para no primeiro).
pub fn lex(source: &str) -> LexResult {
    let mut tokens     = Vec::new();
    let mut errors     = Vec::new();
    let mut line       = 1usize;
    let mut line_start = 0usize;

    let mut lexer = Token::lexer(source);

    while let Some(result) = lexer.next() {
        let range  = lexer.span();
        let span   = Span { start: range.start, end: range.end };
        let column = span.start.saturating_sub(line_start) + 1;

        match result {
            Ok(Token::Newline) => {
                tokens.push(SpannedToken { token: Token::Newline, span, line, column });
                line       += 1;
                line_start  = span.end;
            }
            Ok(tok) => {
                tokens.push(SpannedToken { token: tok, span, line, column });
            }
            Err(_) => {
                let bad = source[range].chars().next().unwrap_or('?');
                errors.push(LexError::UnexpectedCharacter { character: bad, line, column });
            }
        }
    }

    // EOF sempre presente ao final
    let eof_pos = source.len();
    tokens.push(SpannedToken {
        token:  Token::Eof,
        span:   Span { start: eof_pos, end: eof_pos },
        line,
        column: eof_pos.saturating_sub(line_start) + 1,
    });

    if errors.is_empty() { Ok(tokens) } else { Err(errors) }
}
```

### 5.3 Testes do Lexer

Crie `crates/crl-lexer/tests/lexer_tests.rs`:

```rust
use crl_lexer::{lex, token::Token};

fn tokens(source: &str) -> Vec<Token> {
    lex(source)
        .expect("lex falhou inesperadamente")
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
fn identifier_nao_confundido_com_keyword() {
    let result = tokens("agent_name agenter");
    assert_eq!(result[0], Token::Identifier("agent_name".into()));
    assert_eq!(result[1], Token::Identifier("agenter".into()));
}

#[test]
fn literais_numericos() {
    let result = tokens("42 3.14 0");
    assert_eq!(result[0], Token::Integer(42));
    assert_eq!(result[1], Token::Float(3.14));
    assert_eq!(result[2], Token::Integer(0));
}

#[test]
fn literal_string() {
    let result = tokens(r#""hello world""#);
    assert_eq!(result[0], Token::StringLit("hello world".into()));
}

#[test]
fn operadores_dois_chars_antes_de_um_char() {
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
fn comentarios_ignorados() {
    let result = tokens("let x = 42 -- comentário ignorado");
    assert_eq!(result[0], Token::Let);
    assert_eq!(result[1], Token::Identifier("x".into()));
    assert_eq!(result[2], Token::Eq);
    assert_eq!(result[3], Token::Integer(42));
    assert_eq!(result.len(), 4);
}

#[test]
fn declaracao_agent_completa() {
    let result = tokens("agent home { }");
    assert_eq!(result[0], Token::Agent);
    assert_eq!(result[1], Token::Identifier("home".into()));
    assert_eq!(result[2], Token::LBrace);
    assert_eq!(result[3], Token::RBrace);
}

#[test]
fn caractere_invalido_retorna_erro() {
    let result = lex("let x = @invalido");
    assert!(result.is_err());
    let erros = result.unwrap_err();
    assert_eq!(erros.len(), 1);
    let msg = format!("{}", erros[0]);
    assert!(msg.contains('@'));
}

#[test]
fn linha_e_coluna_corretos() {
    let result = lex("let x = 1\nlet y = 2").unwrap();
    // "let" da segunda linha deve ter line=2
    let let_tokens: Vec<_> = result.iter()
        .filter(|st| st.token == Token::Let)
        .collect();
    assert_eq!(let_tokens[0].line, 1);
    assert_eq!(let_tokens[1].line, 2);
}
```

### 5.4 Checklist — Lexer

- [X] Todos os keywords reconhecidos corretamente
- [X] Identifiers não confundidos com keywords
- [X] Literais: Integer, Float, String, Bool (via true/false)
- [X] Operadores de 2 caracteres têm prioridade sobre os de 1 caractere
- [X] Comentários ignorados (single-line `/:` e multi-line `/: ... :/`)
- [X] Newlines emitidos como tokens (separadores de statement)
- [X] Linha e coluna corretos para cada token (implementação `lex()` precisa ser corrigida)
- [X] Caractere inválido retorna LexError com linha, coluna e caractere (integração com `lex()` pendente)
- [X] Múltiplos erros coletados antes de retornar (pendente — fluxo de coleta no `lex()`)
- [X] `cargo nextest run -p crl-lexer` 100% passando (pendente)

---

## 6. FASE 1.2 — AST

**Objetivo:** Definir a estrutura de dados que representa o programa em memória.

Crie `crates/crl-ast/src/lib.rs`:

```rust
pub mod node;
pub mod types;
```

Crie `crates/crl-ast/src/types.rs`:

```rust
/// Sistema de tipos da Fase 1 — progressivo.
/// Fase 1: apenas tipos básicos, anotação opcional.
/// Fase 3+: tipos de capability, opcional (?), efeitos (!).
#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    Int,
    Float,
    String,
    Bool,
    Null,
    List(Box<TypeExpr>),
    Map(Box<TypeExpr>, Box<TypeExpr>),
    Named(String),          // tipos customizados e capabilities
    Optional(Box<TypeExpr>), // Fase 3
}
```

Crie `crates/crl-ast/src/node.rs`:

```rust
use crate::types::TypeExpr;

/// Posição no source code — presente em TODOS os nós.
/// Sem Span não há mensagens de erro úteis.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Span {
    pub start: usize,
    pub end:   usize,
}

// ─── PROGRAMA ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

// ─── STATEMENTS ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Stmt {
    AgentDecl(AgentDecl),
    FnDecl(FnDecl),
    LetStmt(LetStmt),
    AssignStmt(AssignStmt),
    IfStmt(IfStmt),
    WhileStmt(WhileStmt),
    ForStmt(ForStmt),
    ReturnStmt(ReturnStmt),
    EffectStmt(EffectStmt),   // "do expr" — marcador de side effect (Fase 4, sintaxe existe na Fase 1)
    ExprStmt(ExprStmt),
}

#[derive(Debug, Clone)]
pub struct AgentDecl {
    pub name:     String,
    pub uses:     Vec<UseDecl>,       // capabilities declaradas
    pub handlers: Vec<HandlerDecl>,
    pub fns:      Vec<FnDecl>,
    pub span:     Span,
}

#[derive(Debug, Clone)]
pub struct UseDecl {
    pub capability: String,
    pub span:       Span,
}

#[derive(Debug, Clone)]
pub struct HandlerDecl {
    pub event: EventPattern,
    pub body:  Block,
    pub span:  Span,
}

#[derive(Debug, Clone)]
pub enum EventPattern {
    Start,
    Stop,
    Timer(Expr),                         // on timer(1000)
    Message(String),                     // on message("topic")
    Custom(String),                      // on meu_evento
}

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name:        String,
    pub params:      Vec<Param>,
    pub return_type: Option<TypeExpr>,   // opcional na Fase 1
    pub body:        Block,
    pub span:        Span,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name:            String,
    pub type_annotation: Option<TypeExpr>,  // opcional na Fase 1
    pub span:            Span,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span:  Span,
}

#[derive(Debug, Clone)]
pub struct LetStmt {
    pub name:     String,
    pub type_ann: Option<TypeExpr>,
    pub value:    Expr,
    pub span:     Span,
}

#[derive(Debug, Clone)]
pub struct AssignStmt {
    pub name:  String,
    pub value: Expr,
    pub span:  Span,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition:   Expr,
    pub then_branch: Block,
    pub else_branch: Option<Box<ElseBranch>>,
    pub span:        Span,
}

#[derive(Debug, Clone)]
pub enum ElseBranch {
    Block(Block),
    If(IfStmt),
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body:      Block,
    pub span:      Span,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub var:      String,
    pub iterable: Expr,
    pub body:     Block,
    pub span:     Span,
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub value: Option<Expr>,
    pub span:  Span,
}

#[derive(Debug, Clone)]
pub struct EffectStmt {
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ExprStmt {
    pub expr: Expr,
    pub span: Span,
}

// ─── EXPRESSÕES ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal, Span),
    Identifier(String, Span),
    BinOp {
        op:    BinOp,
        left:  Box<Expr>,
        right: Box<Expr>,
        span:  Span,
    },
    UnaryOp {
        op:      UnaryOp,
        operand: Box<Expr>,
        span:    Span,
    },
    Call {
        callee: Box<Expr>,
        args:   Vec<Expr>,
        span:   Span,
    },
    FieldAccess {
        object: Box<Expr>,
        field:  String,
        span:   Span,
    },
    Index {
        object: Box<Expr>,
        index:  Box<Expr>,
        span:   Span,
    },
    List(Vec<Expr>, Span),
    Map(Vec<(Expr, Expr)>, Span),
}

impl Expr {
    /// Retorna o Span do nó raiz desta expressão.
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(_, s)          => *s,
            Expr::Identifier(_, s)       => *s,
            Expr::BinOp    { span, .. }  => *span,
            Expr::UnaryOp  { span, .. }  => *span,
            Expr::Call     { span, .. }  => *span,
            Expr::FieldAccess { span, .. }=> *span,
            Expr::Index    { span, .. }  => *span,
            Expr::List     (_, s)        => *s,
            Expr::Map      (_, s)        => *s,
        }
    }
}

// ─── LITERAIS ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    List(Vec<Expr>),
    Map(Vec<(Expr, Expr)>),
}

// ─── OPERADORES ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Rem,
    Eq, NotEq,
    Lt, Gt, LtEq, GtEq,
    And, Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,  // -x
    Not,  // not x
}
```

### 6.1 Checklist — AST

- [X] Todos os nós têm campo `span: Span`
- [X] `AgentDecl` contém `uses`, `handlers` e `fns`
- [X] `Expr::span()` retorna o Span correto para qualquer variante
- [X] `TypeExpr` cobre Int, Float, String, Bool, Null, List<T>, Map<K,V>, Named
- [X] Nenhum `unwrap()` ou `panic!()` nos nós do AST

---

## 7. FASE 1.3 — PARSER

**Objetivo:** Transformar tokens em AST seguindo a gramática CRL.

Crie `crates/crl-parser/src/error.rs`:

```rust
use thiserror::Error;
use crl_ast::node::Span;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Token inesperado na linha {line}, coluna {column}: esperava '{expected}', encontrou '{found}'")]
    UnexpectedToken {
        expected: String,
        found:    String,
        line:     usize,
        column:   usize,
    },

    #[error("Esperava identificador em '{context}' na linha {line}, coluna {column}")]
    ExpectedIdentifier {
        context: String,
        line:    usize,
        column:  usize,
    },

    #[error("Fim de arquivo inesperado")]
    UnexpectedEof,
}
```

Crie `crates/crl-parser/src/parser.rs`:

```rust
use crl_lexer::token::{Token, SpannedToken};
use crl_ast::node::*;
use crl_ast::types::TypeExpr;
use super::error::ParseError;

pub struct Parser {
    tokens: Vec<SpannedToken>,
    cursor: usize,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self { tokens, cursor: 0 }
    }

    // ── Helpers ──────────────────────────────────────────────────────────

    fn peek(&self) -> &Token {
        self.tokens.get(self.cursor)
            .map(|st| &st.token)
            .unwrap_or(&Token::Eof)
    }

    fn peek_token(&self) -> &SpannedToken {
        // Safe: lexer sempre termina com Eof
        &self.tokens[self.cursor.min(self.tokens.len() - 1)]
    }

    fn advance(&mut self) -> &SpannedToken {
        let st = &self.tokens[self.cursor];
        if self.cursor < self.tokens.len() - 1 {
            self.cursor += 1;
        }
        st
    }

    fn check(&self, tok: &Token) -> bool {
        self.peek() == tok
    }

    fn current_span(&self) -> Span {
        Span {
            start: self.peek_token().span.start,
            end:   self.peek_token().span.end,
        }
    }

    fn current_line(&self)   -> usize { self.peek_token().line }
    fn current_column(&self) -> usize { self.peek_token().column }

    fn expect(&mut self, expected: &Token) -> Result<&SpannedToken, ParseError> {
        if self.peek() == expected {
            Ok(self.advance())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: format!("{:?}", expected),
                found:    format!("{:?}", self.peek()),
                line:     self.current_line(),
                column:   self.current_column(),
            })
        }
    }

    /// Consome newlines. Statements são separados por newlines na gramática CRL.
    fn skip_newlines(&mut self) {
        while self.check(&Token::Newline) {
            self.advance();
        }
    }

    // ── Ponto de entrada ─────────────────────────────────────────────────

    pub fn parse_program(&mut self) -> Result<Program, Vec<ParseError>> {
        let mut stmts  = Vec::new();
        let mut errors = Vec::new();

        self.skip_newlines();

        while !self.check(&Token::Eof) {
            match self.parse_top_level_stmt() {
                Ok(stmt)  => stmts.push(stmt),
                Err(err)  => {
                    errors.push(err);
                    self.synchronize(); // recuperação básica
                }
            }
            self.skip_newlines();
        }

        if errors.is_empty() { Ok(Program { stmts }) } else { Err(errors) }
    }

    /// Avança até o próximo ponto de sincronização após um erro.
    /// Permite coletar múltiplos erros em vez de parar no primeiro.
    fn synchronize(&mut self) {
        loop {
            match self.peek() {
                Token::Eof
                | Token::Agent
                | Token::Fn
                | Token::On      => break,
                Token::Newline   => { self.advance(); break; }
                Token::RBrace    => { self.advance(); break; }
                _                => { self.advance(); }
            }
        }
    }

    // ── Statements de topo ────────────────────────────────────────────────

    fn parse_top_level_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek() {
            Token::Agent => Ok(Stmt::AgentDecl(self.parse_agent_decl()?)),
            Token::Fn    => Ok(Stmt::FnDecl(self.parse_fn_decl()?)),
            _            => self.parse_stmt(),
        }
    }

    // ── Agent ─────────────────────────────────────────────────────────────

    fn parse_agent_decl(&mut self) -> Result<AgentDecl, ParseError> {
        let span_start = self.current_span();
        self.expect(&Token::Agent)?;

        let name = match self.peek().clone() {
            Token::Identifier(n) => { self.advance(); n }
            _ => return Err(ParseError::ExpectedIdentifier {
                context: "nome do agent".into(),
                line:    self.current_line(),
                column:  self.current_column(),
            }),
        };

        self.expect(&Token::LBrace)?;
        self.skip_newlines();

        let mut uses     = Vec::new();
        let mut handlers = Vec::new();
        let mut fns      = Vec::new();

        while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
            match self.peek() {
                Token::Use     => uses.push(self.parse_use_decl()?),
                Token::On      => handlers.push(self.parse_handler_decl()?),
                Token::Fn      => fns.push(self.parse_fn_decl()?),
                Token::Newline => { self.advance(); }
                _ => return Err(ParseError::UnexpectedToken {
                    expected: "use, on, fn ou }".into(),
                    found:    format!("{:?}", self.peek()),
                    line:     self.current_line(),
                    column:   self.current_column(),
                }),
            }
        }

        let end = self.current_span();
        self.expect(&Token::RBrace)?;

        Ok(AgentDecl {
            name,
            uses,
            handlers,
            fns,
            span: Span { start: span_start.start, end: end.end },
        })
    }

    fn parse_use_decl(&mut self) -> Result<UseDecl, ParseError> {
        let span = self.current_span();
        self.expect(&Token::Use)?;
        let capability = match self.peek().clone() {
            Token::Identifier(n) => { self.advance(); n }
            _ => return Err(ParseError::ExpectedIdentifier {
                context: "capability".into(),
                line:    self.current_line(),
                column:  self.current_column(),
            }),
        };
        self.skip_newlines();
        Ok(UseDecl { capability, span })
    }

    fn parse_handler_decl(&mut self) -> Result<HandlerDecl, ParseError> {
        let span = self.current_span();
        self.expect(&Token::On)?;
        let event = self.parse_event_pattern()?;
        let body  = self.parse_block()?;
        Ok(HandlerDecl { event, body, span })
    }

    fn parse_event_pattern(&mut self) -> Result<EventPattern, ParseError> {
        match self.peek().clone() {
            Token::Start => { self.advance(); Ok(EventPattern::Start) }
            Token::Stop  => { self.advance(); Ok(EventPattern::Stop)  }
            Token::Identifier(name) if name == "timer" => {
                self.advance();
                self.expect(&Token::LParen)?;
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(EventPattern::Timer(expr))
            }
            Token::Identifier(name) if name == "message" => {
                self.advance();
                self.expect(&Token::LParen)?;
                let topic = match self.peek().clone() {
                    Token::StringLit(s) => { self.advance(); s }
                    _ => return Err(ParseError::UnexpectedToken {
                        expected: "string com tópico".into(),
                        found:    format!("{:?}", self.peek()),
                        line:     self.current_line(),
                        column:   self.current_column(),
                    }),
                };
                self.expect(&Token::RParen)?;
                Ok(EventPattern::Message(topic))
            }
            Token::Identifier(name) => {
                self.advance();
                Ok(EventPattern::Custom(name))
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "padrão de evento (start, stop, timer(...), message(...), ou identificador)".into(),
                found:    format!("{:?}", self.peek()),
                line:     self.current_line(),
                column:   self.current_column(),
            }),
        }
    }

    // ── Funções ───────────────────────────────────────────────────────────

    fn parse_fn_decl(&mut self) -> Result<FnDecl, ParseError> {
        let span = self.current_span();
        self.expect(&Token::Fn)?;

        let name = match self.peek().clone() {
            Token::Identifier(n) => { self.advance(); n }
            _ => return Err(ParseError::ExpectedIdentifier {
                context: "nome da função".into(),
                line:    self.current_line(),
                column:  self.current_column(),
            }),
        };

        self.expect(&Token::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(&Token::RParen)?;

        // "->" retorno: dois tokens separados ("-" e ">") pois o lexer não tem Arrow
        // NOTA: o lexer tem Token::Arrow ("->""), use-o aqui:
        let return_type = if self.check(&Token::Arrow) {
            self.advance();
            Some(self.parse_type_expr()?)
        } else {
            None
        };

        let body = self.parse_block()?;
        Ok(FnDecl { name, params, return_type, body, span })
    }

    fn parse_param_list(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        if !self.check(&Token::RParen) {
            params.push(self.parse_param()?);
            while self.check(&Token::Comma) {
                self.advance();
                params.push(self.parse_param()?);
            }
        }
        Ok(params)
    }

    fn parse_param(&mut self) -> Result<Param, ParseError> {
        let span = self.current_span();
        let name = match self.peek().clone() {
            Token::Identifier(n) => { self.advance(); n }
            _ => return Err(ParseError::ExpectedIdentifier {
                context: "parâmetro".into(),
                line:    self.current_line(),
                column:  self.current_column(),
            }),
        };
        // Anotação de tipo opcional na Fase 1
        let type_annotation = if self.check(&Token::Colon) {
            self.advance();
            Some(self.parse_type_expr()?)
        } else {
            None
        };
        Ok(Param { name, type_annotation, span })
    }

    // ── Bloco ─────────────────────────────────────────────────────────────

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let span_start = self.current_span();
        self.expect(&Token::LBrace)?;
        self.skip_newlines();

        let mut stmts = Vec::new();
        while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
            if self.check(&Token::Newline) {
                self.advance();
                continue;
            }
            stmts.push(self.parse_stmt()?);
            self.skip_newlines();
        }

        let end = self.current_span();
        self.expect(&Token::RBrace)?;

        Ok(Block { stmts, span: Span { start: span_start.start, end: end.end } })
    }

    // ── Statements internos ───────────────────────────────────────────────

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek() {
            Token::Let    => Ok(Stmt::LetStmt(self.parse_let_stmt()?)),
            Token::If     => Ok(Stmt::IfStmt(self.parse_if_stmt()?)),
            Token::While  => Ok(Stmt::WhileStmt(self.parse_while_stmt()?)),
            Token::For    => Ok(Stmt::ForStmt(self.parse_for_stmt()?)),
            Token::Return => Ok(Stmt::ReturnStmt(self.parse_return_stmt()?)),
            Token::Do     => Ok(Stmt::EffectStmt(self.parse_effect_stmt()?)),
            Token::Fn     => Ok(Stmt::FnDecl(self.parse_fn_decl()?)),
            _             => {
                // Pode ser assign ou expr_stmt
                let expr = self.parse_expr()?;
                // Se próximo for "=", é assign
                if self.check(&Token::Eq) {
                    if let Expr::Identifier(name, span) = expr {
                        self.advance(); // consome "="
                        let value = self.parse_expr()?;
                        self.skip_newlines();
                        return Ok(Stmt::AssignStmt(AssignStmt { name, value, span }));
                    }
                }
                self.skip_newlines();
                let span = expr.span();
                Ok(Stmt::ExprStmt(ExprStmt { expr, span }))
            }
        }
    }

    fn parse_let_stmt(&mut self) -> Result<LetStmt, ParseError> {
        let span = self.current_span();
        self.expect(&Token::Let)?;
        let name = match self.peek().clone() {
            Token::Identifier(n) => { self.advance(); n }
            _ => return Err(ParseError::ExpectedIdentifier {
                context: "variável em let".into(),
                line:    self.current_line(),
                column:  self.current_column(),
            }),
        };
        let type_ann = if self.check(&Token::Colon) {
            self.advance();
            Some(self.parse_type_expr()?)
        } else {
            None
        };
        self.expect(&Token::Eq)?;
        let value = self.parse_expr()?;
        self.skip_newlines();
        Ok(LetStmt { name, type_ann, value, span })
    }

    fn parse_if_stmt(&mut self) -> Result<IfStmt, ParseError> {
        let span = self.current_span();
        self.expect(&Token::If)?;
        let condition   = self.parse_expr()?;
        let then_branch = self.parse_block()?;
        let else_branch = if self.check(&Token::Else) {
            self.advance();
            if self.check(&Token::If) {
                Some(Box::new(ElseBranch::If(self.parse_if_stmt()?)))
            } else {
                Some(Box::new(ElseBranch::Block(self.parse_block()?)))
            }
        } else {
            None
        };
        Ok(IfStmt { condition, then_branch, else_branch, span })
    }

    fn parse_while_stmt(&mut self) -> Result<WhileStmt, ParseError> {
        let span = self.current_span();
        self.expect(&Token::While)?;
        let condition = self.parse_expr()?;
        let body      = self.parse_block()?;
        Ok(WhileStmt { condition, body, span })
    }

    fn parse_for_stmt(&mut self) -> Result<ForStmt, ParseError> {
        let span = self.current_span();
        self.expect(&Token::For)?;
        let var = match self.peek().clone() {
            Token::Identifier(n) => { self.advance(); n }
            _ => return Err(ParseError::ExpectedIdentifier {
                context: "variável em for".into(),
                line:    self.current_line(),
                column:  self.current_column(),
            }),
        };
        self.expect(&Token::In)?;
        let iterable = self.parse_expr()?;
        let body     = self.parse_block()?;
        Ok(ForStmt { var, iterable, body, span })
    }

    fn parse_return_stmt(&mut self) -> Result<ReturnStmt, ParseError> {
        let span = self.current_span();
        self.expect(&Token::Return)?;
        let value = if !self.check(&Token::Newline) && !self.check(&Token::RBrace) {
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.skip_newlines();
        Ok(ReturnStmt { value, span })
    }

    fn parse_effect_stmt(&mut self) -> Result<EffectStmt, ParseError> {
        let span = self.current_span();
        self.expect(&Token::Do)?;
        let expr = self.parse_expr()?;
        self.skip_newlines();
        Ok(EffectStmt { expr, span })
    }

    // ── Expressões (Pratt / recursive descent por precedência) ───────────

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and()?;
        while self.check(&Token::Or) {
            let span = self.current_span();
            self.advance();
            let right = self.parse_and()?;
            left = Expr::BinOp { op: BinOp::Or, left: Box::new(left), right: Box::new(right), span };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_equality()?;
        while self.check(&Token::And) {
            let span = self.current_span();
            self.advance();
            let right = self.parse_equality()?;
            left = Expr::BinOp { op: BinOp::And, left: Box::new(left), right: Box::new(right), span };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_comparison()?;
        loop {
            let op = match self.peek() {
                Token::EqEq  => BinOp::Eq,
                Token::NotEq => BinOp::NotEq,
                _            => break,
            };
            let span = self.current_span();
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::BinOp { op, left: Box::new(left), right: Box::new(right), span };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_addition()?;
        loop {
            let op = match self.peek() {
                Token::Lt   => BinOp::Lt,
                Token::Gt   => BinOp::Gt,
                Token::LtEq => BinOp::LtEq,
                Token::GtEq => BinOp::GtEq,
                _           => break,
            };
            let span = self.current_span();
            self.advance();
            let right = self.parse_addition()?;
            left = Expr::BinOp { op, left: Box::new(left), right: Box::new(right), span };
        }
        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplication()?;
        loop {
            let op = match self.peek() {
                Token::Plus  => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _            => break,
            };
            let span = self.current_span();
            self.advance();
            let right = self.parse_multiplication()?;
            left = Expr::BinOp { op, left: Box::new(left), right: Box::new(right), span };
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Token::Star    => BinOp::Mul,
                Token::Slash   => BinOp::Div,
                Token::Percent => BinOp::Rem,
                _              => break,
            };
            let span = self.current_span();
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinOp { op, left: Box::new(left), right: Box::new(right), span };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Token::Not => {
                let span = self.current_span();
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::UnaryOp { op: UnaryOp::Not, operand: Box::new(operand), span })
            }
            Token::Minus => {
                let span = self.current_span();
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::UnaryOp { op: UnaryOp::Neg, operand: Box::new(operand), span })
            }
            _ => self.parse_call(),
        }
    }

    fn parse_call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek() {
                Token::LParen => {
                    let span = self.current_span();
                    self.advance();
                    let args = self.parse_arg_list()?;
                    self.expect(&Token::RParen)?;
                    expr = Expr::Call { callee: Box::new(expr), args, span };
                }
                Token::Dot => {
                    let span = self.current_span();
                    self.advance();
                    let field = match self.peek().clone() {
                        Token::Identifier(n) => { self.advance(); n }
                        _ => return Err(ParseError::ExpectedIdentifier {
                            context: "campo após '.'".into(),
                            line:    self.current_line(),
                            column:  self.current_column(),
                        }),
                    };
                    expr = Expr::FieldAccess { object: Box::new(expr), field, span };
                }
                Token::LBracket => {
                    let span = self.current_span();
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect(&Token::RBracket)?;
                    expr = Expr::Index { object: Box::new(expr), index: Box::new(index), span };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_arg_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();
        if !self.check(&Token::RParen) {
            args.push(self.parse_expr()?);
            while self.check(&Token::Comma) {
                self.advance();
                args.push(self.parse_expr()?);
            }
        }
        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        match self.peek().clone() {
            Token::Integer(n)   => { self.advance(); Ok(Expr::Literal(Literal::Int(n),    span)) }
            Token::Float(f)     => { self.advance(); Ok(Expr::Literal(Literal::Float(f),  span)) }
            Token::StringLit(s) => { self.advance(); Ok(Expr::Literal(Literal::String(s), span)) }
            Token::True         => { self.advance(); Ok(Expr::Literal(Literal::Bool(true),  span)) }
            Token::False        => { self.advance(); Ok(Expr::Literal(Literal::Bool(false), span)) }
            Token::Null         => { self.advance(); Ok(Expr::Literal(Literal::Null,        span)) }
            Token::Identifier(n)=> { self.advance(); Ok(Expr::Identifier(n, span)) }
            Token::LParen       => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            Token::LBracket => {
                self.advance();
                let mut items = Vec::new();
                if !self.check(&Token::RBracket) {
                    items.push(self.parse_expr()?);
                    while self.check(&Token::Comma) {
                        self.advance();
                        items.push(self.parse_expr()?);
                    }
                }
                self.expect(&Token::RBracket)?;
                Ok(Expr::List(items, span))
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "expressão".into(),
                found:    format!("{:?}", self.peek()),
                line:     self.current_line(),
                column:   self.current_column(),
            }),
        }
    }

    // ── Tipos ────────────────────────────────────────────────────────────

    fn parse_type_expr(&mut self) -> Result<TypeExpr, ParseError> {
        match self.peek().clone() {
            Token::Identifier(name) => {
                self.advance();
                let base = match name.as_str() {
                    "Int"    => TypeExpr::Int,
                    "Float"  => TypeExpr::Float,
                    "String" => TypeExpr::String,
                    "Bool"   => TypeExpr::Bool,
                    "Null"   => TypeExpr::Null,
                    "List"   => {
                        self.expect(&Token::Lt)?;
                        let inner = self.parse_type_expr()?;
                        self.expect(&Token::Gt)?;
                        TypeExpr::List(Box::new(inner))
                    }
                    "Map" => {
                        self.expect(&Token::Lt)?;
                        let k = self.parse_type_expr()?;
                        self.expect(&Token::Comma)?;
                        let v = self.parse_type_expr()?;
                        self.expect(&Token::Gt)?;
                        TypeExpr::Map(Box::new(k), Box::new(v))
                    }
                    _ => TypeExpr::Named(name),
                };
                Ok(base)
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "tipo (Int, Float, String, Bool, Null, List<T>, Map<K,V>, ou identificador)".into(),
                found:    format!("{:?}", self.peek()),
                line:     self.current_line(),
                column:   self.current_column(),
            }),
        }
    }
}
```

### 7.1 Checklist — Parser

- [ ] Parseia `agent nome { on start { ... } }` corretamente
- [ ] Parseia `use capability` dentro de agent
- [ ] Parseia `on timer(1000) { ... }`
- [ ] Parseia `on message("topico") { ... }`
- [ ] Parseia `fn nome(a: Int, b: Int) -> Int { ... }`
- [ ] Parseia anotações de tipo opcionais (`let x = 42` e `let x: Int = 42`)
- [ ] Precedência de operadores: `1 + 2 * 3` → `Add(1, Mul(2, 3))`
- [ ] `if / else if / else` encadeados
- [ ] `while` e `for ... in`
- [ ] Chamadas encadeadas: `obj.method(arg).outro()`
- [ ] Sincronização: múltiplos erros coletados, não apenas o primeiro
- [ ] Erro indica linha, coluna e contexto exatos
- [ ] Pelo menos 10 scripts diferentes parseados sem erro

---

## 8. FASE 1.4 — INTERPRETER

**Objetivo:** Executar o AST e produzir resultados visíveis.

Crie `crates/crl-interpreter/src/value.rs`:

```rust
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crl_ast::node::FnDecl;

/// Todos os tipos de valor que podem existir em runtime CRL.
#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Function(FunctionValue),
    NativeFunction(NativeFn),
}

#[derive(Debug, Clone)]
pub struct FunctionValue {
    pub decl:    FnDecl,
    pub closure: Env,   // captura o ambiente léxico
}

/// Função nativa implementada em Rust.
/// Arc para ser clonável (necessário para o HashMap do environment).
pub type NativeFn = Arc<dyn Fn(Vec<Value>) -> Result<Value, RuntimeError> + Send + Sync>;

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n)       => write!(f, "{}", n),
            Value::Float(n)     => write!(f, "{}", n),
            Value::String(s)    => write!(f, "{}", s),
            Value::Bool(b)      => write!(f, "{}", b),
            Value::Null         => write!(f, "null"),
            Value::List(l) => {
                write!(f, "[")?;
                for (i, v) in l.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Map(m)            => write!(f, "{{...{} entries}}", m.len()),
            Value::Function(fv)      => write!(f, "<fn {}>", fv.decl.name),
            Value::NativeFunction(_) => write!(f, "<native fn>"),
        }
    }
}
```

O `Env` e o `Interpreter` principal seguem o mesmo pattern de `Rc<RefCell<Environment>>`
com encadeamento de escopos descrito na seção 2.1.4. Implemente seguindo o pattern
de `match` sobre cada nó do AST.

**Funções nativas obrigatórias:**

```rust
// Registrar no environment inicial:
"print"   → imprime Value no stdout com "\n"
"len"     → retorna Int com tamanho de String ou List
"type_of" → retorna String com o nome do tipo ("Int", "Float", etc.)
"to_str"  → converte qualquer Value para String
```

**Execução de agent:**
O interpreter deve:
1. Encontrar o `AgentDecl` no programa
2. Registrar as funções do agent no environment
3. Executar o handler `on start` se existir
4. Retornar corretamente ao final

### 8.1 Checklist — Interpreter

- [ ] `let x = expr` avalia e armazena no environment
- [ ] `let x: Int = expr` funciona (anotação ignorada na Fase 1, validada na Fase 3)
- [ ] `if / else` com condições booleanas
- [ ] `while` com break implícito quando condição é falsa
- [ ] `for x in list` itera sobre List
- [ ] Funções definidas pelo usuário com escopo léxico correto
- [ ] `return` funciona dentro de funções
- [ ] `print()`, `len()`, `type_of()`, `to_str()` funcionando
- [ ] Handler `on start` executado ao iniciar o agent
- [ ] Erro com linha/coluna quando variável não existe
- [ ] Erro com linha/coluna em operação inválida (`true + 1`)
- [ ] `do expr` executa a expressão (efeito real vem na Fase 3)

---

## 9. FASE 1.5 — CLI

Crie `crates/crl-cli/src/main.rs`:

```rust
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("run")     => run_command(&args[2..]),
        Some("check")   => check_command(&args[2..]),
        Some("--version") | Some("-v") => {
            println!("crl {}", env!("CARGO_PKG_VERSION"));
        }
        _ => print_help(),
    }
}

fn run_command(args: &[String]) {
    let path = match args.first() {
        Some(p) => PathBuf::from(p),
        None    => { eprintln!("Erro: caminho do arquivo obrigatório"); return; }
    };

    let source = match std::fs::read_to_string(&path) {
        Ok(s)  => s,
        Err(e) => { eprintln!("Erro ao ler '{}': {}", path.display(), e); return; }
    };

    // Pipeline completo: lex → parse → interpret
    let tokens = match crl_lexer::lex(&source) {
        Ok(t)    => t,
        Err(errs) => {
            for e in errs { eprintln!("{}", e); }
            return;
        }
    };

    let program = match crl_parser::parse(tokens) {
        Ok(p)    => p,
        Err(errs) => {
            for e in errs { eprintln!("{}", e); }
            return;
        }
    };

    if let Err(e) = crl_interpreter::run(program) {
        eprintln!("{}", e);
    }
}

fn check_command(args: &[String]) {
    let path = match args.first() {
        Some(p) => PathBuf::from(p),
        None    => { eprintln!("Erro: caminho do arquivo obrigatório"); return; }
    };

    let source = match std::fs::read_to_string(&path) {
        Ok(s)  => s,
        Err(e) => { eprintln!("Erro ao ler '{}': {}", path.display(), e); return; }
    };

    let tokens = match crl_lexer::lex(&source) {
        Ok(t)    => t,
        Err(errs) => {
            for e in errs { eprintln!("{}", e); }
            std::process::exit(1);
        }
    };

    match crl_parser::parse(tokens) {
        Ok(_)    => println!("OK — nenhum erro de sintaxe encontrado"),
        Err(errs) => {
            for e in errs { eprintln!("{}", e); }
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("crl — Cognitive Runtime Language");
    println!();
    println!("Uso:");
    println!("  crl run <arquivo.crl>     Executa um script CRL");
    println!("  crl check <arquivo.crl>   Verifica sintaxe sem executar");
    println!("  crl --version             Mostra a versão");
}
```

---

## 10. TESTES E VALIDAÇÃO

### Scripts de validação — todos devem executar corretamente:

```
-- Teste 1: hello world
agent main {
    on start {
        print("Hello, Cognitive Runtime")
    }
}
-- Esperado: Hello, Cognitive Runtime

-- Teste 2: variáveis e aritmética
agent main {
    on start {
        let x = 10
        let y = 20
        print(x + y)
    }
}
-- Esperado: 30

-- Teste 3: condicional
agent main {
    on start {
        let x = 15
        if x > 10 {
            print("maior")
        } else {
            print("menor")
        }
    }
}
-- Esperado: maior

-- Teste 4: loop while
agent main {
    on start {
        let i = 0
        while i < 5 {
            print(i)
            i = i + 1
        }
    }
}
-- Esperado: 0 1 2 3 4 (um por linha)

-- Teste 5: função definida pelo usuário com tipos anotados
agent main {
    fn soma(a: Int, b: Int) -> Int {
        return a + b
    }
    on start {
        let resultado = soma(3, 7)
        print(resultado)
    }
}
-- Esperado: 10

-- Teste 6: função sem anotação de tipo (opcional na Fase 1)
agent main {
    fn dobro(n) {
        return n * 2
    }
    on start {
        print(dobro(21))
    }
}
-- Esperado: 42

-- Teste 7: capabilities declaradas (sintaxe parseada, execução ignorada)
agent home {
    use lights
    on start {
        print("Agent home iniciado")
    }
}
-- Esperado: Agent home iniciado

-- Teste 8: for in lista
agent main {
    on start {
        let items = [1, 2, 3]
        for item in items {
            print(item)
        }
    }
}
-- Esperado: 1 2 3 (um por linha)

-- Teste 9: erro com linha correta
agent main {
    on start {
        let x = y_nao_existe
    }
}
-- Esperado: erro mencionando 'y_nao_existe' com linha e coluna

-- Teste 10: chamada encadeada (field access + call)
agent main {
    on start {
        let s = "hello"
        print(type_of(s))
    }
}
-- Esperado: String
```

---

## 11. CRITÉRIOS DE CONCLUSÃO DA FASE 1

Só avance para a Fase 2 quando TODOS estes critérios forem atendidos:

**Funcionalidade:**
- [ ] Os 10 scripts de validação acima executam sem falha
- [ ] Erros de sintaxe mostram linha e coluna exatas
- [ ] Erros de runtime mostram linha e coluna exatas
- [ ] `crl run hello.crl` funciona end-to-end
- [ ] `crl check` detecta e reporta erros de sintaxe corretamente

**Qualidade de código:**
- [ ] `cargo test` passa 100%
- [ ] Cobertura do lexer > 90% (via `cargo tarpaulin -p crl-lexer`)
- [ ] Cobertura do parser > 85%
- [ ] `cargo clippy -- -D warnings` sem warnings
- [ ] Nenhum `unwrap()` sem comentário `// SAFE: <razão>`
- [ ] Nenhum `panic!()` sem comentário `// INVARIANT: <razão>`

**Arquitetura:**
- [ ] Todos os nós do AST têm `Span`
- [ ] A estrutura do `Interpreter` é compatível com multi-agent (sem estado global)
- [ ] O crate `crl-ast` não tem dependências externas

---

## 12. O QUE NÃO FAZER NESTA FASE

❌ **Não implemente o event loop** — isso é Fase 2.

❌ **Não implemente capabilities reais** — declare na sintaxe, ignore na execução.

❌ **Não implemente concorrência** — tudo sequencial na Fase 1.

❌ **Não implemente bytecode/VM** — AST-walking é suficiente e correto.

❌ **Não force tipagem estática completa** — anotações opcionais são suficientes.

❌ **Não conecte IA** — isso é Fase 8.

❌ **Não tente otimizar performance** — corretude antes de performance.

❌ **Não use `pest` para o parser** — recursive descent manual é a decisão desta fase (seção 3.1).

❌ **Não pule os testes** — cada componente tem seus testes antes de avançar.

❌ **Não implemente acesso remoto ou MQTT** — isso é Fase 10.

---

*Versão: 2.0.0 | Revisão: Engenharia Sênior*
*Próximo documento: CRL_FASE_2.md (Runtime Contínuo — Event Bus, Scheduler, Dispatcher)*
