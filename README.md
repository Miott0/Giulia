# Giulia — Cognitive Runtime Language

Repositório do projeto Giulia — Fase 0–1 (setup inicial).

Resumo
- Implementação em Rust; objetivo inicial: lexer, AST, parser, interpreter e CLI (Fase 1).
- Leia os documentos de especificação: [CRL_FASE_0_1.md](CRL_FASE_0_1.md) and [CRL_MAPA_GERAL.md](CRL_MAPA_GERAL.md).

Setup rápido
1. Instale o Rust toolchain (rustup):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustc --version  # deve ser >= 1.75
cargo --version
```

2. Build inicial:

```bash
cargo build
```

3. Editor recomendado: VSCode com extensões `rust-analyzer`, `Even Better TOML`, `Error Lens`.

O que eu já configurei aqui
- `.gitignore` (arquivos de build e editor)
- `examples/phase1/hello.crl` (exemplo mínimo)
- memória de contexto criada: /memories/repo/memory-context.md

Próximos passos sugeridos
- Confirmar se deseja que eu crie os crates esqueleto (`giulia-lexer`, `giulia-ast`, `giulia-parser`, `giulia-interpreter`, `giulia`) agora.
- Ou prefira que eu apenas prepare o ambiente e documente os passos para você executar localmente.
