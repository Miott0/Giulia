# CRL — Fase 0 + Fase 1
## Guia de Execução: Do Zero ao Primeiro Script Funcionando
### Documento Operacional · Controle Passo a Passo · Versão 3.0

---

> **Como usar este documento**
> Guia operacional das Fases 0 e 1. Siga na ordem exata.
> Cada seção tem checklist. Só avance quando todos os itens estiverem marcados.
> Ao final, você terá um interpretador mínimo executando scripts CRL reais,
> com a estrutura interna pronta para as fases seguintes sem refatoração.
>
> **v3.0 — O que mudou:**
> - `AgentDecl` no AST agora inclui `ErrorPolicy`, `EventPolicy`, `ChannelDecl`
>   (parseados mas ignorados na execução — fundação para Fases 2 e 4)
> - `AgentDefinition` e `AgentInstance` definidos como structs desde a Fase 1
> - Seção de armadilhas comuns adicionada com base nas lacunas auditadas
> - Testes de validação expandidos para 12 scripts

---

## ÍNDICE

1. Pré-requisitos e ambiente
2. Fase 0 — Estudo dirigido
3. Decisões técnicas desta fase
4. Setup do projeto Rust
5. Fase 1.1 — Lexer
6. Fase 1.2 — AST
7. Fase 1.3 — Parser
8. Fase 1.4 — Interpreter
9. Fase 1.5 — CLI
10. Testes e validação
11. Critérios de conclusão da Fase 1
12. O que NÃO fazer nesta fase
13. Armadilhas comuns e como evitá-las

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

# Ferramentas de desenvolvimento
cargo install cargo-watch      # recompila ao salvar
cargo install cargo-nextest    # test runner mais rápido e paralelo
cargo install cargo-expand     # inspeciona macros expandidas (útil para logos)
cargo install cargo-tarpaulin  # cobertura de testes
```

### Editor recomendado

VSCode com:
- `rust-analyzer` — LSP completo
- `Even Better TOML` — para Cargo.toml
- `Error Lens` — erros inline

---

## 2. FASE 0 — ESTUDO DIRIGIDO

**Objetivo:** Entender os conceitos que você vai implementar antes de implementar.

> Cada item aqui corresponde a um componente real que você vai escrever.
> Pular esta fase significa reescrever código nas fases seguintes.

### 2.1 O que estudar e por quê

---

#### 2.1.1 Como um Lexer funciona

**Por que importa:** Você vai escrever um lexer na Fase 1.1.

Um lexer converte caracteres em tokens — unidades atômicas com significado.
É uma máquina de estados finita: em cada momento está em um estado
(lendo string, lendo número, lendo identifier...) e transita conforme lê.

```
Entrada:  let x = 42
Saída:    [Let, Identifier("x"), Eq, Integer(42), Eof]
```

O lexer não entende estrutura — ele apenas reconhece padrões.

**O que o `logos` crate faz por você:**
Você descreve os tokens com regex e derives. O logos gera a máquina de
estados otimizada automaticamente. Sem logos: ~500 linhas. Com logos: ~50.

**Leitura:** "Crafting Interpreters" (Nystrom) — Capítulo 3 (Scanning).
Docs do crate `logos`: https://docs.rs/logos

---

#### 2.1.2 Como um AST funciona

**Por que importa:** O AST é a estrutura de dados central de todo o projeto.
Parser, interpreter, type checker, supervisor — todos operam sobre o AST.

O AST representa o programa como árvore, capturando estrutura sem detalhes
irrelevantes (espaços, comentários, parênteses redundantes).

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

A precedência de operadores já está resolvida na estrutura da árvore —
`Mul` está mais fundo, logo é avaliado primeiro.

**Por que "Abstract":**
`(1 + 2)` e `1 + 2` produzem o mesmo nó `BinOp(Add, 1, 2)`.
Os parênteses existem na sintaxe, não no AST.

**Leitura:** "Crafting Interpreters" — Capítulo 5.

---

#### 2.1.3 Como um Parser funciona

**Por que importa:** Você vai escrever um parser na Fase 1.3.

**Estratégia: Recursive Descent**
Cada regra da gramática vira uma função. A recursão da gramática vira
recursão no código. É a abordagem mais legível e fácil de estender.

```
Gramática:
if_stmt ::= "if" expr block ("else" block)?

Função em Rust:
fn parse_if_stmt(&mut self) -> Result<IfStmt, ParseError> {
    let span = self.current_span();
    self.expect(&Token::If)?;
    let condition   = self.parse_expr()?;
    let then_branch = self.parse_block()?;
    let else_branch = if self.check(&Token::Else) {
        self.advance();
        Some(self.parse_block()?)
    } else {
        None
    };
    Ok(IfStmt { condition, then_branch, else_branch, span })
}
```

**Por que não `pest` nesta fase:**
A gramática vai mudar com frequência. Com recursive descent, mudar a gramática
= mudar a função correspondente. Com pest, mudar a gramática = mudar o `.pest`
+ interpretar a nova CST + adaptar o código. Mais camadas = mais atrito.
Reavaliar pest quando a gramática estabilizar na Fase 3+.

**Leitura:** "Crafting Interpreters" — Capítulos 6–8.

---

#### 2.1.4 Como um Interpreter funciona

**Por que importa:** Você vai escrever um interpreter na Fase 1.4.

O interpreter percorre o AST e executa cada nó. Pattern matching sobre o tipo do nó.

```rust
fn eval_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
    match expr {
        Expr::Literal(lit, _)  => Ok(Value::from(lit)),
        Expr::Identifier(name, span) => {
            self.env.get(name).ok_or_else(|| RuntimeError::UndefinedVariable {
                name: name.clone(), span: *span,
            })
        }
        Expr::BinOp { op, left, right, span } => {
            let l = self.eval_expr(left)?;
            let r = self.eval_expr(right)?;
            self.eval_binop(op, l, r, *span)
        }
        Expr::Call { callee, args, span } => {
            let func   = self.eval_expr(callee)?;
            let evargs = args.iter()
                .map(|a| self.eval_expr(a))
                .collect::<Result<Vec<_>, _>>()?;
            self.call_function(func, evargs, *span)
        }
        // ...
    }
}
```

**Ambiente (Environment):**
Mapeia nomes a valores. Encadeado para suportar escopos léxicos.

```
Ambiente global:   { "print": NativeFn, "len": NativeFn }
    └── Ambiente da função "soma": { "a": Int(3), "b": Int(7) }
            └── Ambiente do if: { "temp": Bool(true) }
```

Ao buscar uma variável, sobe a cadeia até encontrar ou retornar erro com Span.

**Leitura:** "Crafting Interpreters" — Capítulos 7–10.

---

#### 2.1.5 Actor Model e Supervisor Pattern

**Por que importa:** O runtime das fases seguintes usa esses dois modelos.
Entendê-los agora evita redesign do `AgentDecl` na Fase 2.

**Actor Model:**
Actors são unidades de computação isoladas que:
- Têm estado próprio (não compartilhado)
- Comunicam exclusivamente por mensagens
- Processam uma mensagem de cada vez (mailbox sequencial)

No CRL, cada `agent` é um Actor. O event bus é o sistema de mensagens.

**Supervisor Pattern:**
Um supervisor monitora actors e decide como reagir a falhas:
- `restart_handler` — re-executa o handler que falhou
- `restart_agent` — reinicia o agent inteiro preservando contexto
- `stop_agent` — para o agent e registra o motivo
- `escalate` — propaga o erro para o supervisor pai

**Por que importa para a Fase 1:**
O `AgentDecl` no AST deve aceitar `on_error {}` mesmo que a Fase 1 ignore
sua execução. Se o AST não modelar isso agora, o parser da Fase 2 precisará
reescrever estruturas já em uso — o pior tipo de refatoração.

**Leitura:**
- Actor Model: https://en.wikipedia.org/wiki/Actor_model
- Supervisor Pattern: https://www.erlang.org/doc/design_principles/sup_princ.html
  (leia a seção "Supervision Principles" — 10 minutos suficientes)

---

#### 2.1.6 Prioridade de Eventos e Backpressure

**Por que importa:** O Event Bus da Fase 2 usa fila de prioridade.
O `AgentDecl` deve modelar `event_policy {}` desde a Fase 1.

**Problema da fila plana:**
Se um sensor de gás e um timer cosmético chegam simultaneamente numa fila FIFO,
o timer pode ser processado antes da emergência. Isso é um bug de segurança.

**Solução: fila de prioridade com 4 níveis:**
Critical (0) > High (1) > Normal (2) > Low (3).
O scheduler sempre drena Critical antes de processar outros.

**Backpressure:**
O que fazer quando a fila está cheia? As opções têm semânticas diferentes:
- `drop_oldest` — descarta o evento mais antigo (correto para streams de percepção)
- `drop_newest` — descarta o evento recém-chegado (correto para histórico)
- `error_to_agent` — notifica o agent do overflow
- `never_drop` — fila cresce (apenas para Critical)

**Por que importa para a Fase 1:**
`event_policy {}` no `AgentDecl` deve ser parseado, mesmo que ignorado.
Assim, scripts que declaram `event_policy` já funcionam sintaticamente
quando a Fase 2 implementar a semântica.

---

### 2.2 Checklist de Estudo — Fase 0

- [ ] Leia os Capítulos 1–8 de "Crafting Interpreters" (craftinginterpreters.com)
- [ ] Escreva à mão os tokens de:
      `agent home { on_error { strategy = "restart_handler" } on start { let x = 42 } }`
      Resultado esperado: `[Agent, Identifier("home"), LBrace, OnError, LBrace,
      Identifier("strategy"), Eq, StringLit("restart_handler"), Newline, RBrace,
      On, Start, LBrace, Let, Identifier("x"), Eq, Integer(42), RBrace, RBrace, Eof]`
- [ ] Desenhe o AST de: `if x > 10 { print(x) } else { print(0) }`
- [ ] Leia a documentação do crate `logos` e execute o exemplo básico
- [ ] Leia a seção "Supervision Principles" do guia Erlang/OTP (link acima)
- [ ] Leia sobre Actor Model (Wikipedia) — 20 minutos
- [ ] Escreva em texto livre: "Como o CRL executa este código e o que acontece se o handler falhar?"
      ```
      agent main {
          on_error { strategy = "restart_handler" max_retries = 3 }
          fn soma(a, b) { return a + b }
          on start {
              let resultado = soma(3, 7)
              print(resultado)
          }
      }
      ```

---

## 3. DECISÕES TÉCNICAS DESTA FASE

Decisões locais às Fases 0–1, com raciocínio explícito.
Cada uma pode ser revisada nas fases seguintes.

### 3.1 Recursive descent em vez de pest

**Decisão:** Parser recursive descent manual.
**Raciocínio:** Gramática muda frequentemente na Fase 1. Recursive descent:
mudar a gramática = mudar a função. Pest: mudar a gramática = mudar `.pest`
+ CST + código adaptado. Custo-benefício favorece recursive descent agora.
**Revisão:** Avaliar pest na Fase 3 quando a gramática estabilizar.

### 3.2 Tipagem anotada como opcional

**Decisão:** `let x: Int = 42` é válido; `let x = 42` também é.
**Raciocínio:** MVP executando primeiro. Inferência completa é um compilador separado.
**Revisão:** Anotações obrigatórias em assinaturas de função na Fase 4.

### 3.3 `Rc<RefCell<Environment>>` para escopo

**Decisão:** Ambiente de variáveis com `Rc<RefCell<>>` para encadeamento.
**Raciocínio:** Corretude antes de performance. `Rc<RefCell<>>` é idiomático
para interpretadores simples em Rust e evita lifetime hell.
**Revisão:** Arena ou índice por inteiro se o profiler mostrar hotspot na Fase 4+.

### 3.4 AST modela Fase 2+ mesmo sem executar

**Decisão:** `AgentDecl` inclui `ErrorPolicy`, `EventPolicy`, `ChannelDecl`
como campos opcionais. O parser os consome. O interpreter os ignora na Fase 1.
**Raciocínio:** Evitar refatoração de AST quando a Fase 2 for implementada.
Mudar o AST depois que o interpreter já usa ele é a refatoração mais custosa possível.
**Custo:** Mínimo — campos `Option<T>` não têm overhead quando `None`.

### 3.5 Separação Definition/Instance desde a Fase 1

**Decisão:** `AgentDefinition` e `AgentInstance` são structs distintas desde a Fase 1.
O interpreter cria uma `AgentDefinition` a partir do `AgentDecl` do AST,
e uma `AgentInstance` para executar.
**Raciocínio:** Na Fase 9, múltiplas instâncias do mesmo blueprint serão necessárias.
Fazer a separação depois exigiria refatorar o interpreter inteiro.
**Custo:** Uma struct extra e um nível de indireção — negligenciável.

### 3.6 `sled` para persistência futura (Fase 5)

**Decisão:** Quando memória persistente entrar, usar `sled`.
**Raciocínio:** RocksDB requer bindings C. `sled` é Rust puro, compila sem atrito.
Documentado agora para que o crate `crl-context` já use a abstração certa.

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
logos     = "0.14"
thiserror = "1.0"
miette    = { version = "5.10", features = ["fancy"] }
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
[dependencies]  # sem dependências externas — o AST é puro Rust
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
cargo build    # Esperado: Finished
cargo test     # Esperado: running 0 tests (ainda sem testes)
```

### 4.5 Criar estrutura de exemplos

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

cat > examples/phase1/with_error_policy.crl << 'EOF'
-- este script testa que on_error é parseado sem erro na Fase 1
agent main {
    on_error {
        strategy    = "restart_handler"
        max_retries = 3
        backoff_ms  = 500
    }

    on start {
        let msg = "Agent com error_policy iniciado"
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

/// Posição no source code. Presente em todos os tokens e nós do AST.
/// Copy é conveniente — dois usize não justificam referência.
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
/// ATENÇÃO À ORDEM:
/// - Keywords devem vir ANTES de Identifier (logos usa a primeira que casar)
/// - Operadores de 2 chars (==, !=) devem vir ANTES dos de 1 char (=, !)
/// - Float deve vir ANTES de Integer (para não tokenizar 3.14 como Int(3), Dot, Int(14))
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r]+")]          // ignora espaços e tabs (não newlines)
#[logos(skip r"--[^\n]*")]          // comentário single-line: -- até fim da linha
#[logos(skip r"/:([^:]|:[^/])*:/")] // comentário multi-line: /: ... :/
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
    #[token("channel")]     Channel,         // Fase 4 — parseado na Fase 1
    #[token("priority")]    Priority,        // Fase 2 — parseado na Fase 1
    #[token("concurrent")]  Concurrent,      // Fase 4 — parseado na Fase 1

    // ---- EVENTOS BUILT-IN ----
    #[token("start")]           Start,
    #[token("stop")]            Stop,
    #[token("on_error")]        OnError,     // Supervisor — parseado na Fase 1
    #[token("event_policy")]    EventPolicy, // Backpressure — parseado na Fase 1
    #[token("ai_policy")]       AiPolicy,    // IA — parseado na Fase 1

    // ---- NÍVEIS DE PRIORIDADE ----
    #[token("critical")]    PriorityCritical,
    #[token("high")]        PriorityHigh,
    #[token("normal")]      PriorityNormal,
    #[token("low")]         PriorityLow,

    // ---- OPERADORES ----
    // CRÍTICO: operadores de 2 chars ANTES dos de 1 char
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

    // Scientific ANTES de Float ANTES de Integer
    /// Número com expoente: 1e10, 2.5e-3
    #[regex(r"[0-9]+(?:\.[0-9]+)?[eE][+-]?[0-9]+",
        |lex| lex.slice().parse::<f64>().ok())]
    Scientific(f64),

    /// Float: 3.14, 0.5
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    Float(f64),

    /// Inteiro: 42, 0, 1000
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Integer(i64),

    /// String entre aspas duplas.
    /// Fase 1: sem escape sequences (ex: \n, \").
    /// Fase 3+: suporte a escape sequences.
    #[regex(r#""[^"]*""#, |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string())
    })]
    StringLit(String),

    /// Identifier: variáveis, funções, agents, nomes de evento.
    /// DEVE vir depois de todas as keywords para não sobrescrever.
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    /// Fim do arquivo — sempre o último token.
    Eof,
}
```

### 5.2 Implementar o Lexer

Crie `crates/crl-lexer/src/error.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum LexError {
    #[error("Caractere inesperado '{character}' na linha {line}, coluna {column}")]
    UnexpectedCharacter { character: char, line: usize, column: usize },

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
/// Coleta TODOS os erros antes de retornar — não para no primeiro.
/// Isso permite reportar múltiplos erros de uma vez (melhor UX).
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
                // logos retorna Err para qualquer char não reconhecido
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
    lex(source).expect("lex falhou")
        .into_iter()
        .map(|st| st.token)
        .filter(|t| !matches!(t, Token::Newline | Token::Eof))
        .collect()
}

#[test] fn keywords_reconhecidos() {
    let r = tokens("agent on fn let return if else while for in use do");
    assert_eq!(r[0], Token::Agent);
    assert_eq!(r[1], Token::On);
    assert_eq!(r[5], Token::If);
    assert_eq!(r[11], Token::Do);
}

#[test] fn novos_keywords_fase2_parseados() {
    // on_error, event_policy e channel devem ser reconhecidos sem erro
    let r = tokens("on_error event_policy channel priority concurrent");
    assert_eq!(r[0], Token::OnError);
    assert_eq!(r[1], Token::EventPolicy);
    assert_eq!(r[2], Token::Channel);
    assert_eq!(r[3], Token::Priority);
    assert_eq!(r[4], Token::Concurrent);
}

#[test] fn priority_levels_reconhecidos() {
    let r = tokens("critical high normal low");
    assert_eq!(r[0], Token::PriorityCritical);
    assert_eq!(r[1], Token::PriorityHigh);
    assert_eq!(r[2], Token::PriorityNormal);
    assert_eq!(r[3], Token::PriorityLow);
}

#[test] fn identifier_nao_confundido_com_keyword() {
    let r = tokens("agent_name on_something channel_data");
    assert_eq!(r[0], Token::Identifier("agent_name".into()));
    assert_eq!(r[1], Token::Identifier("on_something".into()));
    assert_eq!(r[2], Token::Identifier("channel_data".into()));
}

#[test] fn literais_numericos_com_prioridade_correta() {
    // 3.14 não pode ser tokenizado como Integer(3) Dot Integer(14)
    let r = tokens("42 3.14 1e10 0");
    assert_eq!(r[0], Token::Integer(42));
    assert_eq!(r[1], Token::Float(3.14));
    assert_eq!(r[2], Token::Scientific(1e10));
    assert_eq!(r[3], Token::Integer(0));
}

#[test] fn operadores_dois_chars_antes_de_um() {
    // == não pode ser tokenizado como Eq Eq
    let r = tokens("== != <= >= -> < > =");
    assert_eq!(r[0], Token::EqEq);
    assert_eq!(r[1], Token::NotEq);
    assert_eq!(r[2], Token::LtEq);
    assert_eq!(r[3], Token::GtEq);
    assert_eq!(r[4], Token::Arrow);
    assert_eq!(r[5], Token::Lt);
    assert_eq!(r[6], Token::Gt);
    assert_eq!(r[7], Token::Eq);
}

#[test] fn comentario_single_line_ignorado() {
    let r = tokens("let x = 42 -- isso é ignorado");
    assert_eq!(r.len(), 4); // Let, Identifier, Eq, Integer
}

#[test] fn comentario_multi_line_ignorado() {
    let r = tokens("let /: este comentário\nspana múltiplas linhas :/ x = 42");
    assert_eq!(r[0], Token::Let);
    assert_eq!(r[1], Token::Identifier("x".into()));
}

#[test] fn linha_e_coluna_corretos() {
    let result = lex("let x = 1\nlet y = 2").unwrap();
    let lets: Vec<_> = result.iter().filter(|t| t.token == Token::Let).collect();
    assert_eq!(lets[0].line, 1); assert_eq!(lets[0].column, 1);
    assert_eq!(lets[1].line, 2); assert_eq!(lets[1].column, 1);
}

#[test] fn erro_com_posicao_correta() {
    let result = lex("let x = @invalido");
    assert!(result.is_err());
    let erros = result.unwrap_err();
    assert_eq!(erros.len(), 1);
    let msg = format!("{}", erros[0]);
    assert!(msg.contains('@'));
    assert!(msg.contains("coluna 9")); // 'let x = ' tem 8 chars, '@' está na col 9
}

#[test] fn multiplos_erros_coletados() {
    // dois caracteres inválidos — ambos devem aparecer
    let result = lex("let @ = #");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().len(), 2);
}
```

### 5.4 Checklist — Lexer

- [X] Todos os keywords reconhecidos: `agent`, `on`, `fn`, `let`, `return`, `if`, `else`, `while`, `for`, `in`, `use`, `do`, `send`
- [X] Novos keywords reconhecidos: `on_error`, `event_policy`, `ai_policy`, `channel`, `priority`, `concurrent`
- [X] Níveis de prioridade reconhecidos: `critical`, `high`, `normal`, `low`
- [X] Identifiers não confundidos com keywords (ex: `on_something` é `Identifier`)
- [X] Float processado antes de Integer (3.14 não vira Int(3) Dot Int(14))
- [X] Operadores de 2 chars processados antes dos de 1 char
- [X] Comentário single-line ignorado (`--`)
- [X] Comentário multi-line ignorado (`/: ... :/`)
- [X] Linha e coluna corretos para cada token
- [X] Erro com posição correta para caractere inválido
- [X] TODOS os erros coletados antes de retornar (não para no primeiro)
- [X] `cargo nextest run -p giulia-lexer` 100% passando

---

## 6. FASE 1.2 — AST

**Objetivo:** Definir a estrutura de dados que representa o programa em memória.

**REGRA DESTA FASE:** O AST modela construções de fases futuras como campos
opcionais. O parser os consome. O interpreter os ignora. Isso evita refatoração
do AST quando as fases seguintes os implementarem.

Crie `crates/crl-ast/src/lib.rs`:
```rust
pub mod node;
pub mod types;
pub mod policies;  // NOVO v3.0
```

Crie `crates/crl-ast/src/types.rs`:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    Int, Float, String, Bool, Null,
    List(Box<TypeExpr>),
    Map(Box<TypeExpr>, Box<TypeExpr>),
    Named(String),
    Optional(Box<TypeExpr>),   // Fase 3: tipo?
}
```

Crie `crates/crl-ast/src/policies.rs`:
```rust
/// Política de recuperação de falhas (Supervisor Pattern) — Fase 2.
/// Parseada na Fase 1, semântica implementada na Fase 2.
#[derive(Debug, Clone, Default)]
pub struct ErrorPolicy {
    pub strategy:    Option<String>,   // "restart_handler" | "restart_agent" | "stop_agent" | "escalate"
    pub max_retries: Option<u32>,
    pub backoff_ms:  Option<u64>,
    pub on_exhaust:  Option<String>,
}

/// Política de backpressure e fila de eventos — Fase 2.
/// Parseada na Fase 1, semântica implementada na Fase 2.
#[derive(Debug, Clone, Default)]
pub struct EventPolicy {
    pub on_overflow:    Option<String>,  // "drop_oldest" | "drop_newest" | "error" | "never_drop"
    pub critical_queue: Option<usize>,
    pub normal_queue:   Option<usize>,
    pub low_queue:      Option<usize>,
}

/// Política de chamadas de IA — Fase 8.
/// Parseada na Fase 1, semântica implementada na Fase 8.
#[derive(Debug, Clone, Default)]
pub struct AiPolicy {
    pub timeout_ms:        Option<u64>,
    pub max_queue_during:  Option<usize>,
    pub on_timeout:        Option<String>,  // "return_error" | "return_fallback" | "cancel_and_continue"
    pub fallback_response: Option<String>,
    pub cache_identical:   Option<bool>,
}

/// Nível de prioridade de um evento — Fase 2.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum EventPriorityLevel {
    Critical,
    High,
    #[default]
    Normal,
    Low,
}

/// Declaração de canal tipado — Fase 4.
#[derive(Debug, Clone)]
pub struct ChannelDecl {
    pub name:      String,
    pub chan_type: crate::types::TypeExpr,
    pub span:      Span,
}
```

Crie `crates/crl-ast/src/node.rs`:

```rust
use crate::types::TypeExpr;
use crate::policies::{ErrorPolicy, EventPolicy, AiPolicy, EventPriorityLevel, ChannelDecl};

/// Posição no source code — presente em TODOS os nós.
/// Sem Span não há mensagens de erro úteis.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Span {
    pub start: usize,
    pub end:   usize,
}

// ─── SEPARAÇÃO DEFINITION / INSTANCE (v3.0) ──────────────────────────────────
//
// AgentDecl  = o AST bruto do parser (parsing)
// AgentDefinition = blueprint compilado (análise + interpreter setup)
// AgentInstance   = agent vivo em execução (runtime — Fase 2)
//
// Na Fase 1, o interpreter cria AgentDefinition a partir de AgentDecl.
// Na Fase 2, o runtime cria AgentInstance a partir de AgentDefinition.
// Nunca misturar as três — cada uma tem um ciclo de vida diferente.

// ─── PROGRAMA ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Program {
    pub stmts: Vec<Stmt>,
    pub span:  Span,
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
    EffectStmt(EffectStmt),   // "do expr" — side effect explícito (Fase 4)
    SendStmt(SendStmt),       // "send(...)" — canal tipado (Fase 4)
    ExprStmt(ExprStmt),
}

/// Declaração de agent — o blueprint bruto do parser.
/// Inclui campos de fases futuras como Option<T> para evitar refatoração.
#[derive(Debug, Clone)]
pub struct AgentDecl {
    pub name:         String,
    pub version:      Option<u64>,          // hot reload — Fase 7

    // ── campos da Fase 1 ──
    pub capabilities: Vec<UseDecl>,
    pub handlers:     Vec<HandlerDecl>,
    pub functions:    Vec<FnDecl>,

    // ── campos parseados, ignorados na Fase 1 ──
    pub error_policy: Option<ErrorPolicy>,  // Supervisor — Fase 2
    pub event_policy: Option<EventPolicy>,  // Backpressure — Fase 2
    pub ai_policy:    Option<AiPolicy>,     // IA — Fase 8
    pub channels:     Vec<ChannelDecl>,     // Canais tipados — Fase 4

    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct UseDecl {
    pub capability: String,
    pub span:       Span,
}

/// Handler de evento com prioridade e flag de concorrência.
#[derive(Debug, Clone)]
pub struct HandlerDecl {
    pub event:       EventPattern,
    pub priority:    EventPriorityLevel,    // padrão: Normal
    pub concurrent:  bool,                  // padrão: false (Fase 4)
    pub body:        Block,
    pub span:        Span,
}

/// Padrão de evento — extensível por fase.
#[derive(Debug, Clone)]
pub enum EventPattern {
    Start,
    Stop,
    Timer(Expr),
    Message(MessageTarget),     // tipado: Fase 4
    Speech,
    Image,
    SensorChange(String),
    Network(String),
    Idle(Expr),
    Custom(String),
}

/// Alvo de uma mensagem — simples (nome) ou tipado (agent.canal).
#[derive(Debug, Clone)]
pub enum MessageTarget {
    Simple(String),              // on message("topico")
    Typed { agent: String, channel: String }, // on message(sensor.temperature)
}

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name:        String,
    pub params:      Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub body:        Block,
    pub span:        Span,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name:            String,
    pub type_annotation: Option<TypeExpr>,
    pub span:            Span,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span:  Span,
}

// ─── STATEMENTS CONCRETOS ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct LetStmt {
    pub name:     String,
    pub type_ann: Option<TypeExpr>,
    pub value:    Expr,
    pub span:     Span,
}

#[derive(Debug, Clone)]
pub struct AssignStmt { pub name: String, pub value: Expr, pub span: Span }

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition:   Expr,
    pub then_branch: Block,
    pub else_branch: Option<Box<ElseBranch>>,
    pub span:        Span,
}

#[derive(Debug, Clone)]
pub enum ElseBranch { Block(Block), If(IfStmt) }

#[derive(Debug, Clone)]
pub struct WhileStmt { pub condition: Expr, pub body: Block, pub span: Span }

#[derive(Debug, Clone)]
pub struct ForStmt { pub var: String, pub iterable: Expr, pub body: Block, pub span: Span }

#[derive(Debug, Clone)]
pub struct ReturnStmt { pub value: Option<Expr>, pub span: Span }

#[derive(Debug, Clone)]
pub struct EffectStmt { pub expr: Expr, pub span: Span }

/// `send("agent").canal(value)` — canal tipado (Fase 4, sintaxe parseada na Fase 1)
#[derive(Debug, Clone)]
pub struct SendStmt {
    pub target_agent:   String,
    pub channel:        Option<String>,  // None para send legado com topic string
    pub topic:          Option<String>,  // para send legado: send("agent", "topic", value)
    pub payload:        Expr,
    pub span:           Span,
}

#[derive(Debug, Clone)]
pub struct ExprStmt { pub expr: Expr, pub span: Span }

// ─── EXPRESSÕES ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal, Span),
    Identifier(String, Span),
    BinOp    { op: BinOp, left: Box<Expr>, right: Box<Expr>, span: Span },
    UnaryOp  { op: UnaryOp, operand: Box<Expr>, span: Span },
    Call     { callee: Box<Expr>, args: Vec<Expr>, span: Span },
    FieldAccess { object: Box<Expr>, field: String, span: Span },
    Index    { object: Box<Expr>, index: Box<Expr>, span: Span },
    List(Vec<Expr>, Span),
    Map(Vec<(Expr, Expr)>, Span),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(_, s)        => *s,
            Expr::Identifier(_, s)     => *s,
            Expr::BinOp { span, .. }   => *span,
            Expr::UnaryOp { span, .. } => *span,
            Expr::Call { span, .. }    => *span,
            Expr::FieldAccess { span, .. } => *span,
            Expr::Index { span, .. }   => *span,
            Expr::List(_, s)           => *s,
            Expr::Map(_, s)            => *s,
        }
    }
}

// ─── LITERAIS ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64), Float(f64), Scientific(f64),
    String(String), Bool(bool), Null,
    List(Vec<Expr>), Map(Vec<(Expr, Expr)>),
}

// ─── OPERADORES ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Rem,
    Eq, NotEq, Lt, Gt, LtEq, GtEq,
    And, Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp { Neg, Not }
```

### 6.1 Checklist — AST

- [X] Todos os nós compilando sem erros
- [X] Todos os nós têm campo `span: Span`
- [X] `Expr::span()` implementado e testado
- [X] `AgentDecl` inclui `error_policy`, `event_policy`, `ai_policy`, `channels` como `Option`/`Vec`
- [X] `HandlerDecl` inclui `priority` e `concurrent`
- [X] `SendStmt` modelado para canais tipados
- [X] O seguinte programa pode ser representado no AST sem perda:
      ```
      agent main {
          on_error { strategy = "restart_handler" max_retries = 3 }
          on start priority high {
              let x = 1 + 2
              if x > 2 { print(x) }
          }
      }
      ```

---

## 7. FASE 1.3 — PARSER

**Objetivo:** Converter tokens em AST.

O parser deve aceitar `on_error {}`, `event_policy {}`, `ai_policy {}`,
`channel nome: Tipo`, e `priority level` em handlers — mesmo que o interpreter
não execute essas construções na Fase 1. **A gramática não deve mudar na Fase 2.**

### 7.1 Estrutura base

Crie `crates/crl-parser/src/error.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum ParseError {
    #[error("Token inesperado: esperava '{expected}', encontrou '{found}' na linha {line}, coluna {column}")]
    UnexpectedToken { expected: String, found: String, line: usize, column: usize },

    #[error("Esperava um identificador em '{context}' na linha {line}, coluna {column}")]
    ExpectedIdentifier { context: String, line: usize, column: usize },

    #[error("Alvo de atribuição inválido na linha {line}, coluna {column}")]
    InvalidAssignTarget { line: usize, column: usize },

    #[error("Tipo inválido em declaração de canal na linha {line}, coluna {column}")]
    InvalidChannelType { line: usize, column: usize },
}
```

Crie `crates/crl-parser/src/lib.rs`:

```rust
mod error;
mod parser;

pub use error::ParseError;
pub use parser::Parser;

use crl_lexer::token::SpannedToken;
use crl_ast::node::Program;

pub fn parse(tokens: Vec<SpannedToken>) -> Result<Program, Vec<ParseError>> {
    Parser::new(tokens).parse_program()
}
```

### 7.2 Implementação do Parser

O parser implementa recursive descent. Abaixo as funções mais importantes.
As demais seguem o mesmo padrão.

Crie `crates/crl-parser/src/parser.rs`:

```rust
use crl_lexer::token::{Token, SpannedToken};
use crl_ast::node::*;
use crl_ast::types::TypeExpr;
use crl_ast::policies::{ErrorPolicy, EventPolicy, AiPolicy, EventPriorityLevel, ChannelDecl};
use crate::error::ParseError;

pub struct Parser {
    tokens: Vec<SpannedToken>,
    cursor: usize,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self { tokens, cursor: 0 }
    }

    // ── HELPERS ──────────────────────────────────────────────────────────────

    fn peek(&self) -> &Token {
        self.tokens.get(self.cursor).map(|t| &t.token).unwrap_or(&Token::Eof)
    }

    fn peek_next(&self) -> &Token {
        self.tokens.get(self.cursor + 1).map(|t| &t.token).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> &SpannedToken {
        let t = &self.tokens[self.cursor];
        if self.cursor < self.tokens.len() - 1 { self.cursor += 1; }
        t
    }

    fn check(&self, tok: &Token) -> bool { self.peek() == tok }

    fn expect(&mut self, expected: &Token) -> Result<&SpannedToken, ParseError> {
        if self.peek() == expected {
            Ok(self.advance())
        } else {
            let cur = self.tokens.get(self.cursor);
            Err(ParseError::UnexpectedToken {
                expected: format!("{:?}", expected),
                found:    format!("{:?}", self.peek()),
                line:     cur.map(|t| t.line).unwrap_or(0),
                column:   cur.map(|t| t.column).unwrap_or(0),
            })
        }
    }

    fn skip_newlines(&mut self) {
        while self.check(&Token::Newline) { self.advance(); }
    }

    fn current_span(&self) -> Span {
        self.tokens.get(self.cursor)
            .map(|t| Span { start: t.span.start, end: t.span.end })
            .unwrap_or_default()
    }

    fn loc(&self) -> (usize, usize) {
        self.tokens.get(self.cursor)
            .map(|t| (t.line, t.column))
            .unwrap_or((0, 0))
    }

    // ── SINCRONIZAÇÃO DE ERROS ───────────────────────────────────────────────

    /// Avança até um ponto seguro de retomada após um erro.
    fn synchronize(&mut self) {
        while !matches!(self.peek(),
            Token::Agent | Token::Fn | Token::On | Token::Let |
            Token::OnError | Token::EventPolicy | Token::Channel |
            Token::RBrace | Token::Eof)
        {
            self.advance();
        }
    }

    // ── PROGRAMA ─────────────────────────────────────────────────────────────

    pub fn parse_program(&mut self) -> Result<Program, Vec<ParseError>> {
        let mut stmts  = Vec::new();
        let mut errors = Vec::new();
        let start      = self.current_span();

        self.skip_newlines();

        while !self.check(&Token::Eof) {
            match self.parse_top_level() {
                Ok(s)  => stmts.push(s),
                Err(e) => { errors.push(e); self.synchronize(); }
            }
            self.skip_newlines();
        }

        if errors.is_empty() { Ok(Program { stmts, span: start }) }
        else                 { Err(errors) }
    }

    fn parse_top_level(&mut self) -> Result<Stmt, ParseError> {
        match self.peek() {
            Token::Agent => Ok(Stmt::AgentDecl(self.parse_agent_decl()?)),
            Token::Fn    => Ok(Stmt::FnDecl(self.parse_fn_decl()?)),
            Token::Let   => Ok(Stmt::LetStmt(self.parse_let_stmt()?)),
            _ => {
                let (l, c) = self.loc();
                Err(ParseError::UnexpectedToken {
                    expected: "agent, fn, ou let".into(),
                    found: format!("{:?}", self.peek()),
                    line: l, column: c,
                })
            }
        }
    }

    // ── AGENT ─────────────────────────────────────────────────────────────────

    fn parse_agent_decl(&mut self) -> Result<AgentDecl, ParseError> {
        let span = self.current_span();
        self.expect(&Token::Agent)?;

        let name = self.parse_identifier("nome do agent")?;

        // versão opcional: agent home v2 { ... }
        let version = if let Token::Identifier(s) = self.peek().clone() {
            if s == "v" { self.advance(); Some(self.parse_u64_literal()?) }
            else { None }
        } else { None };

        self.expect(&Token::LBrace)?;
        self.skip_newlines();

        let mut capabilities = Vec::new();
        let mut handlers     = Vec::new();
        let mut functions    = Vec::new();
        let mut channels     = Vec::new();
        let mut error_policy = None;
        let mut event_policy = None;
        let mut ai_policy    = None;

        while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
            self.skip_newlines();
            match self.peek().clone() {
                Token::Use         => capabilities.push(self.parse_use_decl()?),
                Token::On          => handlers.push(self.parse_handler_decl()?),
                Token::Fn          => functions.push(self.parse_fn_decl()?),
                Token::Channel     => channels.push(self.parse_channel_decl()?),
                Token::OnError     => error_policy = Some(self.parse_error_policy()?),
                Token::EventPolicy => event_policy = Some(self.parse_event_policy()?),
                Token::AiPolicy    => ai_policy    = Some(self.parse_ai_policy()?),
                Token::RBrace      => break,
                _ => {
                    let (l, c) = self.loc();
                    return Err(ParseError::UnexpectedToken {
                        expected: "use, on, fn, channel, on_error, event_policy, ou }".into(),
                        found: format!("{:?}", self.peek()),
                        line: l, column: c,
                    });
                }
            }
            self.skip_newlines();
        }

        self.expect(&Token::RBrace)?;

        Ok(AgentDecl {
            name, version, capabilities, handlers, functions,
            channels, error_policy, event_policy, ai_policy, span,
        })
    }

    fn parse_use_decl(&mut self) -> Result<UseDecl, ParseError> {
        let span = self.current_span();
        self.expect(&Token::Use)?;
        let capability = self.parse_identifier("nome da capability")?;
        self.skip_newlines();
        Ok(UseDecl { capability, span })
    }

    fn parse_channel_decl(&mut self) -> Result<ChannelDecl, ParseError> {
        let span = self.current_span();
        self.expect(&Token::Channel)?;
        let name = self.parse_identifier("nome do canal")?;
        self.expect(&Token::Colon)?;
        let chan_type = self.parse_type_expr()?;
        self.skip_newlines();
        Ok(ChannelDecl { name, chan_type, span })
    }

    fn parse_error_policy(&mut self) -> Result<ErrorPolicy, ParseError> {
        self.expect(&Token::OnError)?;
        self.expect(&Token::LBrace)?;
        self.skip_newlines();
        let mut policy = ErrorPolicy::default();
        while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
            let key = self.parse_identifier("campo de on_error")?;
            self.expect(&Token::Eq)?;
            match key.as_str() {
                "strategy"    => policy.strategy    = Some(self.parse_string_value()?),
                "max_retries" => policy.max_retries = Some(self.parse_u32_literal()?),
                "backoff_ms"  => policy.backoff_ms  = Some(self.parse_u64_literal()?),
                "on_exhaust"  => policy.on_exhaust  = Some(self.parse_string_value()?),
                _ => { /* campo desconhecido: ignora graciosamente */ self.parse_expr()?; }
            }
            self.skip_newlines();
        }
        self.expect(&Token::RBrace)?;
        self.skip_newlines();
        Ok(policy)
    }

    fn parse_event_policy(&mut self) -> Result<EventPolicy, ParseError> {
        self.expect(&Token::EventPolicy)?;
        self.expect(&Token::LBrace)?;
        self.skip_newlines();
        let mut policy = EventPolicy::default();
        while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
            let key = self.parse_identifier("campo de event_policy")?;
            self.expect(&Token::Eq)?;
            match key.as_str() {
                "on_overflow"    => policy.on_overflow    = Some(self.parse_string_value()?),
                "critical_queue" => policy.critical_queue = Some(self.parse_usize_literal()?),
                "normal_queue"   => policy.normal_queue   = Some(self.parse_usize_literal()?),
                "low_queue"      => policy.low_queue      = Some(self.parse_usize_literal()?),
                _ => { self.parse_expr()?; }
            }
            self.skip_newlines();
        }
        self.expect(&Token::RBrace)?;
        self.skip_newlines();
        Ok(policy)
    }

    fn parse_ai_policy(&mut self) -> Result<AiPolicy, ParseError> {
        self.expect(&Token::AiPolicy)?;
        self.expect(&Token::LBrace)?;
        self.skip_newlines();
        let mut policy = AiPolicy::default();
        while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
            let key = self.parse_identifier("campo de ai_policy")?;
            self.expect(&Token::Eq)?;
            match key.as_str() {
                "timeout_ms"        => policy.timeout_ms       = Some(self.parse_u64_literal()?),
                "max_queue_during"  => policy.max_queue_during = Some(self.parse_usize_literal()?),
                "on_timeout"        => policy.on_timeout       = Some(self.parse_string_value()?),
                "fallback_response" => policy.fallback_response= Some(self.parse_string_value()?),
                "cache_identical"   => policy.cache_identical  = Some(self.parse_bool_value()?),
                _ => { self.parse_expr()?; }
            }
            self.skip_newlines();
        }
        self.expect(&Token::RBrace)?;
        self.skip_newlines();
        Ok(policy)
    }

    fn parse_handler_decl(&mut self) -> Result<HandlerDecl, ParseError> {
        let span = self.current_span();
        self.expect(&Token::On)?;
        let event = self.parse_event_pattern()?;

        // `priority level` — opcional
        let priority = if self.check(&Token::Priority) {
            self.advance();
            match self.peek() {
                Token::PriorityCritical => { self.advance(); EventPriorityLevel::Critical }
                Token::PriorityHigh     => { self.advance(); EventPriorityLevel::High }
                Token::PriorityNormal   => { self.advance(); EventPriorityLevel::Normal }
                Token::PriorityLow      => { self.advance(); EventPriorityLevel::Low }
                _ => EventPriorityLevel::Normal,
            }
        } else { EventPriorityLevel::Normal };

        // `concurrent` — opcional
        let concurrent = if self.check(&Token::Concurrent) {
            self.advance(); true
        } else { false };

        let body = self.parse_block()?;
        Ok(HandlerDecl { event, priority, concurrent, body, span })
    }

    fn parse_event_pattern(&mut self) -> Result<EventPattern, ParseError> {
        let (l, c) = self.loc();
        match self.peek().clone() {
            Token::Start => { self.advance(); Ok(EventPattern::Start) }
            Token::Stop  => { self.advance(); Ok(EventPattern::Stop)  }
            Token::Identifier(name) => {
                self.advance();
                match name.as_str() {
                    "timer" => {
                        self.expect(&Token::LParen)?;
                        let dur = self.parse_expr()?;
                        self.expect(&Token::RParen)?;
                        Ok(EventPattern::Timer(dur))
                    }
                    "speech" => Ok(EventPattern::Speech),
                    "image"  => Ok(EventPattern::Image),
                    "message" => {
                        self.expect(&Token::LParen)?;
                        // detecta se é tipado (agent.canal) ou simples
                        let target = if let Token::Identifier(agent) = self.peek().clone() {
                            self.advance();
                            if self.check(&Token::Dot) {
                                self.advance();
                                let channel = self.parse_identifier("nome do canal")?;
                                MessageTarget::Typed { agent, channel }
                            } else {
                                MessageTarget::Simple(agent)
                            }
                        } else {
                            MessageTarget::Simple(self.parse_string_value()?)
                        };
                        self.expect(&Token::RParen)?;
                        Ok(EventPattern::Message(target))
                    }
                    "sensor_change" => {
                        self.expect(&Token::LParen)?;
                        let s = self.parse_string_value()?;
                        self.expect(&Token::RParen)?;
                        Ok(EventPattern::SensorChange(s))
                    }
                    "network" => {
                        self.expect(&Token::LParen)?;
                        let s = self.parse_string_value()?;
                        self.expect(&Token::RParen)?;
                        Ok(EventPattern::Network(s))
                    }
                    "idle" => {
                        self.expect(&Token::LParen)?;
                        let dur = self.parse_expr()?;
                        self.expect(&Token::RParen)?;
                        Ok(EventPattern::Idle(dur))
                    }
                    other => Ok(EventPattern::Custom(other.to_string())),
                }
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "padrão de evento".into(),
                found: format!("{:?}", self.peek()),
                line: l, column: c,
            })
        }
    }

    // ── BLOCO E STATEMENTS ───────────────────────────────────────────────────

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let span = self.current_span();
        self.expect(&Token::LBrace)?;
        self.skip_newlines();
        let mut stmts = Vec::new();
        while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
            stmts.push(self.parse_stmt()?);
            self.skip_newlines();
        }
        self.expect(&Token::RBrace)?;
        Ok(Block { stmts, span })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek() {
            Token::Let    => Ok(Stmt::LetStmt(self.parse_let_stmt()?)),
            Token::Return => Ok(Stmt::ReturnStmt(self.parse_return_stmt()?)),
            Token::If     => Ok(Stmt::IfStmt(self.parse_if_stmt()?)),
            Token::While  => Ok(Stmt::WhileStmt(self.parse_while_stmt()?)),
            Token::For    => Ok(Stmt::ForStmt(self.parse_for_stmt()?)),
            Token::Do     => Ok(Stmt::EffectStmt(self.parse_effect_stmt()?)),
            Token::Send   => Ok(Stmt::SendStmt(self.parse_send_stmt()?)),
            _             => self.parse_assign_or_expr_stmt(),
        }
    }

    fn parse_let_stmt(&mut self) -> Result<LetStmt, ParseError> {
        let span = self.current_span();
        self.expect(&Token::Let)?;
        let name    = self.parse_identifier("let statement")?;
        let type_ann = if self.check(&Token::Colon) {
            self.advance();
            Some(self.parse_type_expr()?)
        } else { None };
        self.expect(&Token::Eq)?;
        let value = self.parse_expr()?;
        self.skip_newlines();
        Ok(LetStmt { name, type_ann, value, span })
    }

    fn parse_effect_stmt(&mut self) -> Result<EffectStmt, ParseError> {
        let span = self.current_span();
        self.expect(&Token::Do)?;
        let expr = self.parse_expr()?;
        self.skip_newlines();
        Ok(EffectStmt { expr, span })
    }

    fn parse_send_stmt(&mut self) -> Result<SendStmt, ParseError> {
        let span = self.current_span();
        self.expect(&Token::Send)?;
        self.expect(&Token::LParen)?;
        let target_agent = self.parse_string_value()?;
        self.expect(&Token::RParen)?;

        // send("agent").canal(value) — tipado
        if self.check(&Token::Dot) {
            self.advance();
            let channel = self.parse_identifier("nome do canal")?;
            self.expect(&Token::LParen)?;
            let payload = self.parse_expr()?;
            self.expect(&Token::RParen)?;
            self.skip_newlines();
            return Ok(SendStmt { target_agent, channel: Some(channel), topic: None, payload, span });
        }

        // send("agent", "topic", value) — legado
        self.expect(&Token::Comma)?;
        let topic = self.parse_string_value()?;
        self.expect(&Token::Comma)?;
        let payload = self.parse_expr()?;
        self.skip_newlines();
        Ok(SendStmt { target_agent, channel: None, topic: Some(topic), payload, span })
    }

    fn parse_return_stmt(&mut self) -> Result<ReturnStmt, ParseError> {
        let span = self.current_span();
        self.expect(&Token::Return)?;
        let value = if !self.check(&Token::Newline) && !self.check(&Token::RBrace) {
            Some(self.parse_expr()?)
        } else { None };
        self.skip_newlines();
        Ok(ReturnStmt { value, span })
    }

    fn parse_if_stmt(&mut self) -> Result<IfStmt, ParseError> {
        let span = self.current_span();
        self.expect(&Token::If)?;
        let condition   = self.parse_expr()?;
        let then_branch = self.parse_block()?;
        let else_branch = if self.check(&Token::Else) {
            self.advance();
            self.skip_newlines();
            if self.check(&Token::If) {
                Some(Box::new(ElseBranch::If(self.parse_if_stmt()?)))
            } else {
                Some(Box::new(ElseBranch::Block(self.parse_block()?)))
            }
        } else { None };
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
        let var      = self.parse_identifier("variável do for")?;
        self.expect(&Token::In)?;
        let iterable = self.parse_expr()?;
        let body     = self.parse_block()?;
        Ok(ForStmt { var, iterable, body, span })
    }

    fn parse_assign_or_expr_stmt(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.parse_expr()?;
        if self.check(&Token::Eq) {
            self.advance();
            let value = self.parse_expr()?;
            self.skip_newlines();
            if let Expr::Identifier(name, span) = expr {
                return Ok(Stmt::AssignStmt(AssignStmt { name, value, span }));
            }
            let (l, c) = self.loc();
            return Err(ParseError::InvalidAssignTarget { line: l, column: c });
        }
        self.skip_newlines();
        Ok(Stmt::ExprStmt(ExprStmt { span: expr.span(), expr }))
    }

    // ── EXPRESSÕES (Pratt / precedência) ─────────────────────────────────────

    fn parse_expr(&mut self) -> Result<Expr, ParseError> { self.parse_or() }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut l = self.parse_and()?;
        while self.check(&Token::Or) {
            let span = self.current_span(); self.advance();
            let r = self.parse_and()?;
            l = Expr::BinOp { op: BinOp::Or, left: Box::new(l), right: Box::new(r), span };
        }
        Ok(l)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut l = self.parse_eq()?;
        while self.check(&Token::And) {
            let span = self.current_span(); self.advance();
            let r = self.parse_eq()?;
            l = Expr::BinOp { op: BinOp::And, left: Box::new(l), right: Box::new(r), span };
        }
        Ok(l)
    }

    fn parse_eq(&mut self) -> Result<Expr, ParseError> {
        let mut l = self.parse_cmp()?;
        loop {
            let op = match self.peek() { Token::EqEq => BinOp::Eq, Token::NotEq => BinOp::NotEq, _ => break };
            let span = self.current_span(); self.advance();
            let r = self.parse_cmp()?;
            l = Expr::BinOp { op, left: Box::new(l), right: Box::new(r), span };
        }
        Ok(l)
    }

    fn parse_cmp(&mut self) -> Result<Expr, ParseError> {
        let mut l = self.parse_add()?;
        loop {
            let op = match self.peek() {
                Token::Lt => BinOp::Lt, Token::Gt => BinOp::Gt,
                Token::LtEq => BinOp::LtEq, Token::GtEq => BinOp::GtEq, _ => break,
            };
            let span = self.current_span(); self.advance();
            let r = self.parse_add()?;
            l = Expr::BinOp { op, left: Box::new(l), right: Box::new(r), span };
        }
        Ok(l)
    }

    fn parse_add(&mut self) -> Result<Expr, ParseError> {
        let mut l = self.parse_mul()?;
        loop {
            let op = match self.peek() { Token::Plus => BinOp::Add, Token::Minus => BinOp::Sub, _ => break };
            let span = self.current_span(); self.advance();
            let r = self.parse_mul()?;
            l = Expr::BinOp { op, left: Box::new(l), right: Box::new(r), span };
        }
        Ok(l)
    }

    fn parse_mul(&mut self) -> Result<Expr, ParseError> {
        let mut l = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Token::Star => BinOp::Mul, Token::Slash => BinOp::Div, Token::Percent => BinOp::Rem, _ => break,
            };
            let span = self.current_span(); self.advance();
            let r = self.parse_unary()?;
            l = Expr::BinOp { op, left: Box::new(l), right: Box::new(r), span };
        }
        Ok(l)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        match self.peek() {
            Token::Not   => { self.advance(); let e = self.parse_unary()?; Ok(Expr::UnaryOp { op: UnaryOp::Not, operand: Box::new(e), span }) }
            Token::Minus => { self.advance(); let e = self.parse_unary()?; Ok(Expr::UnaryOp { op: UnaryOp::Neg, operand: Box::new(e), span }) }
            _ => self.parse_call(),
        }
    }

    fn parse_call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;
        loop {
            let span = self.current_span();
            match self.peek() {
                Token::LParen => {
                    self.advance();
                    let args = self.parse_arg_list()?;
                    self.expect(&Token::RParen)?;
                    expr = Expr::Call { callee: Box::new(expr), args, span };
                }
                Token::Dot => {
                    self.advance();
                    let field = self.parse_identifier("acesso a campo")?;
                    expr = Expr::FieldAccess { object: Box::new(expr), field, span };
                }
                Token::LBracket => {
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
            while self.check(&Token::Comma) { self.advance(); args.push(self.parse_expr()?); }
        }
        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        let (l, c) = self.loc();
        match self.peek().clone() {
            Token::Integer(n)   => { self.advance(); Ok(Expr::Literal(Literal::Int(n), span)) }
            Token::Float(f)     => { self.advance(); Ok(Expr::Literal(Literal::Float(f), span)) }
            Token::Scientific(f)=> { self.advance(); Ok(Expr::Literal(Literal::Scientific(f), span)) }
            Token::StringLit(s) => { self.advance(); Ok(Expr::Literal(Literal::String(s), span)) }
            Token::True         => { self.advance(); Ok(Expr::Literal(Literal::Bool(true), span)) }
            Token::False        => { self.advance(); Ok(Expr::Literal(Literal::Bool(false), span)) }
            Token::Null         => { self.advance(); Ok(Expr::Literal(Literal::Null, span)) }
            Token::Identifier(name) => { self.advance(); Ok(Expr::Identifier(name, span)) }
            Token::LParen => {
                self.advance();
                let e = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(e)
            }
            Token::LBracket => {
                self.advance();
                let mut items = Vec::new();
                if !self.check(&Token::RBracket) {
                    items.push(self.parse_expr()?);
                    while self.check(&Token::Comma) { self.advance(); items.push(self.parse_expr()?); }
                }
                self.expect(&Token::RBracket)?;
                Ok(Expr::List(items, span))
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "expressão".into(), found: format!("{:?}", self.peek()), line: l, column: c,
            })
        }
    }

    // ── TIPOS ────────────────────────────────────────────────────────────────

    fn parse_type_expr(&mut self) -> Result<TypeExpr, ParseError> {
        let (l, c) = self.loc();
        match self.peek().clone() {
            Token::Identifier(name) => {
                self.advance();
                let base = match name.as_str() {
                    "Int"    => TypeExpr::Int,
                    "Float"  => TypeExpr::Float,
                    "String" => TypeExpr::String,
                    "Bool"   => TypeExpr::Bool,
                    "Null"   => TypeExpr::Null,
                    other    => TypeExpr::Named(other.to_string()),
                };
                // tipo? — opcional
                if self.check(&Token::Identifier("?".into())) { // nota: ? não é token ainda
                    Ok(TypeExpr::Optional(Box::new(base)))
                } else {
                    Ok(base)
                }
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "tipo".into(), found: format!("{:?}", self.peek()), line: l, column: c,
            })
        }
    }

    // ── FUNÇÃO ───────────────────────────────────────────────────────────────

    fn parse_fn_decl(&mut self) -> Result<FnDecl, ParseError> {
        let span = self.current_span();
        self.expect(&Token::Fn)?;
        let name        = self.parse_identifier("nome da função")?;
        self.expect(&Token::LParen)?;
        let params      = self.parse_param_list()?;
        self.expect(&Token::RParen)?;
        let return_type = if self.check(&Token::Arrow) {
            self.advance(); Some(self.parse_type_expr()?)
        } else { None };
        let body = self.parse_block()?;
        Ok(FnDecl { name, params, return_type, body, span })
    }

    fn parse_param_list(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        if !self.check(&Token::RParen) {
            params.push(self.parse_param()?);
            while self.check(&Token::Comma) { self.advance(); params.push(self.parse_param()?); }
        }
        Ok(params)
    }

    fn parse_param(&mut self) -> Result<Param, ParseError> {
        let span = self.current_span();
        let name = self.parse_identifier("parâmetro")?;
        let type_annotation = if self.check(&Token::Colon) {
            self.advance(); Some(self.parse_type_expr()?)
        } else { None };
        Ok(Param { name, type_annotation, span })
    }

    // ── HELPERS DE VALOR LITERAL ──────────────────────────────────────────────

    fn parse_identifier(&mut self, context: &str) -> Result<String, ParseError> {
        let (l, c) = self.loc();
        match self.peek().clone() {
            Token::Identifier(name) => { self.advance(); Ok(name) }
            _ => Err(ParseError::ExpectedIdentifier { context: context.into(), line: l, column: c })
        }
    }

    fn parse_string_value(&mut self) -> Result<String, ParseError> {
        let (l, c) = self.loc();
        match self.peek().clone() {
            Token::StringLit(s) => { self.advance(); Ok(s) }
            _ => Err(ParseError::UnexpectedToken {
                expected: "string".into(), found: format!("{:?}", self.peek()), line: l, column: c,
            })
        }
    }

    fn parse_u32_literal(&mut self) -> Result<u32, ParseError> {
        let (l, c) = self.loc();
        match self.peek().clone() {
            Token::Integer(n) => { self.advance(); Ok(n as u32) }
            _ => Err(ParseError::UnexpectedToken {
                expected: "inteiro".into(), found: format!("{:?}", self.peek()), line: l, column: c,
            })
        }
    }

    fn parse_u64_literal(&mut self) -> Result<u64, ParseError> {
        let (l, c) = self.loc();
        match self.peek().clone() {
            Token::Integer(n) => { self.advance(); Ok(n as u64) }
            _ => Err(ParseError::UnexpectedToken {
                expected: "inteiro".into(), found: format!("{:?}", self.peek()), line: l, column: c,
            })
        }
    }

    fn parse_usize_literal(&mut self) -> Result<usize, ParseError> {
        let (l, c) = self.loc();
        match self.peek().clone() {
            Token::Integer(n) => { self.advance(); Ok(n as usize) }
            _ => Err(ParseError::UnexpectedToken {
                expected: "inteiro".into(), found: format!("{:?}", self.peek()), line: l, column: c,
            })
        }
    }

    fn parse_bool_value(&mut self) -> Result<bool, ParseError> {
        let (l, c) = self.loc();
        match self.peek() {
            Token::True  => { self.advance(); Ok(true) }
            Token::False => { self.advance(); Ok(false) }
            _ => Err(ParseError::UnexpectedToken {
                expected: "true ou false".into(), found: format!("{:?}", self.peek()), line: l, column: c,
            })
        }
    }
}
```

### 7.2 Checklist — Parser

- [X] Parseia `agent home { on start { ... } }` corretamente
- [X] Parseia `on_error { strategy = "restart_handler" max_retries = 3 }` sem erro
- [X] Parseia `event_policy { on_overflow = "drop_oldest" critical_queue = 100 }` sem erro
- [x] Parseia `ai_policy { timeout_ms = 8000 fallback_response = "..." }` sem erro
- [X] Parseia `channel temperature: Float` sem erro
- [X] Parseia `on start priority high { ... }` com prioridade correta
- [X] Parseia `on start concurrent { ... }` com flag `concurrent = true`
- [X] Parseia `send("monitor").temperature(42.0)` como `SendStmt` com canal tipado
- [X] Parseia `send("agent", "topic", value)` como `SendStmt` legado
- [X] Expressões com precedência correta (`1 + 2 * 3` → `Add(1, Mul(2,3))`)
- [X] `if / else if / else` funciona
- [X] Recuperação de erro (`synchronize`) reporta múltiplos erros
- [X] Erro com linha/coluna em token inesperado

---

## 8. FASE 1.4 — INTERPRETER

**Objetivo:** Executar o AST. Na Fase 1, executa apenas `on start`.

**Separação Definition/Instance — implementada aqui:**

```rust
// crates/crl-interpreter/src/lib.rs

use crl_ast::node::{AgentDecl, Program};
use std::collections::HashMap;

/// Blueprint compilado — imutável, derivado do AST.
/// Na Fase 2, será movido para crl-runtime e enriquecido.
pub struct AgentDefinition {
    pub name:         String,
    pub handlers:     Vec<crl_ast::node::HandlerDecl>,
    pub functions:    Vec<crl_ast::node::FnDecl>,
    // error_policy e event_policy existem no AST mas são ignorados na Fase 1
}

/// Instância em execução — tem environment próprio.
/// Na Fase 2, terá mailbox, contexto e métricas.
pub struct AgentInstance {
    pub definition: std::sync::Arc<AgentDefinition>,
    pub env:        Environment,
}

impl AgentDefinition {
    pub fn from_decl(decl: &AgentDecl) -> Self {
        Self {
            name:      decl.name.clone(),
            handlers:  decl.handlers.clone(),
            functions: decl.functions.clone(),
        }
    }
}
```

**O interpreter cria ambos e executa `on start`:**

```rust
pub fn run(program: Program) -> Result<(), RuntimeError> {
    for stmt in &program.stmts {
        if let Stmt::AgentDecl(decl) = stmt {
            let definition = Arc::new(AgentDefinition::from_decl(decl));
            let mut instance = AgentInstance {
                definition: definition.clone(),
                env: Environment::new_global(),
            };

            // Registrar funções do agent no environment
            for fn_decl in &definition.functions {
                instance.env.define(fn_decl.name.clone(), Value::Function(fn_decl.clone()));
            }

            // Executar on start se existir
            for handler in &definition.handlers {
                if matches!(handler.event, EventPattern::Start) {
                    instance.exec_block(&handler.body)?;
                    break;
                }
            }
        }
    }
    Ok(())
}
```

**Funções nativas obrigatórias:**
- `print(value)` — stdout
- `len(list_or_string)` — tamanho
- `type_of(value)` — nome do tipo como String
- `to_str(value)` — conversão para String

### 8.1 Checklist — Interpreter

- [ ] `AgentDefinition` e `AgentInstance` são structs separadas
- [ ] `let x = expr` avalia e armazena no environment
- [ ] `if / else` com condições booleanas
- [ ] `while` com break implícito
- [ ] `for x in list` itera sobre List
- [ ] Funções com escopo léxico correto
- [ ] `return` funciona dentro de funções
- [ ] `print`, `len`, `type_of`, `to_str` funcionando
- [ ] Handler `on start` executado ao iniciar o agent
- [ ] `error_policy`, `event_policy` no AST — ignorados sem crash
- [ ] `channel` no AST — ignorado sem crash
- [ ] `send(...)` no AST — ignorado sem crash (Fase 4 implementa)
- [ ] Erro com linha/coluna quando variável não existe
- [ ] Erro com linha/coluna em operação inválida

---

## 9. FASE 1.5 — CLI

```rust
// crates/crl-cli/src/main.rs

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("run")              => run_command(&args[2..]),
        Some("check")            => check_command(&args[2..]),
        Some("--version") | Some("-v") => println!("crl {}", env!("CARGO_PKG_VERSION")),
        _                        => print_help(),
    }
}
```

**Comandos obrigatórios:**
- `crl run arquivo.crl` — executa o script
- `crl check arquivo.crl` — verifica sintaxe sem executar
- `crl --version` — imprime versão

---

## 10. TESTES E VALIDAÇÃO

Todos os scripts abaixo devem executar sem erro:

```
-- Teste 1: hello world
agent main { on start { print("Hello, Cognitive Runtime") } }
-- Esperado: Hello, Cognitive Runtime

-- Teste 2: aritmética
agent main { on start { let x = 10 let y = 20 print(x + y) } }
-- Esperado: 30

-- Teste 3: condicional
agent main { on start { let x = 15 if x > 10 { print("maior") } else { print("menor") } } }
-- Esperado: maior

-- Teste 4: loop while
agent main { on start { let i = 0 while i < 5 { print(i) i = i + 1 } } }
-- Esperado: 0 1 2 3 4

-- Teste 5: função com tipos
agent main {
    fn soma(a: Int, b: Int) -> Int { return a + b }
    on start { print(soma(3, 7)) }
}
-- Esperado: 10

-- Teste 6: função sem tipos
agent main {
    fn dobro(n) { return n * 2 }
    on start { print(dobro(21)) }
}
-- Esperado: 42

-- Teste 7: capability declarada (sintaxe aceita, ignorada na Fase 1)
agent home { use lights on start { print("home iniciado") } }
-- Esperado: home iniciado

-- Teste 8: for in lista
agent main { on start { let items = [1, 2, 3] for item in items { print(item) } } }
-- Esperado: 1 2 3

-- Teste 9: erro com linha correta
agent main { on start { let x = y_nao_existe } }
-- Esperado: erro mencionando 'y_nao_existe' com linha e coluna

-- Teste 10: on_error parseado sem crash (Fase 2 implementa a semântica)
agent main {
    on_error {
        strategy    = "restart_handler"
        max_retries = 3
        backoff_ms  = 500
    }
    on start { print("on_error aceito sem crash") }
}
-- Esperado: on_error aceito sem crash

-- Teste 11: event_policy parseado sem crash
agent main {
    event_policy {
        on_overflow    = "drop_oldest"
        critical_queue = 100
    }
    on start { print("event_policy aceito sem crash") }
}
-- Esperado: event_policy aceito sem crash

-- Teste 12: handler com prioridade e channel parseados sem crash
agent main {
    channel temperature: Float
    on start priority high { print("priority e channel aceitos") }
}
-- Esperado: priority e channel aceitos
```

---

## 11. CRITÉRIOS DE CONCLUSÃO DA FASE 1

Só avance para a Fase 2 quando TODOS estes critérios forem atendidos:

**Funcionalidade:**
- [ ] Os 12 scripts de validação executam sem falha
- [ ] Erros de sintaxe mostram linha e coluna exatas
- [ ] Erros de runtime mostram linha e coluna exatas
- [ ] `crl run hello.crl` funciona end-to-end
- [ ] `crl check` detecta e reporta erros de sintaxe

**Qualidade de código:**
- [ ] `cargo test` passa 100%
- [ ] Cobertura do lexer > 90%
- [ ] Cobertura do parser > 85%
- [ ] `cargo clippy -- -D warnings` sem warnings
- [ ] Nenhum `unwrap()` sem comentário `// SAFE: <razão>`
- [ ] Nenhum `panic!()` sem comentário `// INVARIANT: <razão>`

**Arquitetura — fundação para fases seguintes:**
- [ ] Todos os nós do AST têm `Span`
- [ ] `AgentDefinition` e `AgentInstance` são structs separadas
- [ ] `AgentDecl` aceita `on_error`, `event_policy`, `ai_policy`, `channel` sem crash
- [ ] `HandlerDecl` tem campo `priority` e `concurrent`
- [ ] O crate `crl-ast` não tem dependências externas
- [ ] `AgentInstance.env` não usa estado global (seguro para multi-agent)

---

## 12. O QUE NÃO FAZER NESTA FASE

❌ **Não implemente o event loop** — Fase 2.
❌ **Não implemente capabilities reais** — declare, parse, ignore.
❌ **Não implemente concorrência** — tudo sequencial.
❌ **Não implemente bytecode/VM** — AST-walking é correto aqui.
❌ **Não force tipagem estática** — anotações opcionais são suficientes.
❌ **Não conecte IA** — Fase 8.
❌ **Não otimize performance** — corretude primeiro.
❌ **Não use `pest`** — recursive descent é a decisão desta fase.
❌ **Não pule os testes** — cada componente tem testes antes de avançar.
❌ **Não implemente acesso remoto ou MQTT** — Fase 10.
❌ **Não implemente a semântica de `on_error`** — parse, ignore, Fase 2 executa.
❌ **Não implemente a semântica de `event_policy`** — parse, ignore, Fase 2 executa.
❌ **Não implemente channels tipados** — parse, ignore, Fase 4 executa.

---

## 13. ARMADILHAS COMUNS E COMO EVITÁ-LAS

### 13.1 Float antes de Integer no lexer

**Armadilha:** Se `Integer` vier antes de `Float` no enum `Token`, a string
`3.14` será tokenizada como `Integer(3)`, `Dot`, `Integer(14)`.
O parser vai criar uma expressão de field access em vez de um float.

**Solução:** `Scientific` antes de `Float` antes de `Integer` — nesta ordem.

### 13.2 Operadores de 1 char consumindo parte de operadores de 2 chars

**Armadilha:** Se `Eq` (`=`) vier antes de `EqEq` (`==`) no enum,
`==` será tokenizado como `Eq`, `Eq`.

**Solução:** Sempre declare operadores de 2 chars ANTES dos de 1 char.

### 13.3 Keywords consumidas como Identifiers

**Armadilha:** Se `Identifier` vier antes de qualquer keyword no enum,
`agent` será tokenizado como `Identifier("agent")`.

**Solução:** Todas as keywords devem vir ANTES de `Identifier`.

### 13.4 `AgentDecl` sem campos futuros — refatoração cara

**Armadilha:** Implementar `AgentDecl` sem `error_policy`, `event_policy`,
`channels` na Fase 1. Na Fase 2, o parser precisa ser reescrito e todos
os lugares que constroem `AgentDecl` precisam ser atualizados.

**Solução:** Incluir como `Option<T>` e `Vec<T>` agora. Custo zero, benefício enorme.

### 13.5 Misturar AgentDefinition e AgentInstance

**Armadilha:** Usar o `AgentDecl` do AST diretamente como estado de execução.
Quando a Fase 9 exigir múltiplas instâncias do mesmo blueprint, a estrutura
toda precisará ser reescrita.

**Solução:** `AgentDefinition` = blueprint imutável (Arc<>). `AgentInstance` = estado vivo.
Separação desde a Fase 1, mesmo que na Fase 1 exista apenas uma instância.

### 13.6 Fila de eventos plana na Fase 2

**Armadilha:** Implementar o Event Bus da Fase 2 com uma única `VecDeque`.
Um sensor de emergência fica atrás de ticks de timer.

**Solução:** Desde o início da Fase 2, usar 4 filas separadas por prioridade.
`BinaryHeap` ou 4 `VecDeque` com poll em cascata. Nunca uma fila única.

### 13.7 Handlers com falha silenciosa

**Armadilha:** Implementar o runtime da Fase 2 sem tratar `Err` dos handlers.
Um handler que quebra simplesmente para de funcionar sem aviso.

**Solução:** Todo handler é executado dentro do `Supervisor`.
Qualquer `Err` aciona a `ErrorPolicy` do agent antes de qualquer outra ação.

### 13.8 `send()` sem tipo verificado

**Armadilha:** Implementar `send("agent", "topic", value)` na Fase 4 com `Value`
genérico. Um agent muda o tipo do payload e o receptor quebra silenciosamente.

**Solução:** Canais declarados com `channel nome: Tipo`. O type checker verifica
compatibilidade antes de executar. `Value` genérico em channel é um smell.

---

*Versão: 3.0.0 | Revisão: Auditoria técnica — 10 lacunas incorporadas*
*Próximo documento: CRL_FASE_2.md (Runtime Contínuo, Supervisor, Observer, Event Bus com Prioridade)*
