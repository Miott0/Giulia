# CRL — Cognitive Runtime Language
## Mapa Geral do Projeto · Do Zero ao Jarvis
### Documento de Arquitetura e Roadmap Completo · Versão 2.0

---

> **Como usar este documento**
> Este é o mapa permanente do projeto. Ele descreve cada fase, cada decisão arquitetural,
> cada componente e cada risco. Consulte-o antes de iniciar qualquer fase nova.
> Ele deve ser atualizado conforme o projeto evolui e decisões são revisadas.
>
> **v2.0 — Revisão de engenharia:** Decisões rígidas convertidas em hipóteses dirigidas.
> Roadmap expandido com camada de rede/IoT, acesso remoto e adaptação comportamental.
> Separação explícita entre Linguagem, Runtime e Ecossistema.

---

## ÍNDICE

1. Visão do Projeto
2. Três Blocos Arquiteturais (Linguagem · Runtime · Ecossistema)
3. Decisões Arquiteturais — Firmes vs. Hipóteses
4. Modelo Mental do Sistema
5. Pilha Tecnológica
6. Estrutura de Diretórios Final
7. Gramática Formal da Linguagem (EBNF)
8. Roadmap Completo — Fase 0 a Fase 10
9. Glossário Técnico
10. Riscos e Mitigações
11. Princípios que Nunca Devem Ser Violados

---

## 1. VISÃO DO PROJETO

### O que é o CRL

CRL (Cognitive Runtime Language) é uma linguagem de programação e runtime criados
do zero com um propósito único: ser a fundação computacional de agentes cognitivos
contínuos — sistemas que percebem o mundo, reagem a eventos, lembram de contexto,
integram inteligência artificial como uma camada plugável, e se comunicam com
dispositivos, redes e usuários remotos de forma controlada e segura.

### O que NÃO é o CRL

- Não é um framework de IA (como LangChain ou AutoGen)
- Não é uma linguagem de uso geral (como Python ou Rust)
- Não é um assistente virtual pronto (como Alexa ou Google Assistant)
- Não é uma linguagem de scripting simples
- Não é um sistema de automação residencial (como Home Assistant)

### A metáfora central

```
IA          = o motor         (gera potência cognitiva — plugável, substituível)
Runtime CRL = o chassi        (estrutura que segura tudo — o que você está construindo)
Scheduler   = a transmissão   (distribui o trabalho)
Memória     = o tanque        (armazena contexto e histórico)
Capabilities= as rodas        (tocam o mundo real — hardware, APIs, IoT)
Sensores    = os olhos/ouvidos (percepção contínua)
Atuadores   = as mãos         (ação no mundo)
Rede/IoT    = as estradas     (contexto externo e dispositivos conectados)
```

O motor não dirige o carro. O chassi dirige o motor.

---

## 2. TRÊS BLOCOS ARQUITETURAIS

Esta separação é fundamental. Confundir os três blocos causa overengineering
prematuro e acoplamento desnecessário.

```
┌─────────────────────────────────────────────────────────────────┐
│  BLOCO 1 — LINGUAGEM                                            │
│  Gramática · Lexer · Parser · AST · Semântica · Tipos           │
│  → Define como o programador escreve agents e handlers          │
├─────────────────────────────────────────────────────────────────┤
│  BLOCO 2 — RUNTIME                                              │
│  Scheduler · Event Bus · Agents · Contexto · Capabilities       │
│  → Define como o sistema executa e reage continuamente          │
├─────────────────────────────────────────────────────────────────┤
│  BLOCO 3 — ECOSSISTEMA                                          │
│  Plugins · Rede · Dispositivos IoT · IA · Aprendizado           │
│  → Define como o sistema se conecta ao mundo externo            │
└─────────────────────────────────────────────────────────────────┘
```

**Regra de dependência:**
- Linguagem não depende de Runtime nem de Ecossistema
- Runtime depende da Linguagem (executa o código)
- Ecossistema depende do Runtime (capabilities são chamadas pelo runtime)
- Essa direção nunca se inverte

---

## 3. DECISÕES ARQUITETURAIS — FIRMES vs. HIPÓTESES

### Como ler esta seção

Cada decisão tem uma classificação:
- **FIRME** — embasada em requisitos concretos, não deve ser revertida sem análise de impacto
- **HIPÓTESE ATUAL** — direção preferida, mas a ser validada na prática antes de comprometer
- **DECISÃO DE FASE** — será tomada em momento específico, não antes

---

### 3.1 Rust como linguagem de implementação

**Classificação: FIRME**

**Raciocínio:**
O sistema opera 24/7 integrado com hardware real (microfone, câmera, sensores, IoT).
Falhas de memória, data races e vazamentos não são aceitáveis nesse contexto.
Rust oferece segurança de memória sem GC, `async/await` nativo e ecossistema WASM
robusto — os três pilares de que o CRL precisa.

---

### 3.2 Arquitetura event-driven com Actor Model

**Classificação: FIRME**

**Raciocínio:**
Agents são naturalmente isolados: têm estado próprio, reagem a eventos, comunicam
por mensagens. O Actor Model é o mapeamento direto dessa arquitetura. Compartilhamento
de memória entre agents seria uma fonte constante de bugs em um sistema contínuo.

**Estrutura interna desde o início:**
```rust
struct Runtime {
    agents: HashMap<AgentId, AgentInstance>,  // mapa, não valor único — multi-agent ready
    event_bus: EventBus,
    context_store: ContextStore,              // isolado por AgentId
    capability_registry: CapabilityRegistry,
    scheduler: Scheduler,
}
```

**Regras imutáveis:**
- Agents NUNCA compartilham memória diretamente
- Comunicação SEMPRE por channels tipados ou eventos
- IO bloqueante vai para `spawn_blocking`
- CPU intensivo vai para worker threads dedicados

---

### 3.3 Capability-Based Security

**Classificação: FIRME**

**Raciocínio:**
Um agent que controla luzes não deve poder acessar câmeras sem declaração explícita.
O sistema integra com hardware real, APIs externas e dispositivos IoT — o princípio
do menor privilégio não é opcional, é uma propriedade de segurança fundamental.

**Modelo de capabilities:**
```
agent home {
    use lights       -- declara explicitamente o que pode usar
    use thermostat

    on start {
        lights.enable()     -- permitido
        camera.view()       -- ERRO: capability não declarada
    }
}
```

---

### 3.4 Tipagem: Progressiva em direção a Estática com Inferência

**Classificação: HIPÓTESE ATUAL → Decisão de Fase 3**

**Revisão da v1:**
A v1 declarava "tipagem estática com inferência total" como decisão imutável desde
a Fase 1. Isso foi prematuro. Inferência completa é um compilador em si, e implementar
isso antes de ter um MVP funcional é overengineering que atrasa o projeto sem entregar valor.

**Abordagem progressiva adotada:**

| Fase | Tipagem | O que isso entrega |
|---|---|---|
| 1–2 | Dinâmica com anotações opcionais | MVP funcional, scripts rodam |
| 3 | Verificação de capabilities em tempo de análise | Segurança de acesso |
| 4–5 | Tipos obrigatórios em assinaturas de funções | Contratos claros |
| 6+ | Inferência básica para variáveis locais | Ergonomia |
| 7+ | Sistema de efeitos tipados | Auditabilidade formal |

**Invariante que nunca muda:**
Errors de tipo devem sempre indicar linha, coluna e contexto exatos.

---

### 3.5 Sistema de Efeitos: Progressivo em 3 Camadas

**Classificação: FIRME quanto à direção, HIPÓTESE quanto à implementação**

**Camada 1 — Capabilities como guardas (Fases 1–3):**
```
agent home {
    use lights
    on start {
        lights.enable()   -- runtime verifica se a capability está declarada
    }
}
```

**Camada 2 — Efeitos marcados sintaticamente (Fases 4–6):**
```
on speech {
    let intent = ai.reason(context)   -- efeito AI (rastreado pelo tipo de retorno)
    do lights.enable()                -- "do" marca side effect IO explicitamente
}
```

**Camada 3 — Sistema algébrico formal (Fases 7+):**
```
-- funções declaram seus efeitos no tipo
fn enable_lights() -> () ! [IO, RequiresCapability(Lights)] { ... }
```

**Regra:** A camada 3 NÃO deve ser implementada antes da Fase 7.
Sistemas de efeitos algébricos são complexos demais para fases iniciais.

---

### 3.6 Multi-Agent: Arquitetura Pronta, Complexidade Gradual

**Classificação: FIRME quanto à arquitetura interna, HIPÓTESE quanto à exposição**

**O que entra desde o Dia 1:**
A estrutura interna do runtime usa `HashMap<AgentId, AgentInstance>` desde a Fase 1.
Refatorar a fundação para multi-agent depois é equivalente a trocar os alicerces
de um prédio em uso.

**O que entra progressivamente:**
- Fase 1–2: Um único agent executa corretamente
- Fase 4: Dois agents se comunicam por channel
- Fase 5+: N agents com contextos isolados e coordenados

---

### 3.7 Estratégia de Execução: AST-Walking → Bytecode VM

**Classificação: DECISÃO DE FASE**

**Fase 1–5:** AST-walking interpreter. Mais simples de implementar, debugar e
estender. Performance suficiente para os primeiros casos de uso.

**Fase 6+:** Avaliar se a performance é aceitável. Se não for (ex: processamento
de percepção multimodal sendo bloqueado pelo interpretador), introduzir bytecode VM.

**Critério concreto para a decisão:** Se o tempo de eval de um handler for > 1ms
em média para scripts simples, a VM deve ser considerada. Antes disso, não.

---

### 3.8 Acesso Remoto: Celular, Web e APIs

**Classificação: FIRME quanto ao modelo de segurança, DECISÃO DE FASE quanto à implementação**

**Modelo de acesso:**
O celular (ou painel web) não executa código CRL diretamente.
Ele envia **intenções autenticadas** ao runtime. O runtime valida, autoriza e executa.

```
Celular / Web → Intenção assinada → Gateway do Runtime
                                          │
                                    Autenticação
                                          │
                                    Autorização (capabilities)
                                          │
                                    Execução controlada
                                          │
                                    Resposta ao cliente
```

**Protocolos suportados (planejados para Fase 10):**
- WebSocket (bidireccional, baixa latência para comandos interativos)
- HTTP/REST (APIs, webhooks, integrações pontuais)
- MQTT (IoT, mensagens leves para sensores e atuadores)

**Segurança obrigatória:**
- Toda intenção remota requer autenticação
- Intenções remotas têm capabilities restritas (subset do que o agent local tem)
- Nenhuma intenção remota pode invocar capabilities não pré-autorizadas

---

### 3.9 Integração com Dispositivos IoT

**Classificação: FIRME quanto ao modelo, DECISÃO DE FASE quanto à implementação**

**Modelo de integração:**
Dispositivos IoT são tratados como **capabilities de rede**. O runtime não distingue
entre uma capability local (ex: microfone USB) e uma capability de rede (ex: sensor
ESP32 via MQTT) — ambas seguem o mesmo contrato de capability.

**Dispositivos planejados:**
- Smart home (Home Assistant, Matter, Zigbee)
- Microcontroladores (ESP32, Arduino) via MQTT
- Single-board computers (Raspberry Pi) via TCP/HTTP
- Sensores genéricos (temperatura, presença, luminosidade)

**Protocolo preferencial para IoT:** MQTT — é o barramento natural para dispositivos
com restrições de recursos, suporta pub/sub nativo e tem ecossistema maduro em Rust.

---

### 3.10 Aprendizado e Adaptação Comportamental

**Classificação: HIPÓTESE ATUAL — entra na Fase 9**

O sistema pode ser adaptativo sem prometer nem simular consciência.
Adaptação é modelada como **ajuste de parâmetros persistentes** baseado em histórico observável.

**Mecanismos de adaptação planejados:**

| Mecanismo | Implementação | Exemplo |
|---|---|---|
| Memória consolidável | Context Layer: episódico → persistente | Lembrar preferências do usuário |
| Esquecimento seletivo | TTL + relevância no context store | Limpar contexto de conversas antigas |
| Ajuste de tom | Parâmetros de personalidade mutáveis | Responder mais formal após feedback |
| Formação de hábitos | Padrões de evento → resposta registrados | "Toda segunda de manhã, X acontece" |
| Melhoria de estratégias | Log de sucesso/falha de planos | Evitar ações que falharam antes |

**O que NÃO é adaptação no escopo do CRL:**
- Fine-tuning de modelos de linguagem (isso é do motor de IA, não do runtime)
- Aprendizado por reforço profundo (fora do escopo)
- "Consciência emergente" (ficção científica, não engenharia)

---

### 3.11 Personalidade como Conjunto de Parâmetros Persistentes

**Classificação: HIPÓTESE ATUAL — entra na Fase 9**

Personalidade é um **bloco de configuração versionado**, não um conceito mágico.
É um conjunto de parâmetros que o runtime injeta no contexto de cada agent.

```
personality {
    name          = "Jarvis"
    tone          = "professional"      -- formal | casual | terse | verbose
    language      = "pt-BR"
    response_style= "concise"
    memory_style  = "episodic"          -- episodic | semantic | short_term_only
    boundaries    = ["no_financial_advice", "no_medical_diagnosis"]
    priorities    = ["security", "user_comfort", "energy_efficiency"]
}
```

Esses parâmetros são consumidos pelo motor de IA (Fase 8) para contextualizar
respostas, e pelo runtime para filtrar ações (um agent com `boundaries = ["no_financial_advice"]`
não deve executar capabilities relacionadas a transações financeiras).

---

## 4. MODELO MENTAL DO SISTEMA

### Fluxo de execução completo

```
Mundo Físico / Digital / Rede
        │
        │  (áudio, vídeo, sensor, timer, API, mensagem, comando remoto)
        ▼
┌───────────────────────┐
│  CAMADA DE ENTRADA    │   Captura de sinais do mundo e intenções remotas
│  (Fases 2, 6, 10)     │   Drivers locais + Gateway de rede
└────────┬──────────────┘
         │  raw signal / authenticated intent
         ▼
┌───────────────────────┐
│  EVENT SYSTEM         │   Converte sinais/intenções em Eventos tipados
│  (Fase 2)             │   SpeechEvent, TimerEvent, NetworkEvent, SensorEvent...
└────────┬──────────────┘
         │  typed Event
         ▼
┌───────────────────────┐
│  EVENT BUS            │   Roteia evento para o(s) agent(s) correto(s)
│  (Fase 2)             │   Baseado em subscriptions e capabilities
└────────┬──────────────┘
         │  dispatched Event
         ▼
┌───────────────────────┐
│  SCHEDULER            │   Agenda execução do handler do agent
│  (Fase 2)             │   Prioridade, fairness, backpressure
└────────┬──────────────┘
         │  scheduled Task
         ▼
┌───────────────────────┐
│  AGENT RUNTIME        │   Executa o handler com contexto injetado
│  (Fase 1+)            │   Interpreta o código CRL do handler
└────────┬──────────────┘
         │  context read/write, capability calls
         ▼
┌───────────────────────┐
│  CONTEXT ENGINE       │   Memória curta, persistente, vetorial
│  (Fase 5)             │   Isolada por agent, auditável, adaptativa
└────────┬──────────────┘
         │  AI calls quando necessário
         ▼
┌───────────────────────┐
│  COGNITIVE LAYER      │   LLMs, embeddings, reasoning, planning
│  (Fase 8)             │   Motor, não controlador — sempre substituível
└────────┬──────────────┘
         │  actions / effects
         ▼
┌───────────────────────┐
│  CAPABILITY SYSTEM    │   Verifica permissão antes de executar ação
│  (Fase 3)             │   lights, camera, speaker, MQTT, HTTP, IoT...
└────────┬──────────────┘
         │
         ▼
Mundo Físico / Digital / Rede  (luzes acendem, resposta é dada, dispositivo é controlado)
```

---

## 5. PILHA TECNOLÓGICA

### Linguagem de implementação: Rust

| Atributo Rust | Benefício para o CRL |
|---|---|
| Ownership + Borrow Checker | Elimina data races entre agents sem GC |
| Zero-cost abstractions | Runtime com performance de C |
| `async/await` nativo | Base para o actor model leve |
| Ecossistema de parsers | `logos`, `pest`, `nom` para o lexer/parser |
| Tokio | Runtime assíncrono battle-tested para o scheduler |
| WASM support | Isolamento de plugins via WebAssembly |
| `serde` | Serialização de contexto e memória |

### Dependências por fase

```toml
# Fase 1 — Núcleo da linguagem
logos       = "0.14"          # Lexer por derive macro
thiserror   = "1.0"           # Erros tipados e ergonômicos
miette      = "5.10"          # Diagnósticos de erro com highlighting
# NOTA: pest removido da Fase 1 — usar recursive descent manual.
# pest adiciona indireção que complica o diagnóstico de erro na fase inicial.

# Fase 2 — Runtime contínuo
tokio       = { version = "1", features = ["full"] }
tokio-util  = "0.7"
async-trait = "0.1"

# Fase 3 — Capabilities
# Sem dependência nova — registry implementado em Rust puro

# Fase 4 — Concorrência
# Tokio já inclui channels; não há nova dependência

# Fase 5 — Contexto e memória
serde       = { version = "1", features = ["derive"] }
serde_json  = "1"
sled        = "0.34"          # Preferido sobre RocksDB: puro Rust, sem bindings C
# RocksDB como alternativa se performance de escrita for crítica (avaliar na Fase 5)

# Fase 6 — Percepção
cpal        = "0.15"          # Audio cross-platform
nokhwa      = "0.10"          # Câmera (mais simples que opencv para fase inicial)
# opencv = "0.88" como upgrade se reconhecimento avançado for necessário

# Fase 7 — Plugins
wasmtime    = "18"            # Runtime WASM para isolamento de plugins

# Fase 8 — IA
async-openai = "0.23"         # Integração LLM
candle-core  = "0.6"          # Modelos locais (Mistral, LLaMA, Whisper)

# Fase 9/10 — Rede e IoT
axum         = "0.7"          # HTTP server para gateway de acesso remoto
tokio-tungstenite = "0.21"    # WebSocket
rumqttc      = "0.24"         # MQTT client (puro Rust)
jsonwebtoken = "9"            # JWT para autenticação de intenções remotas
```

**Por que `sled` ao invés de `rocksdb`:**
RocksDB requer bindings C e tem overhead de compilação significativo.
`sled` é implementado inteiramente em Rust, compila sem problemas e tem performance
adequada para o caso de uso (memória de agent, não banco de dados de produção com
milhões de transações/segundo). Migrar para RocksDB é trivial se necessário.

---

## 6. ESTRUTURA DE DIRETÓRIOS FINAL

```
crl/
│
├── Cargo.toml                      # Workspace root
├── Cargo.lock
├── README.md
├── LICENSE
│
├── docs/
│   ├── language-spec.md            # Especificação formal da linguagem
│   ├── architecture.md             # Este documento (versão técnica)
│   ├── grammar.ebnf                # Gramática formal completa
│   ├── decisions/                  # ADRs — Architecture Decision Records
│   │   ├── adr-001-rust.md
│   │   ├── adr-002-actor-model.md
│   │   └── adr-003-typing-strategy.md
│   └── phases/
│       ├── phase-0.md
│       ├── phase-1.md
│       └── ...
│
├── crates/
│   │
│   ├── crl-lexer/                  # FASE 1
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── token.rs
│   │       └── error.rs
│   │
│   ├── crl-ast/                    # FASE 1
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── node.rs             # Nós do AST com Span em todos
│   │       ├── types.rs            # Sistema de tipos progressivo
│   │       └── visitor.rs          # Visitor pattern para traversal
│   │
│   ├── crl-parser/                 # FASE 1
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── parser.rs           # Recursive descent parser
│   │       └── error.rs
│   │
│   ├── crl-typechecker/            # FASE 1 (mínimo) → FASE 3 (capabilities) → FASE 6 (completo)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── checker.rs          # Verificação de tipos e capabilities
│   │       └── env.rs              # Ambiente de tipos
│   │
│   ├── crl-interpreter/            # FASE 1
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── eval.rs             # Avaliador de expressões e statements
│   │       ├── env.rs              # Ambiente de variáveis com escopo léxico
│   │       └── value.rs            # Tipos de valores em runtime
│   │
│   ├── crl-runtime/                # FASE 2
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── event_bus.rs        # Event bus central
│   │       ├── scheduler.rs        # Scheduler de tasks com prioridade
│   │       ├── dispatcher.rs       # Dispatcher: event bus → scheduler
│   │       └── agent_handle.rs     # Handle para um agent em execução
│   │
│   ├── crl-capabilities/           # FASE 3
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── registry.rs         # Registry global de capabilities
│   │       ├── checker.rs          # Verificação de permissão
│   │       └── builtin/
│   │           ├── lights.rs
│   │           ├── microphone.rs
│   │           ├── camera.rs
│   │           └── logger.rs
│   │
│   ├── crl-context/                # FASE 5
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── working.rs          # Memória do handler (variáveis locais)
│   │       ├── session.rs          # Memória de sessão (HashMap em memória)
│   │       ├── persistent.rs       # Memória persistente (sled)
│   │       ├── episodic.rs         # Memória episódica (Fase 9)
│   │       └── context.rs          # AgentContext: fachada unificada
│   │
│   ├── crl-perception/             # FASE 6
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── audio.rs            # Pipeline: mic → VAD → STT → SpeechEvent
│   │       ├── video.rs            # Pipeline: camera → frame → ImageEvent
│   │       └── sensor.rs           # Trait genérico de sensor
│   │
│   ├── crl-plugins/                # FASE 7
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── loader.rs           # Carregamento dinâmico de plugins WASM
│   │       ├── registry.rs         # Registry de plugins com versionamento
│   │       └── sandbox.rs          # Restrições de acesso do plugin
│   │
│   ├── crl-cognitive/              # FASE 8
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── llm.rs              # Trait abstrato de LLM + providers
│   │       ├── embeddings.rs       # Embeddings para memória semântica
│   │       └── planner.rs          # Planejamento de ações
│   │
│   ├── crl-network/                # FASE 10
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── gateway.rs          # HTTP/WebSocket gateway para acesso remoto
│   │       ├── mqtt.rs             # Cliente MQTT para IoT
│   │       ├── auth.rs             # Autenticação de intenções remotas (JWT)
│   │       └── intent.rs           # Modelo de intenção autenticada
│   │
│   └── crl-cli/                    # TODAS AS FASES
│       └── src/
│           ├── main.rs
│           ├── run.rs              # crl run arquivo.crl
│           ├── check.rs            # crl check arquivo.crl
│           └── repl.rs             # crl repl
│
└── examples/
    ├── phase1/
    │   ├── hello.crl
    │   └── functions.crl
    ├── phase2/
    │   └── event_loop.crl
    ├── phase5/
    │   └── memory_agent.crl
    └── phase10/
        └── home_agent.crl
```

---

## 7. GRAMÁTICA FORMAL DA LINGUAGEM (EBNF)

**Nota de versionamento:** Esta gramática é versionada por fase.
Cada construção indica em qual fase é implementada. Nunca implementar
construções além da fase atual antes dos deliverables da fase estarem completos.

```ebnf
(* ============================================================ *)
(* CRL — Cognitive Runtime Language Grammar v2                   *)
(* ============================================================ *)

(* ---- PROGRAMA ROOT ---- *)
program         ::= statement* EOF

(* ---- DECLARAÇÕES DE ALTO NÍVEL ---- *)
statement       ::= agent_decl
                  | fn_decl
                  | let_stmt
                  | expr_stmt

(* ---- AGENT (Fase 1 básico, Fase 3 com capabilities, Fase 9 com personality) ---- *)
agent_decl      ::= "agent" IDENTIFIER "{" agent_body "}"

agent_body      ::= capability_decl*
                    personality_decl?
                    handler_decl*
                    fn_decl*

capability_decl ::= "use" IDENTIFIER NEWLINE              (* Fase 3 *)
                  | "use" "plugin" "(" STRING ")" NEWLINE (* Fase 7 *)

personality_decl ::= "personality" "{" personality_entry* "}"  (* Fase 9 *)
personality_entry ::= IDENTIFIER "=" literal NEWLINE

(* ---- HANDLERS DE EVENTO ---- *)
handler_decl    ::= "on" event_pattern block

event_pattern   ::= "start"                                    (* Fase 1 *)
                  | "stop"                                     (* Fase 1 *)
                  | "timer" "(" expr ")"                       (* Fase 2 *)
                  | "message" "(" IDENTIFIER ")"               (* Fase 4 *)
                  | "speech"                                   (* Fase 6 *)
                  | "image"                                    (* Fase 6 *)
                  | "sensor_change" "(" IDENTIFIER ")"         (* Fase 6 *)
                  | "network" "(" IDENTIFIER ")"               (* Fase 10 *)
                  | "idle" "(" expr ")"                        (* Fase 9 *)
                  | IDENTIFIER                                 (* evento customizado *)

(* ---- FUNÇÕES (Fase 1) ---- *)
fn_decl         ::= "fn" IDENTIFIER "(" param_list? ")" ("->" type_expr)? block

param_list      ::= param ("," param)*
param           ::= IDENTIFIER (":" type_expr)?               (* anotação opcional Fase 1, obrigatória Fase 4 *)

(* ---- BLOCO DE CÓDIGO ---- *)
block           ::= "{" stmt_inner* "}"

stmt_inner      ::= let_stmt
                  | assign_stmt
                  | if_stmt
                  | while_stmt
                  | for_stmt
                  | return_stmt
                  | effect_stmt
                  | expr_stmt

(* ---- STATEMENTS ---- *)
let_stmt        ::= "let" IDENTIFIER (":" type_expr)? "=" expr NEWLINE
assign_stmt     ::= IDENTIFIER "=" expr NEWLINE
return_stmt     ::= "return" expr? NEWLINE
effect_stmt     ::= "do" expr NEWLINE                (* Fase 4: marca side effect explícito *)
expr_stmt       ::= expr NEWLINE

(* ---- CONTROLE DE FLUXO ---- *)
if_stmt         ::= "if" expr block ("else" (if_stmt | block))?
while_stmt      ::= "while" expr block
for_stmt        ::= "for" IDENTIFIER "in" expr block

(* ---- EXPRESSÕES ---- *)
expr            ::= or_expr

or_expr         ::= and_expr ("or" and_expr)*
and_expr        ::= eq_expr ("and" eq_expr)*
eq_expr         ::= cmp_expr (("==" | "!=") cmp_expr)*
cmp_expr        ::= add_expr (("<" | ">" | "<=" | ">=") add_expr)*
add_expr        ::= mul_expr (("+" | "-") mul_expr)*
mul_expr        ::= unary_expr (("*" | "/" | "%") unary_expr)*
unary_expr      ::= ("not" | "-") unary_expr | call_expr
call_expr       ::= primary_expr (call_suffix | index_suffix | field_suffix)*

call_suffix     ::= "(" arg_list? ")"
index_suffix    ::= "[" expr "]"
field_suffix    ::= "." IDENTIFIER

arg_list        ::= expr ("," expr)*

primary_expr    ::= literal
                  | IDENTIFIER
                  | "(" expr ")"
                  | lambda_expr            (* Fase 4 *)

lambda_expr     ::= "|" param_list? "|" (block | expr)  (* Fase 4 *)

(* ---- LITERAIS ---- *)
literal         ::= INTEGER
                  | FLOAT
                  | STRING
                  | BOOL
                  | "null"
                  | list_literal
                  | map_literal

list_literal    ::= "[" (expr ("," expr)*)? "]"
map_literal     ::= "{" (map_entry ("," map_entry)*)? "}"
map_entry       ::= STRING ":" expr

(* ---- SISTEMA DE TIPOS ---- *)
(* Fase 1: tipos básicos, anotação opcional                      *)
(* Fase 3: tipos de capability verificados                       *)
(* Fase 4: anotações obrigatórias em assinaturas de função       *)
(* Fase 7+: sistema de efeitos completo                          *)

type_expr       ::= "Int"
                  | "Float"
                  | "String"
                  | "Bool"
                  | "Null"
                  | "List" "<" type_expr ">"
                  | "Map" "<" type_expr "," type_expr ">"
                  | IDENTIFIER
                  | type_expr "?"                             (* opcional, Fase 3 *)
                  | type_expr "!" "[" effect_list "]"        (* efeitos, Fase 7 *)

effect_list     ::= IDENTIFIER ("," IDENTIFIER)*

(* ---- TOKENS TERMINAIS ---- *)
IDENTIFIER      ::= [a-zA-Z_][a-zA-Z0-9_]*
INTEGER         ::= [0-9]+
FLOAT           ::= [0-9]+ "." [0-9]+
STRING          ::= '"' [^"]* '"'
BOOL            ::= "true" | "false"
NEWLINE         ::= "\n" | "\r\n"
COMMENT         ::= "--" [^\n]* NEWLINE

(* ---- PALAVRAS RESERVADAS ---- *)
(* agent, use, plugin, on, fn, let, return, if, else, while, for, in,
   do, true, false, null, and, or, not, start, stop, speech, image,
   timer, sensor_change, message, network, idle, personality *)
```

---

## 8. ROADMAP COMPLETO — FASE 0 A FASE 10

---

### FASE 0 — Estudo e Modelagem Conceitual

**Duração estimada:** 2–4 semanas
**Objetivo:** Dominar os fundamentos antes de escrever código.

**Por que esta fase existe:**
Sem ela, você vai implementar o lexer e descobrir que o parser não casa com a gramática.
Vai criar o event bus e descobrir que não modelou os tipos de evento corretamente.
Vai reescrever o AST 3 vezes por não entender o modelo de concorrência.

**O que estudar:**

| Tópico | Por que importa para o CRL | Recurso |
|---|---|---|
| Compiladores e Interpreters | Lexer, parser, AST, eval são o núcleo da Fase 1 | "Crafting Interpreters" (Nystrom) — gratuito online |
| Actor Model | Modelo de concorrência dos agents | "Programming Erlang" (Armstrong) + Wikipedia |
| Event Sourcing | Modelo para memória auditável | Martin Fowler: bliki/EventSourcing |
| Capability-Based Security | Modelo de permissões | "Capability Myths Demolished" (Miller et al) |
| Sistemas de Tipos | Base para tipagem progressiva | "Types and Programming Languages" (Pierce) — caps. 1–5 |
| MQTT Protocol | Base para IoT | Documentação oficial mosquitto + HiveMQ guide |

**Deliverables desta fase:**
- [ ] Gramática EBNF v1 (subset da Fase 1) escrita e revisada
- [ ] Diagrama de fluxo do sistema desenhado (papel ou diagrama digital)
- [ ] Ambiente Rust configurado: workspace criado, `cargo build` sem erros
- [ ] Documento pessoal: "Como o CRL vai executar este código passo a passo?"

---

### FASE 1 — Núcleo da Linguagem

**Duração estimada:** 4–8 semanas
**Objetivo:** Parsear e executar scripts CRL simples.

**Meta verificável ao final:**
```
-- arquivo: hello.crl
agent main {
    on start {
        let msg = "Hello, Cognitive Runtime"
        print(msg)
    }
}
```
Este script deve executar e imprimir a mensagem.

**Componentes:**

#### 1.1 Lexer (`crl-lexer`)

Converte texto CRL em sequência de tokens com posição (linha, coluna).

**Implementação:** `logos` crate — reduce ~500 linhas para ~50.

**Tokens obrigatórios na Fase 1:**
```
Keywords:    agent, on, fn, let, return, if, else, while, for, in,
             use, do, and, or, not, true, false, null, start, stop
Literals:    Integer(i64), Float(f64), String(String), Bool implícito via keywords
Operators:   +, -, *, /, %, ==, !=, <, >, <=, >=, =
Delimiters:  {, }, (, ), [, ], ",", ".", ":"
Special:     Identifier(String), Newline, Eof
```

**Erros:** Todo token inválido deve gerar erro com linha, coluna e caractere exatos.

#### 1.2 AST (`crl-ast`)

Representação estruturada do programa em memória.

**CRÍTICO:** Todo nó do AST carrega `Span { start: usize, end: usize }` para diagnósticos precisos.

```rust
enum Stmt {
    AgentDecl { name: String, body: AgentBody, span: Span },
    FnDecl { name: String, params: Vec<Param>, return_type: Option<TypeExpr>, body: Block, span: Span },
    LetStmt { name: String, type_ann: Option<TypeExpr>, value: Expr, span: Span },
    AssignStmt { name: String, value: Expr, span: Span },
    IfStmt { condition: Expr, then_branch: Block, else_branch: Option<Block>, span: Span },
    WhileStmt { condition: Expr, body: Block, span: Span },
    ForStmt { var: String, iterable: Expr, body: Block, span: Span },
    ReturnStmt { value: Option<Expr>, span: Span },
    EffectStmt { expr: Expr, span: Span },   // "do expr" — Fase 4
    ExprStmt { expr: Expr, span: Span },
}

enum Expr {
    Literal(Literal, Span),
    Identifier(String, Span),
    BinOp { op: BinOp, left: Box<Expr>, right: Box<Expr>, span: Span },
    UnaryOp { op: UnaryOp, operand: Box<Expr>, span: Span },
    Call { callee: Box<Expr>, args: Vec<Expr>, span: Span },
    FieldAccess { object: Box<Expr>, field: String, span: Span },
    Index { object: Box<Expr>, index: Box<Expr>, span: Span },
    List(Vec<Expr>, Span),
    Map(Vec<(Expr, Expr)>, Span),
}
```

#### 1.3 Parser (`crl-parser`)

Recursive descent parser. Cada regra da gramática vira uma função.

**Estratégia de erro:** Reportar múltiplos erros com `synchronize()` nos pontos de sincronização (início de statement, `}`, `on`, `fn`).

**Prioridade de operadores (menor → maior):**
```
or → and → == != → < > <= >= → + - → * / % → not - (unário) → . () []
```

#### 1.4 Interpreter (`crl-interpreter`)

AST-walking interpreter. Adequado para Fases 1–5.

```rust
struct Environment {
    values: HashMap<String, Value>,
    parent: Option<Rc<RefCell<Environment>>>,  // escopo léxico encadeado
}

enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Function(FunctionValue),
    NativeFunction(Arc<dyn Fn(Vec<Value>) -> Result<Value, RuntimeError>>),
}
```

**Funções nativas obrigatórias:**
- `print(value)` — stdout
- `len(list_or_string)` — tamanho
- `type_of(value)` — nome do tipo como String
- `to_str(value)` — conversão para String

**Deliverables da Fase 1:**
- [ ] Lexer: todos os tokens reconhecidos com posição
- [ ] Parser: AST correto para todos os construtos da gramática v1
- [ ] Interpreter: let, fn, if, else, while, for, on start/stop
- [ ] Erros com linha/coluna precisas em todos os estágios
- [ ] CLI: `crl run arquivo.crl` e `crl check arquivo.crl`
- [ ] Testes unitários: cobertura > 90% no lexer, > 85% no parser
- [ ] 7 scripts de validação executando corretamente

---

### FASE 2 — Runtime Contínuo

**Duração estimada:** 3–6 semanas
**Objetivo:** O sistema é um processo vivo que reage a eventos continuamente.

**Meta verificável ao final:**
```
agent clock {
    on start {
        print("Sistema iniciado")
    }

    on timer(1000) {
        print("tick")
    }

    on stop {
        print("Encerrando")
    }
}
```
Deve imprimir "tick" a cada segundo indefinidamente até Ctrl+C.

**Componentes:**

#### 2.1 Event Bus

Sistema nervoso central. Todo evento passa por ele.

```rust
enum Event {
    Start,
    Stop,
    Timer { elapsed_ms: u64 },
    Message { from: AgentId, topic: String, payload: Value },
    Custom { name: String, data: Value },
}

trait EventBus: Send + Sync {
    async fn publish(&self, event: Event);
    fn subscribe(&self, agent_id: AgentId, pattern: EventPattern) -> SubscriptionId;
    fn unsubscribe(&self, id: SubscriptionId);
}
```

#### 2.2 Scheduler

Decide QUANDO e COMO os handlers executam.

- Garante que um agent não executa dois handlers simultaneamente (por padrão)
- Backpressure quando a fila está cheia
- Prioriza Stop e erros de sistema sobre eventos de usuário

#### 2.3 Dispatcher

Liga event bus ao scheduler: recebe eventos, encontra handlers, cria tasks.

**Deliverables da Fase 2:**
- [ ] Event bus com pub/sub por tipo
- [ ] Timer com precisão de ±10ms
- [ ] Scheduler sem race conditions (validado com testes de concorrência)
- [ ] Shutdown gracioso
- [ ] 1000 eventos/segundo sem degradação observável

---

### FASE 3 — Sistema de Capabilities

**Duração estimada:** 2–4 semanas
**Objetivo:** Agents só usam recursos declarados explicitamente.

**Meta verificável ao final:**
```
agent home {
    use lights
    on start {
        lights.enable()   -- OK
    }
}

agent visitor {
    on start {
        lights.enable()   -- ERRO: capability 'lights' não declarada
    }
}
```

**Componentes:**

#### 3.1 Trait de Capability

```rust
trait Capability: Send + Sync {
    fn name(&self) -> &'static str;
    fn methods(&self) -> &[MethodSpec];
    fn execute(&self, method: &str, args: Vec<Value>) -> Result<Value, CapabilityError>;
}
```

#### 3.2 Verificação em Análise Estática

O type checker analisa o AST do agent:
1. Toda chamada `X.method()` exige `X` em `use`
2. O método deve existir na capability
3. Tipos dos argumentos devem ser compatíveis (progressivo)

**Capabilities built-in (simuladas) da Fase 3:**
- `lights` — enable(), disable(), set_brightness(level: Int)
- `speaker` — say(text: String)
- `logger` — log(level: String, message: String)

**Deliverables da Fase 3:**
- [ ] Registry de capabilities
- [ ] Verificação estática de `use`
- [ ] 3 capabilities built-in (simuladas)
- [ ] Erro claro com posição quando capability não declarada

---

### FASE 4 — Concorrência Real e Efeitos

**Duração estimada:** 3–5 semanas
**Objetivo:** Múltiplos eventos simultâneos sem interferência, efeitos marcados.

**Meta verificável ao final:**
```
agent multi {
    use speaker
    use lights

    on timer(500) {
        do lights.toggle()
    }

    on timer(1000) {
        do speaker.say("Tick")
    }

    on message("alert") {
        do lights.enable()
        do speaker.say("Alerta recebido")
    }
}
```

**Componentes:**

#### 4.1 Keyword `do`

A partir desta fase, side effects exigem `do`. O compilador usa isso para:
- Verificar que o handler tem a capability necessária
- Gerar logs de auditoria automáticos
- Marcar pontos de paralelização potencial (futuro)

#### 4.2 Channels Tipados entre Agents

```
agent sensor {
    on timer(100) {
        let reading = sensor.read()
        send("monitor", "new_reading", reading)
    }
}

agent monitor {
    on message("new_reading") {
        if event.payload > 80 {
            do lights.alert()
        }
    }
}
```

#### 4.3 Task Kinds

```rust
enum TaskKind {
    Async,      // IO bound
    Blocking,   // CPU bound (thread pool separado)
    Periodic,   // timers e polling
}
```

**Deliverables da Fase 4:**
- [ ] Tasks assíncronas sem bloquear event loop
- [ ] Tasks bloqueantes em thread pool separado
- [ ] Channels tipados entre agents
- [ ] `do` validado pelo compilador
- [ ] Backpressure funcional

---

### FASE 5 — Contexto e Memória

**Duração estimada:** 3–6 semanas
**Objetivo:** Agents lembram, esquecem e consultam histórico.

**Meta verificável:**
```
agent assistant {
    on speech {
        let name = context.get("user_name")
        if name == null {
            context.set("user_name", event.speaker)
            do speaker.say("Olá! Como posso te chamar?")
        } else {
            do speaker.say("Olá de novo, " + name)
        }
    }
}
```

**Camadas de memória:**

| Camada | Duração | Implementação | Caso de uso |
|---|---|---|---|
| Working | Duração do handler | Variáveis locais | Computação temporária |
| Session | Enquanto o agent roda | `HashMap` em memória | Estado da conversa atual |
| Persistent | Indefinida | `sled` (embedded) | Preferências, nome do usuário |
| Episodic | Indefinida + timeline | `sled` com timestamps | "O que aconteceu hoje?" |
| Semantic | Indefinida + busca vetorial | Embeddings + índice vetorial | Memória por significado (Fase 8) |

**API do Context:**
```
context.get(key: String) -> Value?
context.set(key: String, value: Value)             -- session, não persiste
context.remember(key: String, value: Value)        -- persiste entre sessões
context.forget(key: String)                        -- remove da memória persistente
context.episode(description: String, data: Value)  -- registra evento episódico
context.consolidate()                              -- move session → persistent (Fase 9)
context.search(query: String) -> List<Value>       -- busca semântica (Fase 8)
```

**Deliverables da Fase 5:**
- [ ] Session memory isolada por agent
- [ ] Persistent memory (sled)
- [ ] Episodic memory com timestamps
- [ ] API completa (get, set, remember, forget, episode)
- [ ] Serialização de todos os tipos de Value
- [ ] TTL configurável por chave
- [ ] Contexto de um agent nunca acessível a outro agent

---

### FASE 6 — Percepção Multimodal

**Duração estimada:** 4–8 semanas
**Objetivo:** Agents percebem o mundo real.

**Meta verificável:**
```
agent listener {
    use microphone
    on speech {
        print("Ouvi: " + event.transcript)
    }
}
```

**Pipelines:**

```
Microfone → ring buffer → VAD → STT (Whisper) → SpeechEvent → Event Bus
Câmera   → frame buffer → detecção → ImageEvent → Event Bus
Sensor   → polling/interrupt → SensorReading → SensorEvent → Event Bus
```

**Backpressure obrigatório:** Se o event bus estiver saturado, frames e amostras
de áudio devem ser descartados com log — nunca deixar o pipeline bloquear o runtime.

**Deliverables da Fase 6:**
- [ ] Pipeline de áudio: mic → SpeechEvent
- [ ] STT integrado (Whisper via API ou local via candle)
- [ ] Pipeline de vídeo básico: câmera → ImageEvent
- [ ] Trait genérico de sensor
- [ ] Backpressure em todos os pipelines
- [ ] Configuração de dispositivo (câmera 0, microfone 1...)

---

### FASE 7 — Sistema de Plugins

**Duração estimada:** 3–5 semanas
**Objetivo:** Capabilities adicionadas sem recompilar o runtime.

**Mecanismo:** Plugins compilados para WebAssembly e carregados via `wasmtime`.

```rust
// Interface que todo plugin WASM deve implementar:
#[wasm_export] fn capability_name() -> &str;
#[wasm_export] fn capability_methods() -> Vec<MethodSpec>;
#[wasm_export] fn execute(method: &str, args_json: &str) -> &str;  // JSON in/out
```

**Garantias de isolamento:**
- Plugin defeituoso não derruba o runtime
- Plugin sem permissão não acessa filesystem ou rede
- Hot reload sem reiniciar o runtime

**Deliverables da Fase 7:**
- [ ] Carregamento de plugins WASM via wasmtime
- [ ] Sandbox: sem acesso a filesystem/rede sem permissão explícita
- [ ] Hot reload funcional
- [ ] Registry de plugins com versionamento
- [ ] SDK com template e documentação para desenvolver plugins
- [ ] 2 plugins de exemplo funcionando

---

### FASE 8 — Motor de IA

**Duração estimada:** 2–4 semanas
**Objetivo:** IA integrada como capability plugável.

**Meta verificável:**
```
agent assistant {
    use ai
    use speaker

    on speech {
        let response = ai.reason(event.transcript, context)
        do speaker.say(response)
    }
}
```

**Arquitetura:**

```
ai.reason() / ai.plan() / ai.embed() / ai.classify()
        │
        ▼
AI Capability (trait abstrato)
        │
   ┌────┴──────────────┐
   │                   │
OpenAI (API)       Candle (local — Mistral, LLaMA, Whisper)
```

**Funções:**
- `ai.reason(prompt, context)` → resposta textual
- `ai.plan(goal, context)` → lista de ações estruturadas
- `ai.embed(text)` → vetor de embedding (para context.search)
- `ai.classify(text, labels)` → classificação
- `ai.extract(text, schema)` → extração estruturada de dados

**Deliverables da Fase 8:**
- [ ] Trait abstrato de LLM com 2 providers (OpenAI + Candle)
- [ ] `ai.reason()` integrado ao event handler
- [ ] `ai.plan()` retornando lista de ações
- [ ] Embeddings + busca vetorial no context (integra com Fase 5)
- [ ] Rate limiting e retry para APIs externas
- [ ] Cache de respostas idênticas
- [ ] Runtime funciona 100% se ai não for chamado

---

### FASE 9 — Agentes Cognitivos

**Duração estimada:** indefinida (pesquisa e refinamento contínuos)
**Objetivo:** Agents com personalidade, memória episódica e adaptação comportamental.

**Meta conceitual:**
```
agent jarvis {
    personality {
        name          = "Jarvis"
        tone          = "professional"
        language      = "pt-BR"
        memory_style  = "episodic"
        priorities    = ["security", "user_comfort"]
    }

    on speech {
        let intent = ai.reason(event.transcript, context)
        let plan   = ai.plan(intent, context)
        for action in plan {
            do execute(action)
        }
    }

    on idle(300) {
        let summary = context.summarize_period("today")
        do speaker.say("Posso te ajudar com algo?")
    }
}
```

**Novos conceitos desta fase:**
- `personality {}` — parâmetros persistentes que contextualizam IA e filtram capabilities
- `on idle(seconds)` — handler ativado por ausência de eventos
- `context.consolidate()` — move memória de sessão para persistente com filtragem
- `context.summarize_period()` — sumariza episódios de um período
- Adaptação: parâmetros de personalidade ajustáveis por feedback observado

---

### FASE 10 — Integração Real: Rede, IoT e Acesso Remoto

**Duração estimada:** indefinida (integração contínua)
**Objetivo:** O sistema integrado com o mundo real — dispositivos, rede e usuários remotos.

**Sub-fase 10.1 — Gateway de Acesso Remoto:**

```
Celular / Web / API
        │
        │  HTTP POST /intent  (JWT autenticado)
        │  WebSocket          (bidirecional, baixa latência)
        ▼
Gateway (axum + tokio-tungstenite)
        │
        │  Autenticação JWT
        │  Verificação de capabilities remotas permitidas
        ▼
Event Bus do Runtime
        │
        ▼
Agent executa com contexto remoto
```

**Sub-fase 10.2 — Integração IoT via MQTT:**

```
agent home_iot {
    use mqtt
    use lights
    use thermostat

    on network("mqtt/sensor/temperature") {
        let temp = event.payload.value
        if temp > 28 {
            do thermostat.cool(22)
        }
    }

    on speech {
        let intent = ai.reason(event.transcript, context)
        if intent.action == "lights_off" {
            do lights.disable()
            do mqtt.publish("home/lights/status", "off")
        }
    }
}
```

**Dispositivos e protocolos planejados:**

| Categoria | Dispositivos | Protocolo |
|---|---|---|
| Smart home | Home Assistant, Matter, Zigbee | HTTP/REST, WebSocket |
| Microcontroladores | ESP32, Arduino | MQTT |
| SBCs | Raspberry Pi | MQTT, HTTP |
| Sensores genéricos | temperatura, presença, luminosidade | MQTT |
| Atuadores | relés, servos, LEDs | MQTT |
| Outros PCs | notificações, arquivos | WebSocket, HTTP |

**Capability de rede:**
```rust
trait NetworkCapability: Capability {
    async fn publish(&self, topic: &str, payload: Value) -> Result<(), CapError>;
    fn subscribe(&self, topic: &str) -> Receiver<NetworkEvent>;
    async fn request(&self, url: &str, method: &str, body: Option<Value>) -> Result<Value, CapError>;
}
```

**Deliverables da Fase 10:**
- [ ] Gateway HTTP/WebSocket com autenticação JWT
- [ ] Cliente MQTT integrado como capability
- [ ] Eventos de rede chegando ao event bus
- [ ] Capabilities remotas com subset restrito de permissões
- [ ] Agent responde a comandos do celular em < 200ms (latência local)
- [ ] Home Assistant integrado via REST capability
- [ ] ESP32 enviando sensor data via MQTT, agent reagindo

---

## 9. GLOSSÁRIO TÉCNICO

**ADR (Architecture Decision Record)** — Documento que registra uma decisão arquitetural com contexto, opções consideradas e raciocínio.

**Agent** — Unidade cognitiva do sistema. Tem capabilities, handlers e contexto próprio. Nunca compartilha estado com outros agents.

**AST (Abstract Syntax Tree)** — Representação em árvore do programa gerada pelo parser. Base de toda análise e execução.

**Capability** — Permissão declarada para acessar um recurso (hardware, API, rede, dispositivo).

**Context** — Memória do agent. Isolada por agent. Tem camadas (working, session, persistent, episodic, semantic).

**Dispatcher** — Componente que roteia eventos do bus para os handlers corretos.

**Effect / Efeito** — Operação com consequência fora do escopo puro (IO, mutação de estado, rede).

**Event** — Ocorrência tipada no sistema (SpeechEvent, TimerEvent, NetworkEvent, etc).

**Event Bus** — Canal central por onde todos os eventos trafegam antes de serem roteados.

**Event Sourcing** — Padrão onde o estado é derivado de um log imutável de eventos. Aplicado na memória episódica.

**Gateway** — Ponto de entrada autenticado para intenções remotas (celular, web, API).

**Handler** — Bloco `on evento {}` que reage a um evento específico.

**Intent (Intenção)** — Comando autenticado enviado por um cliente remoto. O runtime valida antes de executar.

**Lexer** — Converte texto em tokens. Primeira etapa do pipeline de compilação.

**MQTT** — Protocolo leve de pub/sub para IoT. Barramento natural entre o runtime e dispositivos.

**Parser** — Converte tokens em AST. Segunda etapa do pipeline.

**Plugin** — Capability carregada dinamicamente via WASM.

**Scheduler** — Gerencia quando e como tasks/handlers são executados.

**Span** — Par (start, end) de posições no source code. Presente em todo nó do AST.

**Token** — Unidade atômica da linguagem (keyword, literal, operador, etc).

**Value** — Tipo de dado em runtime: Int, Float, String, Bool, Null, List, Map, Function, NativeFunction.

---

## 10. RISCOS E MITIGAÇÕES

| Risco | Prob. | Impacto | Mitigação |
|---|---|---|---|
| Escopo infinito — projeto nunca termina | Alta | Alto | Deliverables verificáveis por fase. Não avançar sem completar a fase atual. |
| Gramática incompatível com o parser | Alta | Alto | Escrever gramática ANTES do parser. Testar com exemplos concretos. |
| Refatoração de fundação pós-lançamento | Média | Muito Alto | Multi-agent ready desde Dia 1 (HashMap, não valor único). |
| Race conditions | Alta | Alto | Rust borrow checker + testes de concorrência desde a Fase 2. |
| Tipagem rígida prematura bloqueia MVP | Média | Alto | Tipagem progressiva: dinâmica no MVP, estática cresce por fase. |
| Performance do AST-walking insuficiente | Média | Médio | Aceitável até Fase 6. Bytecode VM avaliado com métricas reais na Fase 6. |
| Plugins maliciosos/bugados | Alta | Alto | WASM sandbox obrigatório. Sem exceções. |
| Dependência de API externa (OpenAI) | Alta | Médio | Abstração de provider. Sempre ter fallback local (Candle). |
| Segurança do gateway remoto | Alta | Muito Alto | JWT + capabilities restritas para contexto remoto. Auditoria completa. |
| MQTT não escala para casos futuros | Baixa | Médio | Abstração de protocolo no crate `crl-network`. MQTT é o padrão, mas substituível. |
| Burnout por complexidade acumulada | Média | Muito Alto | Uma fase de cada vez. Celebrar cada deliverable. Revisão do roadmap a cada 3 fases. |

---

## 11. PRINCÍPIOS QUE NUNCA DEVEM SER VIOLADOS

1. **A IA não é o núcleo.** O runtime deve funcionar 100% sem qualquer IA conectada.

2. **Capabilities são explícitas.** Nenhum agent acessa recurso sem declarar `use X`.

3. **Agents não compartilham memória.** Toda comunicação é por channels ou eventos.

4. **Efeitos são visíveis.** Side effects devem ser marcados com `do` e auditáveis.

5. **Erros têm localização.** Todo erro indica arquivo, linha e coluna exatos.

6. **Uma fase por vez.** Não iniciar Fase N+1 antes dos deliverables da Fase N estarem completos.

7. **Testes antes de avançar.** Cobertura mínima: 90% no lexer, 85% no parser, 80% no runtime.

8. **A gramática é a fonte da verdade.** Parser sempre derivado da gramática EBNF. Gramática muda antes do parser.

9. **Arquitetura multi-agent desde o Dia 1.** Nunca assumir um único agent na estrutura interna.

10. **Intenções remotas são intenções, não código.** O gateway recebe intenções, nunca executa código CRL diretamente.

11. **Rust ownership é seu aliado.** Se o borrow checker reclama, o design está errado — não use `unsafe`.

12. **Hipóteses são hipóteses.** Antes de comprometer uma decisão técnica nova, implementar um spike de validação.

---

*Versão: 2.0.0 | Revisão: Engenharia Sênior — incorpora correções do context_sugerido_crl*
*Este documento é vivo. Atualize-o conforme o projeto evolui e decisões são validadas.*
