# CRL — Cognitive Runtime Language
## Mapa Geral do Projeto · Do Zero ao Jarvis
### Documento de Arquitetura e Roadmap Completo · Versão 3.0

---

> **Como usar este documento**
> Este é o mapa permanente do projeto. Ele descreve cada fase, cada decisão
> arquitetural, cada componente e cada risco. Consulte-o antes de iniciar
> qualquer fase nova. Atualize-o conforme decisões são validadas ou revisadas.
>
> **v3.0 — Revisão de auditoria técnica completa:**
> Dez lacunas identificadas e incorporadas ao design:
> (1) Padrões de coordenação entre agents
> (2) Prioridade de eventos por domínio de segurança
> (3) Modelo de falha e recuperação de agent (Supervisor Pattern)
> (4) Observabilidade nativa
> (5) Política de backpressure por tipo de evento
> (6) Modelo de concorrência intra-agent no contexto
> (7) Hot reload de agents CRL
> (8) Latência de IA com política explícita
> (9) Separação formal entre AgentDefinition e AgentInstance
> (10) Canais tipados entre agents

---

## ÍNDICE

1. Visão do Projeto
2. Três Blocos Arquiteturais
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

CRL (Cognitive Runtime Language) é uma linguagem de programação e runtime
criados do zero com um propósito único: ser a fundação computacional de
agentes cognitivos contínuos — sistemas que percebem o mundo, reagem a
eventos com prioridade correta, lembram de contexto, se recuperam de falhas,
integram inteligência artificial como camada plugável e se comunicam com
dispositivos e redes de forma segura e observável.

### O que NÃO é o CRL

- Não é um framework de IA (como LangChain ou AutoGen)
- Não é uma linguagem de uso geral (como Python ou Rust)
- Não é um assistente virtual pronto (como Alexa ou Google Assistant)
- Não é uma linguagem de scripting simples
- Não é um sistema de automação residencial (como Home Assistant)

### A metáfora central

```
IA            = o motor           (gera potência cognitiva — plugável, substituível)
Runtime CRL   = o chassi          (estrutura que segura tudo)
Scheduler     = a transmissão     (distribui o trabalho com prioridade)
Supervisor    = o mecânico        (monitora e recupera componentes com falha)
Memória       = o tanque          (armazena contexto e histórico)
Capabilities  = as rodas          (tocam o mundo real)
Observability = o painel          (mostra o que está acontecendo)
Sensores      = os olhos/ouvidos  (percepção contínua)
Atuadores     = as mãos           (ação no mundo)
Rede/IoT      = as estradas       (contexto externo e dispositivos conectados)
```

O motor não dirige o carro. O chassi dirige o motor.
O mecânico mantém o carro rodando. O painel informa o estado.

---

## 2. TRÊS BLOCOS ARQUITETURAIS

```
┌─────────────────────────────────────────────────────────────────┐
│  BLOCO 1 — LINGUAGEM                                            │
│  Gramática · Lexer · Parser · AST · Semântica · Tipos           │
│  → Define como o programador escreve agents e handlers          │
├─────────────────────────────────────────────────────────────────┤
│  BLOCO 2 — RUNTIME                                              │
│  Scheduler · Event Bus · Supervisor · Agents · Contexto         │
│  Capabilities · Observability · Backpressure                    │
│  → Define como o sistema executa, reage e se recupera           │
├─────────────────────────────────────────────────────────────────┤
│  BLOCO 3 — ECOSSISTEMA                                          │
│  Plugins · Rede · IoT · IA · Aprendizado · Hot Reload           │
│  → Define como o sistema se conecta ao mundo externo            │
└─────────────────────────────────────────────────────────────────┘
```

**Regra de dependência (nunca inverte):**
- Linguagem não depende de Runtime nem de Ecossistema
- Runtime depende da Linguagem (executa o código)
- Ecossistema depende do Runtime (capabilities são chamadas pelo runtime)

---

## 3. DECISÕES ARQUITETURAIS — FIRMES vs. HIPÓTESES

Classificações:
- **FIRME** — embasada em requisitos concretos, não deve ser revertida sem análise de impacto
- **HIPÓTESE ATUAL** — direção preferida, a ser validada antes de comprometer
- **DECISÃO DE FASE** — tomada em momento específico, não antes

---

### 3.1 Rust como linguagem de implementação

**Classificação: FIRME**

O sistema opera 24/7 integrado com hardware real. Falhas de memória, data
races e vazamentos não são aceitáveis. Rust oferece segurança de memória
sem GC, `async/await` nativo, e ecossistema WASM robusto — os três pilares
que o CRL precisa simultaneamente.

---

### 3.2 Arquitetura Event-Driven com Actor Model

**Classificação: FIRME**

Agents são naturalmente isolados: têm estado próprio, reagem a eventos,
comunicam por mensagens. Compartilhamento de memória entre agents é fonte
garantida de bugs em um sistema contínuo.

**Estrutura interna desde o Dia 1:**
```rust
struct Runtime {
    agents:              HashMap<AgentId, AgentInstance>,  // nunca valor único
    event_bus:           EventBus,
    supervisor:          Supervisor,                        // NOVO — v3.0
    context_store:       ContextStore,                      // isolado por AgentId
    capability_registry: CapabilityRegistry,
    scheduler:           Scheduler,
    observer:            Observer,                          // NOVO — v3.0
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

Um agent que controla luzes não deve poder acessar câmeras sem declaração
explícita. O princípio do menor privilégio não é opcional em um sistema
integrado com hardware real e acesso remoto.

```
agent home {
    use lights
    on start {
        lights.enable()   -- permitido: capability declarada
        camera.view()     -- ERRO compile-time: capability não declarada
    }
}
```

---

### 3.4 Tipagem: Progressiva em direção a Estática com Inferência

**Classificação: HIPÓTESE ATUAL → Decisão de Fase 3**

| Fase | Tipagem | Entrega |
|---|---|---|
| 1–2 | Dinâmica com anotações opcionais | MVP funcional |
| 3 | Verificação de capabilities em análise estática | Segurança de acesso |
| 4–5 | Tipos obrigatórios em assinaturas de função | Contratos claros |
| 6+ | Inferência básica para variáveis locais | Ergonomia |
| 7+ | Sistema de efeitos tipados | Auditabilidade formal |

**Invariante:** Erros de tipo sempre indicam linha, coluna e contexto exatos.

---

### 3.5 Sistema de Efeitos: Progressivo em 3 Camadas

**Classificação: FIRME quanto à direção, HIPÓTESE quanto à implementação**

**Camada 1 — Capabilities como guardas (Fases 1–3):**
```
agent home {
    use lights
    on start { lights.enable() }  -- runtime verifica capability
}
```

**Camada 2 — Efeitos marcados sintaticamente (Fases 4–6):**
```
on speech {
    let intent = ai.reason(context)
    do lights.enable()   -- "do" = side effect IO explícito
}
```

**Camada 3 — Sistema algébrico formal (Fases 7+):**
```
fn enable_lights() -> () ! [IO, RequiresCapability(Lights)] { ... }
```

**Regra:** Camada 3 NÃO deve ser implementada antes da Fase 7.

---

### 3.6 Multi-Agent: Arquitetura Pronta, Complexidade Gradual

**Classificação: FIRME quanto à arquitetura interna**

A estrutura interna usa `HashMap<AgentId, AgentInstance>` desde a Fase 1.

**Progressão:**
- Fase 1–2: Um único agent executa corretamente
- Fase 4: Dois agents se comunicam por channel tipado
- Fase 5+: N agents com contextos isolados e coordenados
- Fase 9: Agents cognitivos com coordenação dinâmica

---

### 3.7 Separação entre AgentDefinition e AgentInstance

**Classificação: FIRME — NOVO v3.0**

**O problema:**
O AST contém `AgentDecl` — a definição (blueprint) do agent. Mas o runtime
precisa distinguir explicitamente entre o blueprint e as instâncias em execução,
especialmente para suportar múltiplas instâncias do mesmo blueprint na Fase 9.

**Modelo:**
```rust
// Blueprint: o código CRL compilado — imutável, compartilhável
struct AgentDefinition {
    name:         String,
    capabilities: Vec<CapabilityDecl>,
    handlers:     Vec<HandlerDecl>,        // AST dos handlers
    functions:    Vec<FnDecl>,
    error_policy: ErrorPolicy,             // NOVO — como reagir a falhas
    event_policy: EventPolicy,             // NOVO — prioridades e backpressure
}

// Instância: um agent vivo em memória — estado próprio, contexto próprio
struct AgentInstance {
    id:         AgentId,
    definition: Arc<AgentDefinition>,      // compartilha o blueprint (imutável)
    context:    AgentContext,              // isolado por instância
    mailbox:    Receiver<AgentMessage>,    // canal de entrada de mensagens
    status:     AgentStatus,              // Running | Restarting | Stopped
    metrics:    AgentMetrics,             // NOVO — counters de observabilidade
}
```

**Por que `Arc<AgentDefinition>`:**
Múltiplas instâncias podem compartilhar o mesmo blueprint sem cópia.
Na Fase 9, `spawn agent("room_light", { room: "kitchen" })` cria uma nova
`AgentInstance` com o mesmo `AgentDefinition`, mas contexto isolado.

---

### 3.8 Prioridade de Eventos por Domínio

**Classificação: FIRME — NOVO v3.0**

**O problema:**
Um event bus com fila plana não distingue entre um tick de timer cosmético
e um sensor de gás disparando. Em um sistema de segurança, isso é inaceitável.

**Modelo de prioridade em 4 níveis:**

```rust
enum EventPriority {
    Critical = 0,   // segurança: sensor de gás, alarme, falha crítica
    High     = 1,   // usuário aguardando: speech, comando remoto
    Normal   = 2,   // operação padrão: timers funcionais, sensores regulares
    Low      = 3,   // cosmético: timers de UI, logs periódicos
}
```

**Prioridade padrão por tipo de evento:**

| Tipo de evento | Prioridade padrão | Justificativa |
|---|---|---|
| `on_error` do Supervisor | Critical | Recuperação de falha |
| `on sensor_change("*_alarm")` | Critical | Segurança física |
| `on speech` | High | Usuário esperando resposta |
| `on message(...)` | High | Comunicação inter-agent |
| `on network(...)` | High | Comando remoto autenticado |
| `on timer(N)` | Normal | Operação regular |
| `on sensor_change(...)` | Normal | Leitura de ambiente |
| `on image` | Low | Processamento de frame |
| `on idle(N)` | Low | Comportamento proativo |

**Override na linguagem:**
```
on sensor_change("gas_level") priority critical {
    do alarm.trigger()
}

on timer(5000) priority low {
    do lights.pulse()  -- efeito cosmético
}
```

**Implementação no Scheduler:**
O scheduler usa uma `BinaryHeap<PrioritizedTask>` ao invés de fila FIFO.
Eventos `Critical` nunca são descartados por backpressure — têm fila dedicada.

---

### 3.9 Modelo de Falha e Recuperação — Supervisor Pattern

**Classificação: FIRME — NOVO v3.0**

**O problema:**
Sem um modelo de recuperação, um handler que lança exceção tem três
comportamentos possíveis: mata o agent, mata o runtime inteiro, ou é
ignorado silenciosamente. Todos são errados para um sistema 24/7.

**Inspiração:** Erlang/OTP Supervisor Pattern — um dos designs mais
battle-tested para sistemas de alta disponibilidade.

**Modelo de supervisão no CRL:**

```rust
enum RestartStrategy {
    RestartHandler,     // re-executa o handler com o mesmo evento
    RestartAgent,       // reinicia o agent mantendo o contexto
    StopAgent,          // para o agent e notifica o Supervisor pai
    Escalate,           // propaga o erro para o Supervisor pai
}

struct ErrorPolicy {
    strategy:    RestartStrategy,
    max_retries: u32,              // 0 = sem limite
    backoff_ms:  u64,              // espera antes de reiniciar
    on_exhaust:  RestartStrategy,  // estratégia quando max_retries é atingido
}
```

**Sintaxe na linguagem:**
```
agent home {
    on_error {
        strategy    = "restart_handler"
        max_retries = 3
        backoff_ms  = 1000
        on_exhaust  = "stop_agent"
    }

    on speech {
        -- se este handler falhar: reinicia até 3x com 1s de espera
        -- se falhar 3x: para o agent e registra no Observer
        let response = ai.reason(context)
        do speaker.say(response)
    }
}
```

**Hierarquia de supervisão:**
```
Runtime Supervisor (raiz — sempre ativo)
    │
    ├── Agent "home"       (RestartAgent, max_retries: 5)
    │       └── Handler "on_speech"  (RestartHandler, max_retries: 3)
    │
    ├── Agent "security"   (StopAgent — falha implica parada segura)
    │
    └── Agent "coordinator" (Escalate — propaga para Runtime Supervisor)
```

**Garantias do Supervisor:**
- O Runtime Supervisor nunca morre — é a última linha de defesa
- Um agent com falha não impacta outros agents
- Toda falha é registrada no Observer antes de qualquer ação de recuperação
- O contexto do agent é preservado entre reinicializações de handler

---

### 3.10 Observabilidade Nativa

**Classificação: FIRME — NOVO v3.0**

**O problema:**
Um sistema cognitivo que roda 24/7 sem observabilidade é uma caixa-preta.
Quando algo dá errado às 3am, não há como investigar.

**Modelo:**
O runtime emite eventos de observabilidade automaticamente, sem que o
programador precise instrumentar. O `Observer` é um subscriber especial
do Event Bus que não interfere no fluxo normal.

```rust
enum ObservabilityEvent {
    HandlerStarted  { agent: AgentId, handler: String, event_id: Uuid },
    HandlerFinished { agent: AgentId, handler: String, duration_ms: u64 },
    HandlerFailed   { agent: AgentId, handler: String, error: String, span: Span },
    CapabilityCall  { agent: AgentId, cap: String, method: String, result: CallResult },
    AgentRestarted  { agent: AgentId, reason: String, attempt: u32 },
    AgentStopped    { agent: AgentId, reason: String },
    ContextWrite    { agent: AgentId, key: String, layer: MemoryLayer },
    MessageSent     { from: AgentId, to: AgentId, topic: String },
    BackpressureDrop{ agent: AgentId, event_type: String, queue_size: usize },
}
```

**Backends de observabilidade (plugáveis):**
```rust
trait ObserverBackend: Send + Sync {
    fn record(&self, event: ObservabilityEvent);
}

// Implementações built-in:
struct LogObserver;         // stdout/stderr — Fase 2
struct FileObserver;        // arquivo rotativo — Fase 5
struct MetricsObserver;     // counters/histogramas — Fase 5
struct TracingObserver;     // distributed tracing — Fase 8+
```

**API de health check (Fase 2):**
```
GET /health
→ { status: "ok", agents: { "home": "running", "security": "running" }, uptime_s: 3600 }
```

**O que o programador NÃO precisa fazer:**
- Instrumentar handlers manualmente
- Adicionar logs de erro
- Medir latência de chamadas de capability
- Registrar reinicializações

O runtime faz tudo isso automaticamente.

---

### 3.11 Política de Backpressure por Tipo de Evento

**Classificação: FIRME quanto ao modelo, DECISÃO DE FASE quanto à implementação**
**NOVO v3.0**

**O problema:**
"Backpressure" sem política definida é uma abstração incompleta. A política
errada para o tipo errado de evento causa perda silenciosa de dados críticos
ou acúmulo de dados irrelevantes.

**Políticas disponíveis:**

```rust
enum BackpressurePolicy {
    DropOldest,         // descarta o evento mais antigo da fila
    DropNewest,         // descarta o evento que acabou de chegar
    ErrorToAgent,       // notifica o agent do overflow via evento especial
    NeverDrop,          // fila cresce indefinidamente (apenas para Critical)
}
```

**Política padrão por tipo:**

| Tipo de evento | Política padrão | Justificativa |
|---|---|---|
| `Critical` | `NeverDrop` | Segurança não pode ser descartada |
| `on speech` | `DropOldest` | Fala antiga é inútil se nova chegou |
| `on image` | `DropOldest` | Frame velho é inútil — alta taxa |
| `on timer(N)` | `DropNewest` | Melhor pular um tick do que perder histórico |
| `on message(...)` | `ErrorToAgent` | Agent deve saber que perdeu mensagem |
| `on network(...)` | `ErrorToAgent` | Comando remoto não pode ser descartado silenciosamente |
| `on sensor_change` | `DropOldest` | Leitura recente substitui a antiga |

**Configuração por agent:**
```
agent home {
    event_policy {
        on_overflow      = "drop_oldest"     -- padrão do agent
        critical_queue   = 100              -- tamanho da fila crítica
        normal_queue     = 50
        low_queue        = 20
    }

    on image {
        backpressure = "drop_oldest"        -- override por handler
        max_queue    = 5                    -- apenas 5 frames enfileirados
    }
}
```

---

### 3.12 Concorrência Intra-Agent no Contexto

**Classificação: FIRME — NOVO v3.0**

**O problema:**
Na Fase 4, múltiplos timers de um mesmo agent podem disparar simultaneamente.
Se dois handlers escrevem no contexto ao mesmo tempo, o resultado é indefinido.

**Decisão: Handlers de um mesmo agent são sequenciais por padrão.**

Este é o modelo do Actor Model puro (Erlang): um actor processa uma mensagem
de cada vez. A mailbox garante a ordem. Isso elimina a classe inteira de
bugs de concorrência intra-agent sem necessidade de locks.

```
Mailbox do Agent "home":
[on_timer_500ms] → executa → [on_timer_1000ms] → executa → [on_speech] → executa
                 ← sem sobreposição por padrão →
```

**Override para handlers sem side effects (Fase 4+):**
```
agent home {
    on image concurrent {    -- marca explicitamente como permitido concorrente
        -- leitura apenas, sem escrita no contexto
        let desc = ai.describe(event.frame)
        do display.show(desc)
    }
}
```

**Regra:** `concurrent` só é permitido se o handler não escreve no contexto.
O compilador verifica isso na Fase 6 (análise de efeitos).

**Contexto com commit atômico (Fase 5):**
Mesmo em handlers sequenciais, escritas no contexto são atômicas:
```
context.set("a", 1)   ─┐
context.set("b", 2)    ├── commit atômico ao final do handler
context.set("c", 3)   ─┘
-- se o handler falhar, nenhuma das três escritas é aplicada
```

---

### 3.13 Padrões de Coordenação entre Agents

**Classificação: FIRME quanto ao modelo, DECISÃO DE FASE quanto à implementação**
**NOVO v3.0**

**O problema identificado:**
O Event Bus roteia eventos para agents inscritos, mas não havia modelo
para tarefas compostas que exigem sequenciamento entre múltiplos agents.

**A resposta correta para o CRL:**
Não existe um "super-agente central" fixo na arquitetura. Isso criaria:
- Ponto único de falha
- Gargalo de performance
- Violação do princípio de isolamento

Em vez disso, o CRL define três padrões de colaboração explícitos:

**Padrão 1 — Fan-out (Fase 2):**
O Event Bus roteia um evento para todos os agents inscritos.
Cada agent reage independentemente. Sem coordenação necessária.
```
SpeechEvent →  agent "home"      (controla luzes)
            →  agent "security"  (verifica identidade)
            →  agent "assistant" (gera resposta)
```

**Padrão 2 — Pipeline (Fase 4):**
Um agent processa e envia o resultado para o próximo via `send()`.
Sequenciamento explícito, sem acoplamento estrutural.
```
agent sensor {
    on timer(100) {
        let reading = sensor.read()
        send("processor", "raw_data", reading)
    }
}
agent processor {
    on message("raw_data") {
        let filtered = apply_filter(event.payload)
        send("display", "filtered_data", filtered)
    }
}
```

**Padrão 3 — Coordenador Dinâmico (Fase 9):**
Para tarefas compostas sequenciais, um agent de coordenação é ativado
dinamicamente. Ele é apenas mais um agent — não é privilegiado, tem
suas próprias capabilities declaradas, e pode falhar sem derrubar o sistema.
```
agent coordinator {
    use lights
    use thermostat
    use display

    on message("prepare_room") {
        -- sequência explícita para tarefa composta
        do lights.set_brightness(80)
        do thermostat.set_target(21)
        send("media_agent", "mute_alerts")
        send("display_agent", "presentation_mode")
        context.episode("room_prepared", event.payload)
    }
}
```

**O que diferencia este padrão de um "super-agente":**
- Não tem acesso global — apenas às capabilities declaradas com `use`
- Pode ser reiniciado pelo Supervisor como qualquer outro agent
- Não é instanciado no startup — é chamado quando necessário
- Outros agents funcionam normalmente se ele estiver indisponível

---

### 3.14 Canais Tipados entre Agents

**Classificação: FIRME — NOVO v3.0**

**O problema:**
`send("monitor", "topic", payload)` onde `payload` é `Value` genérico
é uma fonte garantida de bugs em sistemas distribuídos. Se o sender
mudar o tipo do payload, o receiver quebra silenciosamente.

**Modelo de canais tipados:**

Na Fase 4, channels entre agents são declarados com tipo explícito:

```
-- declaração de canal tipado (no agent sender)
agent sensor {
    channel temperature_readings: Float   -- declara o canal e seu tipo

    on timer(100) {
        let temp = thermometer.read()
        send("monitor").temperature_readings(temp)  -- tipado: só aceita Float
    }
}

-- o receiver declara que escuta aquele canal
agent monitor {
    on message(sensor.temperature_readings) {
        -- event.payload é Float garantido pelo runtime
        if event.payload > 28.0 {
            do thermostat.cool()
        }
    }
}
```

**Verificação em tempo de análise:**
- O tipo declarado no canal é verificado em compile-time
- Incompatibilidade de tipo gera erro com linha e coluna
- Canais sem declaração de tipo são permitidos na Fase 4 (retrocompatível),
  mas geram warning; tornam-se obrigatórios na Fase 6

---

### 3.15 Latência de IA: Política Explícita

**Classificação: FIRME quanto ao modelo — NOVO v3.0**

**O problema:**
`ai.reason()` pode levar entre 200ms e 8 segundos dependendo do modelo,
tamanho da resposta e estado da rede. Durante esse tempo, o handler está
suspenso. Outros eventos do mesmo agent são enfileirados na mailbox.

**Sem política explícita, os problemas são:**
- Usuário fala de novo enquanto o LLM processa → o segundo evento espera indefinidamente
- API externa cai → o handler fica suspenso para sempre
- Resposta muito longa → timeout no lado do usuário

**Política de IA adotada:**

```rust
struct AiCallPolicy {
    timeout_ms:       u64,    // padrão: 10_000 (10s)
    max_queue_during: usize,  // máximo de eventos enfileirados enquanto IA processa
    on_timeout:       AiTimeoutBehavior,
    on_queue_full:    BackpressurePolicy,
    cache_identical:  bool,   // cache de respostas idênticas
}

enum AiTimeoutBehavior {
    ReturnError,        // propaga erro para o handler
    ReturnFallback,     // retorna valor padrão configurado
    CancelAndContinue,  // cancela a chamada, continua sem resposta
}
```

**Configuração por agent:**
```
agent assistant {
    use ai
    use speaker

    ai_policy {
        timeout_ms       = 8000
        max_queue_during = 3       -- descarta eventos se mais de 3 esperando
        on_timeout       = "return_fallback"
        fallback_response= "Desculpe, não consegui processar isso agora."
        cache_identical  = true
    }

    on speech {
        let response = ai.reason(event.transcript, context)
        do speaker.say(response)
    }
}
```

**Implementação no runtime:**
A chamada `ai.reason()` é uma `async fn` executada como task separada.
O handler aguarda o resultado com timeout via `tokio::time::timeout`.
Se o timeout ocorrer, aplica `AiTimeoutBehavior` e continua normalmente.

---

### 3.16 Hot Reload de Agents CRL

**Classificação: HIPÓTESE ATUAL — entra na Fase 7**

**O problema:**
Em um sistema 24/7, atualizar o código de um agent exige parar e reiniciar
o runtime — o que implica perda de contexto e de continuidade.

**Modelo de hot reload:**

```
Ciclo de atualização de um agent:
1. Novo código CRL compilado para nova AgentDefinition
2. Runtime sinaliza ao AgentInstance: "prepare para atualizar"
3. Handler em execução termina normalmente
4. Contexto do agent é serializado (snapshot)
5. AgentInstance antiga é parada
6. Nova AgentInstance é criada com nova Definition + contexto restaurado
7. Runtime sinaliza: "agent atualizado"
```

**O que é preservado:** contexto (session + persistent), mailbox pendente
**O que é reiniciado:** estado interno do interpreter, handlers registrados
**O que pode ser incompatível:** mudanças de schema no contexto (versionamento necessário)

**Versionamento de contexto:**
```
agent home v2 {
    context_migration {
        from_version = 1
        migrate = |ctx| {
            -- converte schema antigo para novo
            let old_pref = ctx.get("preference")
            ctx.set("user_preference", old_pref)
            ctx.forget("preference")
        }
    }
    -- restante do agent
}
```

---

### 3.17 Estratégia de Execução: AST-Walking → Bytecode VM

**Classificação: DECISÃO DE FASE**

**Fase 1–5:** AST-walking interpreter. Mais simples de implementar, debugar e estender.

**Fase 6+:** Avaliar com métricas reais. Critério concreto: se o tempo de eval
de um handler for > 1ms em média para scripts simples, a VM deve ser considerada.

---

### 3.18 Acesso Remoto: Celular, Web e APIs

**Classificação: FIRME quanto ao modelo, DECISÃO DE FASE quanto à implementação**

O celular não executa código CRL. Ele envia **intenções autenticadas**.
O runtime valida, autoriza e executa. Intenções remotas têm capabilities
restritas (subset do que o agent local tem).

```
Celular / Web → Intenção assinada (JWT) → Gateway
                                              │
                                        Autenticação
                                              │
                                        Autorização (capabilities subset)
                                              │
                                        Event Bus (prioridade: High)
                                              │
                                        Agent executa
                                              │
                                        Observer registra
                                              │
                                        Resposta ao cliente
```

---

### 3.19 Integração com Dispositivos IoT

**Classificação: FIRME quanto ao modelo, DECISÃO DE FASE quanto à implementação**

Dispositivos IoT são capabilities de rede. O runtime não distingue entre
microfone USB e sensor ESP32 via MQTT — ambos seguem o mesmo contrato de capability.

---

### 3.20 Aprendizado e Adaptação Comportamental

**Classificação: HIPÓTESE ATUAL — entra na Fase 9**

Adaptação é modelada como ajuste de parâmetros persistentes baseado em
histórico observável. Sem consciência emergente, sem fine-tuning de modelos.

| Mecanismo | Implementação | Exemplo |
|---|---|---|
| Memória consolidável | Episódico → persistente | Preferências do usuário |
| Esquecimento seletivo | TTL + relevância | Limpar conversas antigas |
| Ajuste de tom | Parâmetros de personalidade mutáveis | Mais formal após feedback |
| Formação de hábitos | Padrões de evento registrados | Rotinas matinais |
| Melhoria de estratégias | Log de sucesso/falha de planos | Evitar ações que falharam |

---

### 3.21 Personalidade como Conjunto de Parâmetros Persistentes

**Classificação: HIPÓTESE ATUAL — entra na Fase 9**

```
personality {
    name           = "Jarvis"
    tone           = "professional"
    language       = "pt-BR"
    response_style = "concise"
    memory_style   = "episodic"
    boundaries     = ["no_financial_advice", "no_medical_diagnosis"]
    priorities     = ["security", "user_comfort", "energy_efficiency"]
}
```

---

## 4. MODELO MENTAL DO SISTEMA

### Fluxo de execução completo (v3.0)

```
Mundo Físico / Digital / Rede
        │
        │  (áudio, vídeo, sensor, timer, API, mensagem, comando remoto)
        ▼
┌────────────────────────┐
│  CAMADA DE ENTRADA     │   Captura de sinais + intenções remotas autenticadas
│  (Fases 2, 6, 10)      │   Drivers locais + Gateway JWT
└─────────┬──────────────┘
          │  raw signal / authenticated intent
          ▼
┌────────────────────────┐
│  EVENT SYSTEM          │   Converte sinais em Eventos tipados com prioridade
│  (Fase 2)              │   SpeechEvent(High), TimerEvent(Normal), SensorEvent(*)
└─────────┬──────────────┘
          │  typed Event com EventPriority
          ▼
┌────────────────────────┐
│  EVENT BUS             │   Roteia eventos por prioridade + subscriptions
│  (Fase 2)              │   BinaryHeap internamente — Critical nunca descartado
└─────────┬──────────────┘
          │  dispatched Event (priorizado)
          ▼
┌────────────────────────┐
│  BACKPRESSURE LAYER    │   Aplica política por tipo de evento e fila do agent
│  (Fase 2)              │   DropOldest | DropNewest | ErrorToAgent | NeverDrop
└─────────┬──────────────┘
          │  evento aceito ou descartado com log
          ▼
┌────────────────────────┐
│  SCHEDULER             │   Agenda task no AgentInstance correto
│  (Fase 2)              │   Garante sequencialidade intra-agent por padrão
└─────────┬──────────────┘
          │  scheduled Task
          ▼
┌────────────────────────┐
│  AGENT RUNTIME         │   Executa o handler com contexto injetado
│  (Fase 1+)             │   AST-walking interpreter → bytecode VM (Fase 6+)
└─────────┬──────────────┘
          │  falha?
          ▼
┌────────────────────────┐
│  SUPERVISOR            │   Aplica ErrorPolicy: restart | stop | escalate
│  (Fase 2)              │   Registra no Observer antes de agir
└─────────┬──────────────┘
          │  context read/write (atômico), capability calls
          ▼
┌────────────────────────┐
│  CONTEXT ENGINE        │   Working → Session → Persistent → Episodic → Semantic
│  (Fase 5)              │   Isolado por AgentInstance, commit atômico por handler
└─────────┬──────────────┘
          │  AI calls quando necessário (com timeout e fallback)
          ▼
┌────────────────────────┐
│  COGNITIVE LAYER       │   LLMs, embeddings, reasoning, planning
│  (Fase 8)              │   AiCallPolicy: timeout, cache, fallback
└─────────┬──────────────┘
          │  actions / effects
          ▼
┌────────────────────────┐
│  CAPABILITY SYSTEM     │   Verifica permissão antes de executar ação
│  (Fase 3)              │   lights, camera, speaker, MQTT, HTTP, IoT...
└─────────┬──────────────┘
          │  emite ObservabilityEvent em cada passo
          ▼
┌────────────────────────┐
│  OBSERVER              │   Registra métricas, traces, health, logs
│  (Fase 2 básico)       │   Backend plugável: Log | File | Metrics | Tracing
└─────────┬──────────────┘
          │
          ▼
Mundo Físico / Digital / Rede  (luzes acendem, resposta é dada, dispositivo é controlado)
```

---

## 5. PILHA TECNOLÓGICA

### Linguagem de implementação: Rust

| Atributo Rust | Benefício para o CRL |
|---|---|
| Ownership + Borrow Checker | Elimina data races entre agents e no contexto |
| Zero-cost abstractions | Runtime com performance de C |
| `async/await` nativo | Actor model leve sobre thread pool |
| `tokio::sync::mpsc` | Mailboxes tipadas entre agents |
| Ecossistema de parsers | `logos` para lexer |
| Tokio | Runtime assíncrono para scheduler |
| `BinaryHeap` | Fila de prioridade para eventos |
| WASM support | Isolamento de plugins |
| `serde` | Serialização de contexto, snapshot para hot reload |
| `tracing` | Observabilidade nativa |

### Dependências por fase

```toml
# Fase 1 — Núcleo da linguagem
logos       = "0.14"
thiserror   = "1.0"
miette      = { version = "5.10", features = ["fancy"] }

# Fase 2 — Runtime contínuo + Supervisor + Observer básico
tokio       = { version = "1", features = ["full"] }
tokio-util  = "0.7"
async-trait = "0.1"
uuid        = { version = "1", features = ["v4"] }   # IDs de eventos e agents
tracing     = "0.1"                                   # observabilidade
tracing-subscriber = "0.3"

# Fase 3 — Capabilities
# Sem dependência nova — registry em Rust puro

# Fase 4 — Concorrência e channels tipados
# Tokio channels já disponíveis

# Fase 5 — Contexto e memória
serde       = { version = "1", features = ["derive"] }
serde_json  = "1"
sled        = "0.34"          # embedded DB puro Rust

# Fase 6 — Percepção
cpal        = "0.15"          # áudio cross-platform
nokhwa      = "0.10"          # câmera

# Fase 7 — Plugins + Hot Reload
wasmtime    = "18"

# Fase 8 — IA com política de timeout
async-openai  = "0.23"
candle-core   = "0.6"
tokio         = { features = ["time"] }  # já incluso — timeout via tokio::time

# Fase 9/10 — Rede e IoT
axum              = "0.7"
tokio-tungstenite = "0.21"
rumqttc           = "0.24"
jsonwebtoken      = "9"
```

---

## 6. ESTRUTURA DE DIRETÓRIOS FINAL

```
crl/
│
├── Cargo.toml                      # Workspace root
├── Cargo.lock
├── README.md
│
├── docs/
│   ├── grammar.ebnf                # Gramática formal
│   ├── decisions/                  # ADRs
│   │   ├── adr-001-rust.md
│   │   ├── adr-002-actor-model.md
│   │   ├── adr-003-typing-strategy.md
│   │   ├── adr-004-supervisor-pattern.md     # NOVO v3.0
│   │   ├── adr-005-event-priority.md         # NOVO v3.0
│   │   ├── adr-006-backpressure-policy.md    # NOVO v3.0
│   │   ├── adr-007-intra-agent-sequential.md # NOVO v3.0
│   │   └── adr-008-typed-channels.md        # NOVO v3.0
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
│   │       ├── node.rs             # Nós com Span em todos + ErrorPolicy + EventPolicy
│   │       └── types.rs
│   │
│   ├── crl-parser/                 # FASE 1
│   │   └── src/
│   │       ├── lib.rs
│   │       └── parser.rs
│   │
│   ├── crl-typechecker/            # FASE 1 mínimo → FASE 3 capabilities → FASE 6 efeitos
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── checker.rs
│   │       └── env.rs
│   │
│   ├── crl-interpreter/            # FASE 1
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── eval.rs
│   │       ├── env.rs
│   │       └── value.rs
│   │
│   ├── crl-runtime/                # FASE 2
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── event_bus.rs        # BinaryHeap com EventPriority
│   │       ├── scheduler.rs        # Garante sequencialidade intra-agent
│   │       ├── dispatcher.rs
│   │       ├── supervisor.rs       # NOVO v3.0 — Supervisor Pattern
│   │       ├── observer.rs         # NOVO v3.0 — Observabilidade nativa
│   │       ├── backpressure.rs     # NOVO v3.0 — Políticas por tipo
│   │       ├── agent_definition.rs # NOVO v3.0 — Blueprint imutável
│   │       └── agent_instance.rs   # NOVO v3.0 — Instância com contexto e métricas
│   │
│   ├── crl-capabilities/           # FASE 3
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── registry.rs
│   │       ├── checker.rs
│   │       └── builtin/
│   │           ├── lights.rs
│   │           ├── microphone.rs
│   │           ├── camera.rs
│   │           └── logger.rs
│   │
│   ├── crl-context/                # FASE 5
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── working.rs
│   │       ├── session.rs
│   │       ├── persistent.rs
│   │       ├── episodic.rs
│   │       ├── snapshot.rs         # NOVO v3.0 — Serialização para hot reload
│   │       └── context.rs          # commit atômico por handler
│   │
│   ├── crl-perception/             # FASE 6
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── audio.rs
│   │       ├── video.rs
│   │       └── sensor.rs
│   │
│   ├── crl-plugins/                # FASE 7
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── loader.rs
│   │       ├── registry.rs
│   │       ├── sandbox.rs
│   │       └── hot_reload.rs       # NOVO v3.0 — Hot reload de agents e plugins
│   │
│   ├── crl-cognitive/              # FASE 8
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── llm.rs
│   │       ├── policy.rs           # NOVO v3.0 — AiCallPolicy: timeout, fallback, cache
│   │       ├── embeddings.rs
│   │       └── planner.rs
│   │
│   ├── crl-network/                # FASE 10
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── gateway.rs
│   │       ├── mqtt.rs
│   │       ├── auth.rs
│   │       └── intent.rs
│   │
│   └── crl-cli/                    # TODAS AS FASES
│       └── src/
│           ├── main.rs
│           ├── run.rs
│           ├── check.rs
│           └── repl.rs
│
└── examples/
    ├── phase1/hello.crl
    ├── phase2/event_loop.crl
    ├── phase2/supervisor_demo.crl      # NOVO v3.0
    ├── phase4/typed_channels.crl       # NOVO v3.0
    ├── phase4/coordinator_pattern.crl  # NOVO v3.0
    ├── phase5/memory_agent.crl
    └── phase10/home_agent.crl
```

---

## 7. GRAMÁTICA FORMAL DA LINGUAGEM (EBNF)

Cada construção indica em qual fase é implementada.

```ebnf
(* ============================================================ *)
(* CRL — Cognitive Runtime Language Grammar v3                   *)
(* ============================================================ *)

program         ::= statement* EOF

statement       ::= agent_decl | fn_decl | let_stmt | expr_stmt

(* ---- AGENT ---- *)
agent_decl      ::= "agent" IDENTIFIER ("v" INTEGER)? "{"  (* versão opcional Fase 7 *)
                        capability_decl*
                        personality_decl?
                        error_policy_decl?                  (* NOVO v3.0 — Fase 2 *)
                        event_policy_decl?                  (* NOVO v3.0 — Fase 2 *)
                        ai_policy_decl?                     (* NOVO v3.0 — Fase 8 *)
                        context_migration?                  (* NOVO v3.0 — Fase 7 *)
                        handler_decl*
                        channel_decl*                       (* NOVO v3.0 — Fase 4 *)
                        fn_decl*
                    "}"

capability_decl ::= "use" IDENTIFIER NEWLINE
                  | "use" "plugin" "(" STRING ")" NEWLINE   (* Fase 7 *)

(* ---- POLÍTICA DE ERRO (Supervisor Pattern) — Fase 2 ---- *)
error_policy_decl ::= "on_error" "{"
                        ("strategy"    "=" STRING NEWLINE)?  (* restart_handler | restart_agent | stop_agent | escalate *)
                        ("max_retries" "=" INTEGER NEWLINE)?
                        ("backoff_ms"  "=" INTEGER NEWLINE)?
                        ("on_exhaust"  "=" STRING NEWLINE)?
                      "}"

(* ---- POLÍTICA DE EVENTOS — Fase 2 ---- *)
event_policy_decl ::= "event_policy" "{"
                        ("on_overflow"    "=" STRING NEWLINE)?   (* drop_oldest | drop_newest | error | never_drop *)
                        ("critical_queue" "=" INTEGER NEWLINE)?
                        ("normal_queue"   "=" INTEGER NEWLINE)?
                        ("low_queue"      "=" INTEGER NEWLINE)?
                      "}"

(* ---- POLÍTICA DE IA — Fase 8 ---- *)
ai_policy_decl ::= "ai_policy" "{"
                     ("timeout_ms"        "=" INTEGER NEWLINE)?
                     ("max_queue_during"  "=" INTEGER NEWLINE)?
                     ("on_timeout"        "=" STRING NEWLINE)?
                     ("fallback_response" "=" STRING NEWLINE)?
                     ("cache_identical"   "=" BOOL NEWLINE)?
                   "}"

(* ---- MIGRAÇÃO DE CONTEXTO — Fase 7 ---- *)
context_migration ::= "context_migration" "{"
                        "from_version" "=" INTEGER NEWLINE
                        "migrate" "=" lambda_expr NEWLINE
                      "}"

(* ---- CANAIS TIPADOS — Fase 4 ---- *)
channel_decl    ::= "channel" IDENTIFIER ":" type_expr NEWLINE

(* ---- HANDLERS DE EVENTO ---- *)
handler_decl    ::= "on" event_pattern ("priority" priority_level)? ("concurrent")? block

event_pattern   ::= "start"
                  | "stop"
                  | "timer" "(" expr ")"
                  | "message" "(" channel_ref ")"            (* tipado: Fase 4 *)
                  | "speech"
                  | "image"
                  | "sensor_change" "(" IDENTIFIER ")"
                  | "network" "(" IDENTIFIER ")"
                  | "idle" "(" expr ")"
                  | IDENTIFIER

channel_ref     ::= IDENTIFIER                              (* canal local *)
                  | IDENTIFIER "." IDENTIFIER               (* canal de outro agent: agent.canal *)

priority_level  ::= "critical" | "high" | "normal" | "low"

(* ---- FUNÇÕES ---- *)
fn_decl         ::= "fn" IDENTIFIER "(" param_list? ")" ("->" type_expr)? block

param_list      ::= param ("," param)*
param           ::= IDENTIFIER (":" type_expr)?

(* ---- BLOCOS E STATEMENTS ---- *)
block           ::= "{" stmt_inner* "}"

stmt_inner      ::= let_stmt | assign_stmt | if_stmt | while_stmt
                  | for_stmt | return_stmt | effect_stmt | send_stmt | expr_stmt

let_stmt        ::= "let" IDENTIFIER (":" type_expr)? "=" expr NEWLINE
assign_stmt     ::= IDENTIFIER "=" expr NEWLINE
return_stmt     ::= "return" expr? NEWLINE
effect_stmt     ::= "do" expr NEWLINE
send_stmt       ::= "send" "(" STRING ")" "." IDENTIFIER "(" expr ")" NEWLINE  (* Fase 4, tipado *)
                  | "send" "(" STRING "," STRING "," expr ")" NEWLINE          (* Fase 4, legado *)
expr_stmt       ::= expr NEWLINE

if_stmt         ::= "if" expr block ("else" (if_stmt | block))?
while_stmt      ::= "while" expr block
for_stmt        ::= "for" IDENTIFIER "in" expr block

(* ---- EXPRESSÕES (ordem = precedência crescente) ---- *)
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

primary_expr    ::= literal | IDENTIFIER | "(" expr ")" | lambda_expr

lambda_expr     ::= "|" param_list? "|" (block | expr)     (* Fase 4 *)

literal         ::= INTEGER | FLOAT | STRING | BOOL | "null" | list_literal | map_literal
list_literal    ::= "[" (expr ("," expr)*)? "]"
map_literal     ::= "{" (map_entry ("," map_entry)*)? "}"
map_entry       ::= STRING ":" expr

(* ---- PERSONALIDADE — Fase 9 ---- *)
personality_decl ::= "personality" "{" personality_entry* "}"
personality_entry ::= IDENTIFIER "=" literal NEWLINE

(* ---- TIPOS ---- *)
type_expr       ::= "Int" | "Float" | "String" | "Bool" | "Null"
                  | "List" "<" type_expr ">"
                  | "Map" "<" type_expr "," type_expr ">"
                  | IDENTIFIER
                  | type_expr "?"                            (* Fase 3 *)
                  | type_expr "!" "[" effect_list "]"       (* Fase 7 *)

effect_list     ::= IDENTIFIER ("," IDENTIFIER)*

(* ---- TOKENS ---- *)
IDENTIFIER      ::= [a-zA-Z_][a-zA-Z0-9_]*
INTEGER         ::= [0-9]+
FLOAT           ::= [0-9]+ "." [0-9]+
STRING          ::= '"' [^"]* '"'
BOOL            ::= "true" | "false"
NEWLINE         ::= "\n" | "\r\n"
COMMENT_SINGLE  ::= "--" [^\n]* NEWLINE
COMMENT_MULTI   ::= "/:" ([^:] | ":" [^/])* ":/"

(* ---- PALAVRAS RESERVADAS ---- *)
(* agent, use, plugin, on, fn, let, return, if, else, while, for, in,
   do, send, true, false, null, and, or, not, start, stop, speech, image,
   timer, sensor_change, message, network, idle, personality, channel,
   on_error, event_policy, ai_policy, context_migration, priority,
   concurrent, critical, high, normal, low *)
```

---

## 8. ROADMAP COMPLETO — FASE 0 A FASE 10

---

### FASE 0 — Estudo e Modelagem Conceitual

**Duração estimada:** 2–4 semanas
**Objetivo:** Dominar os fundamentos antes de escrever código.

| Tópico | Por que importa | Recurso |
|---|---|---|
| Compiladores e Interpreters | Lexer, parser, AST são o núcleo da Fase 1 | "Crafting Interpreters" (Nystrom) |
| Actor Model | Modelo de concorrência dos agents | "Programming Erlang" (Armstrong) |
| Supervisor Pattern | Base do modelo de recuperação de falhas | Erlang/OTP docs: supervisors |
| Event Sourcing | Modelo para memória auditável | Martin Fowler: bliki/EventSourcing |
| Capability-Based Security | Modelo de permissões | "Capability Myths Demolished" (Miller) |
| Priority Queues | Base para Event Bus com prioridade | CLRS — cap. 6 (Heaps) |
| Sistemas de Tipos | Base para tipagem progressiva | "Types and Programming Languages" (Pierce) |
| MQTT Protocol | Base para IoT | Documentação oficial HiveMQ |

**Deliverables:**
- [ ] Gramática EBNF v1 (subset da Fase 1) escrita
- [ ] Diagrama de fluxo do sistema desenhado (incluindo Supervisor e Observer)
- [ ] Ambiente Rust configurado: workspace criado, `cargo build` sem erros
- [ ] Documento: "Como o CRL executa este código e o que acontece se falhar?"

---

### FASE 1 — Núcleo da Linguagem

**Duração estimada:** 4–8 semanas
**Objetivo:** Parsear e executar scripts CRL simples.

**Meta verificável:**
```
agent main {
    on start {
        let msg = "Hello, Cognitive Runtime"
        print(msg)
    }
}
```

**Componentes:**

#### 1.1 Lexer (`crl-lexer`)
Tokens com posição (linha, coluna). Usa `logos`. Coleta todos os erros antes de retornar.

#### 1.2 AST (`crl-ast`)
Todos os nós com `Span`. Inclui `ErrorPolicy` e `EventPolicy` como campos opcionais
em `AgentDecl` — parseados mas ignorados na execução até a Fase 2.
Inclui `channel_decl` como campo em `AgentDecl` — parseado mas ignorado até a Fase 4.

**CRÍTICO — AgentDecl já reflete a separação Definition/Instance:**
```rust
// No AST (Fase 1): o blueprint
pub struct AgentDecl {
    pub name:         String,
    pub version:      Option<u64>,         // para hot reload — Fase 7
    pub capabilities: Vec<UseDecl>,
    pub error_policy: Option<ErrorPolicy>, // parseado, ignorado na Fase 1
    pub event_policy: Option<EventPolicy>, // parseado, ignorado na Fase 1
    pub ai_policy:    Option<AiPolicy>,    // parseado, ignorado na Fase 8
    pub handlers:     Vec<HandlerDecl>,
    pub channels:     Vec<ChannelDecl>,    // parseado, ignorado na Fase 4
    pub functions:    Vec<FnDecl>,
    pub span:         Span,
}
```

#### 1.3 Parser (`crl-parser`)
Recursive descent. Parseia `on_error`, `event_policy`, `channel` mesmo que
ignore seus efeitos — a gramática não deve mudar nas fases seguintes.

#### 1.4 Interpreter (`crl-interpreter`)
AST-walking. Executa `on start`. Funções nativas: `print`, `len`, `type_of`, `to_str`.

**Deliverables:**
- [ ] 10 scripts de validação executando
- [ ] Erros com linha/coluna em todos os estágios
- [ ] `AgentDecl` aceita `on_error`, `event_policy`, `channel` sem falhar
- [ ] CLI: `crl run` e `crl check`
- [ ] Cobertura: lexer > 90%, parser > 85%

---

### FASE 2 — Runtime Contínuo + Supervisor + Observer

**Duração estimada:** 4–7 semanas
**Objetivo:** Sistema vivo que reage a eventos, se recupera de falhas e é observável.

**Meta verificável:**
```
agent clock {
    on_error {
        strategy    = "restart_handler"
        max_retries = 3
        backoff_ms  = 500
    }

    on start {
        print("Sistema iniciado")
    }

    on timer(1000) priority normal {
        print("tick")
    }

    on stop {
        print("Encerrando")
    }
}
```
Deve imprimir "tick" a cada segundo. Se o handler falhar, reinicia até 3x.
O Observer deve registrar cada execução.

**Componentes:**

#### 2.1 Event Bus com Prioridade
```rust
struct EventBus {
    queues: [VecDeque<PrioritizedEvent>; 4],  // uma fila por nível
    // Critical(0), High(1), Normal(2), Low(3)
}

// Poll: sempre drena Critical antes de High, High antes de Normal...
```

#### 2.2 AgentDefinition e AgentInstance
Separação formal implementada. `AgentDefinition` é `Arc<>` compartilhado.
`AgentInstance` tem mailbox, contexto (mínimo) e status.

#### 2.3 Supervisor
```rust
struct Supervisor {
    agents: HashMap<AgentId, SupervisionRecord>,
}

struct SupervisionRecord {
    policy:   ErrorPolicy,
    attempts: u32,
    last_err: Option<String>,
}

impl Supervisor {
    async fn handle_failure(&mut self, agent_id: AgentId, error: RuntimeError, observer: &Observer);
}
```

#### 2.4 Observer básico
Emite para stdout (LogObserver). Registra: HandlerStarted, HandlerFinished,
HandlerFailed, AgentRestarted, BackpressureDrop.

#### 2.5 Backpressure por tipo
Implementa `DropOldest` e `NeverDrop` para Critical.
`ErrorToAgent` e `DropNewest` entram na Fase 4.

**Deliverables:**
- [ ] Event bus com 4 filas de prioridade
- [ ] AgentDefinition e AgentInstance separados no código
- [ ] Supervisor aplicando ErrorPolicy corretamente
- [ ] Observer registrando execuções e falhas
- [ ] Timer com precisão ±10ms
- [ ] Shutdown gracioso (handlers em execução terminam antes de parar)
- [ ] 1000 eventos/segundo sem degradação
- [ ] Teste: handler que falha é reiniciado conforme a política
- [ ] Teste: evento Critical nunca é descartado mesmo com fila cheia

---

### FASE 3 — Sistema de Capabilities

**Duração estimada:** 2–4 semanas
**Objetivo:** Agents só usam recursos declarados explicitamente.

**Meta verificável:**
```
agent home {
    use lights
    on start { lights.enable() }   -- OK
}
agent visitor {
    on start { lights.enable() }   -- ERRO análise estática: capability não declarada
}
```

**Componentes:**
- Registry de capabilities com trait `Capability`
- Verificação estática no type checker
- 3 capabilities built-in (simuladas): `lights`, `speaker`, `logger`

**Deliverables:**
- [ ] Registry funcional
- [ ] Erro em compile-time com linha/coluna para capability não declarada
- [ ] 3 capabilities built-in respondendo a chamadas

---

### FASE 4 — Concorrência Real, Efeitos e Channels Tipados

**Duração estimada:** 4–6 semanas
**Objetivo:** Múltiplos agents colaborando com tipagem nos canais.

**Meta verificável:**
```
agent sensor {
    channel temperature: Float

    on timer(500) {
        let temp = thermometer.read()
        send("monitor").temperature(temp)
    }
}

agent monitor {
    on message(sensor.temperature) {
        -- event.payload: Float garantido
        if event.payload > 28.0 {
            do thermostat.cool(22.0)
        }
    }
}
```

**Componentes:**

#### 4.1 Keyword `do`
Side effects marcados explicitamente. Compilador valida.

#### 4.2 Channels Tipados
Declaração `channel nome: Tipo` no AgentDecl.
`send("agent").canal(value)` — verificado em tempo de análise.
Canal sem tipo: warning. Canal com tipo incompatível: erro.

#### 4.3 Sequencialidade intra-agent
Mailbox por AgentInstance garante um handler por vez.
`concurrent` keyword disponível para handlers sem escrita no contexto.

#### 4.4 Padrão Coordenador
Documentado nos exemplos. Um agent coordenador é criado como qualquer outro,
sem privilégios — usa `send()` para delegar tarefas a outros agents.

**Deliverables:**
- [ ] Channels tipados declarados e verificados
- [ ] `send("agent").canal(value)` funcionando entre 2 agents
- [ ] Sequencialidade intra-agent verificável com testes de concorrência
- [ ] `do` validado pelo compilador
- [ ] Backpressure `ErrorToAgent` implementado
- [ ] Exemplo de padrão coordenador executando

---

### FASE 5 — Contexto e Memória

**Duração estimada:** 3–6 semanas
**Objetivo:** Agents lembram, esquecem e consultam histórico.

**Camadas de memória:**

| Camada | Duração | Implementação | Caso de uso |
|---|---|---|---|
| Working | Handler atual | Variáveis locais | Computação temporária |
| Session | Enquanto o agent roda | `HashMap` em memória | Estado da conversa |
| Persistent | Indefinida | `sled` | Preferências, nome do usuário |
| Episodic | Indefinida + timeline | `sled` com timestamps | "O que aconteceu hoje?" |
| Semantic | Indefinida + busca vetorial | Embeddings (Fase 8) | Memória por significado |

**Commit atômico por handler:**
Todas as escritas no contexto dentro de um handler são aplicadas atomicamente
ao final. Se o handler falhar, nenhuma escrita é aplicada.

**Snapshot para hot reload (Fase 7):**
`context.snapshot()` serializa todas as camadas para restauração após atualização.

**API do Context:**
```
context.get(key)                    -- session
context.set(key, value)             -- session (não persiste)
context.remember(key, value)        -- persistent
context.forget(key)                 -- remove do persistent
context.episode(desc, data)         -- registra evento episódico
context.consolidate()               -- session → persistent com filtragem
context.search(query)               -- busca semântica (Fase 8)
context.snapshot()                  -- serializa para hot reload (Fase 7)
```

**Deliverables:**
- [ ] Session memory isolada por AgentInstance
- [ ] Persistent memory com sled
- [ ] Commit atômico verificável com testes
- [ ] TTL configurável por chave
- [ ] Snapshot serializado e restaurável
- [ ] Contexto de um agent nunca acessível a outro

---

### FASE 6 — Percepção Multimodal

**Duração estimada:** 4–8 semanas
**Objetivo:** Agents percebem o mundo real continuamente.

```
Microfone → ring buffer → VAD → STT (Whisper) → SpeechEvent(High)  → Event Bus
Câmera    → frame buffer → detecção            → ImageEvent(Low)    → Event Bus
Sensor    → polling/interrupt                  → SensorEvent(Normal)→ Event Bus
```

**Backpressure obrigatório:** `DropOldest` para `ImageEvent`.
Frames de câmera antigos são inúteis — nunca acumular.

**Deliverables:**
- [ ] Pipeline de áudio: mic → SpeechEvent com prioridade High
- [ ] STT integrado (Whisper)
- [ ] Pipeline de vídeo: câmera → ImageEvent com prioridade Low e DropOldest
- [ ] Trait genérico de sensor
- [ ] Todos os pipelines com backpressure configurado

---

### FASE 7 — Sistema de Plugins e Hot Reload

**Duração estimada:** 4–6 semanas
**Objetivo:** Capabilities adicionadas sem recompilar. Agents atualizados sem reiniciar.

**Hot reload de agents:**
```
1. Novo .crl compilado → nova AgentDefinition
2. Runtime notifica AgentInstance: "prepare para atualizar"
3. Handler em execução termina
4. context.snapshot() serializado
5. AgentInstance parada
6. Nova AgentInstance criada com nova Definition + contexto restaurado
7. Se context_migration declarada, executada antes de restaurar
```

**Hot reload de plugins:**
Plugins WASM recarregados via `wasmtime` sem reiniciar o runtime.

**Deliverables:**
- [ ] Carregamento de plugins WASM com sandbox
- [ ] Hot reload de plugin sem reiniciar runtime
- [ ] Hot reload de agent com preservação de contexto
- [ ] context_migration executada quando versão muda
- [ ] 2 plugins de exemplo funcionando

---

### FASE 8 — Motor de IA com Política de Latência

**Duração estimada:** 3–5 semanas
**Objetivo:** IA integrada como capability com comportamento definido em falha.

**Meta verificável:**
```
agent assistant {
    use ai
    use speaker

    ai_policy {
        timeout_ms        = 8000
        max_queue_during  = 3
        on_timeout        = "return_fallback"
        fallback_response = "Desculpe, não consegui processar agora."
        cache_identical   = true
    }

    on speech {
        let response = ai.reason(event.transcript, context)
        do speaker.say(response)
    }
}
```

**Implementação de timeout:**
```rust
// dentro do interpreter ao avaliar ai.reason():
let result = tokio::time::timeout(
    Duration::from_millis(policy.timeout_ms),
    llm_provider.complete(prompt, context)
).await;

match result {
    Ok(Ok(response)) => Value::String(response),
    Ok(Err(e))       => apply_timeout_behavior(policy.on_timeout, &policy.fallback_response, e),
    Err(_timeout)    => apply_timeout_behavior(policy.on_timeout, &policy.fallback_response, "timeout"),
}
```

**Deliverables:**
- [ ] Trait abstrato de LLM com 2 providers (OpenAI + Candle)
- [ ] AiCallPolicy aplicada: timeout, fallback, cache
- [ ] Fila de eventos durante processamento de IA respeitando `max_queue_during`
- [ ] Cache de respostas idênticas funcionando
- [ ] Runtime funciona 100% sem IA conectada
- [ ] Embeddings + busca semântica no context integrados

---

### FASE 9 — Agentes Cognitivos e Coordenação

**Duração estimada:** indefinida
**Objetivo:** Agents com personalidade, memória episódica, autonomia parcial e coordenação dinâmica.

**Meta conceitual:**
```
agent jarvis {
    personality {
        name      = "Jarvis"
        tone      = "professional"
        language  = "pt-BR"
        priorities= ["security", "user_comfort"]
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

**Coordenação dinâmica:**
O `ai.plan()` pode retornar ações que envolvem enviar mensagens para agents
especializados via `send()`. O planner age como coordenador dinâmico —
sem precisar de um "super-agente" fixo na arquitetura.

---

### FASE 10 — Integração Real: Rede, IoT e Acesso Remoto

**Duração estimada:** indefinida

**Sub-fase 10.1 — Gateway de Acesso Remoto:**
HTTP/WebSocket com JWT. Intenções remotas entram no Event Bus com prioridade High.

**Sub-fase 10.2 — Integração IoT via MQTT:**
```
agent home_iot {
    use mqtt
    use thermostat

    on network("mqtt/sensor/temperature") {
        let temp = event.payload.value
        if temp > 28 {
            do thermostat.cool(22)
        }
    }
}
```

---

## 9. GLOSSÁRIO TÉCNICO

**ADR** — Architecture Decision Record. Documento que registra uma decisão com contexto e raciocínio.

**Agent** — Unidade cognitiva. Tem capabilities, handlers, contexto e política de erro próprios.

**AgentDefinition** — O blueprint (código compilado) de um agent. Imutável, compartilhável via `Arc<>`.

**AgentInstance** — Um agent vivo em memória. Tem contexto, mailbox e status próprios.

**AiCallPolicy** — Política que define timeout, fallback e cache para chamadas de IA.

**AST** — Abstract Syntax Tree. Representação em árvore do programa.

**Backpressure** — Mecanismo que controla o que acontece quando uma fila está cheia.

**Capability** — Permissão declarada para acessar um recurso.

**Channel** — Canal de comunicação tipado entre agents.

**Context** — Memória de um AgentInstance. Isolada. Tem camadas: Working, Session, Persistent, Episodic, Semantic.

**Dispatcher** — Roteia eventos do bus para os handlers corretos.

**Effect** — Operação com consequência fora do escopo puro.

**ErrorPolicy** — Configuração de como um agent reage a falhas (Supervisor Pattern).

**Event** — Ocorrência tipada com prioridade.

**EventBus** — Canal central com filas de prioridade.

**EventPolicy** — Configuração de backpressure e tamanho de filas por agent.

**EventPriority** — Nível de urgência: Critical, High, Normal, Low.

**Gateway** — Ponto de entrada autenticado para intenções remotas.

**Handler** — Bloco `on evento {}` que reage a um evento.

**Hot Reload** — Atualização de agent ou plugin sem reiniciar o runtime.

**Lexer** — Converte texto em tokens.

**Mailbox** — Fila de entrada de mensagens de um AgentInstance.

**MQTT** — Protocolo leve de pub/sub para IoT.

**Observer** — Componente que registra eventos de observabilidade automaticamente.

**ObservabilityEvent** — Evento interno emitido pelo runtime para rastreamento.

**Parser** — Converte tokens em AST.

**Plugin** — Capability carregada dinamicamente via WASM.

**Scheduler** — Gerencia execução de tasks por agent, garantindo sequencialidade.

**Snapshot** — Serialização do contexto de um agent para hot reload.

**Span** — Par (start, end) de posições no source code.

**Supervisor** — Componente que monitora agents e aplica ErrorPolicy em caso de falha.

**Token** — Unidade atômica da linguagem.

**Value** — Tipo de dado em runtime: Int, Float, String, Bool, Null, List, Map, Function.

---

## 10. RISCOS E MITIGAÇÕES

| Risco | Prob. | Impacto | Mitigação |
|---|---|---|---|
| Escopo infinito | Alta | Alto | Deliverables verificáveis por fase. Não avançar sem completá-los. |
| Gramática incompatível com parser | Alta | Alto | Gramática EBNF escrita antes do parser. Testar com exemplos. |
| Refatoração de fundação pós-lançamento | Média | Muito Alto | Multi-agent ready desde Dia 1. Definition/Instance separados desde Fase 1. |
| Race conditions | Alta | Alto | Rust borrow checker + sequencialidade intra-agent + testes de concorrência. |
| Evento crítico descartado por backpressure | Média | Muito Alto | Fila `NeverDrop` dedicada para `Critical`. Nunca uma fila plana. |
| Handler falha silenciosamente | Alta | Alto | Supervisor Pattern desde Fase 2. Observer registra toda falha. |
| Sistema sem observabilidade | Média | Alto | Observer nativo desde Fase 2. Não é opcional. |
| Latência de IA trava o handler | Alta | Alto | AiCallPolicy com timeout obrigatório desde Fase 8. |
| Tipo errado em channel inter-agent | Alta | Médio | Channels tipados com verificação em análise estática (Fase 4). |
| Hot reload corrompe contexto | Média | Alto | context_migration + snapshot com versão antes de atualizar. |
| Plugins maliciosos | Alta | Alto | WASM sandbox obrigatório desde Fase 7. |
| Dependência de API externa cai | Alta | Médio | Abstração de provider + fallback local (Candle) + AiCallPolicy. |
| Gateway remoto comprometido | Alta | Muito Alto | JWT + capabilities subset + Observer registra toda intenção remota. |
| Burnout por complexidade | Média | Muito Alto | Uma fase de cada vez. Celebrar cada deliverable. |

---

## 11. PRINCÍPIOS QUE NUNCA DEVEM SER VIOLADOS

1. **A IA não é o núcleo.** O runtime funciona 100% sem IA conectada.

2. **Capabilities são explícitas.** Nenhum agent acessa recurso sem `use X`.

3. **Agents não compartilham memória.** Toda comunicação por channels tipados ou eventos.

4. **Efeitos são visíveis.** Side effects marcados com `do` e auditáveis pelo Observer.

5. **Erros têm localização.** Todo erro indica arquivo, linha e coluna exatos.

6. **Falhas são gerenciadas, não ignoradas.** Todo handler tem uma ErrorPolicy.
   Falha silenciosa não existe — o Supervisor age, o Observer registra.

7. **Eventos têm prioridade.** Nunca uma fila plana. Critical nunca é descartado.

8. **Backpressure tem política.** "Descarta" sem especificar o quê e quando é incompleto.

9. **Handlers são sequenciais por padrão.** Concorrência intra-agent é opt-in explícito.

10. **Canais entre agents são tipados.** `Value` genérico em channel é um smell, não uma feature.

11. **Uma fase por vez.** Não iniciar Fase N+1 antes dos deliverables da Fase N.

12. **Testes antes de avançar.** Cobertura mínima: 90% lexer, 85% parser, 80% runtime.

13. **A gramática é a fonte da verdade.** Parser derivado da gramática. Gramática muda primeiro.

14. **AgentDefinition e AgentInstance são separados.** Nunca misturar blueprint e instância.

15. **Intenções remotas são intenções, não código.** Gateway nunca executa CRL diretamente.

16. **Rust ownership é seu aliado.** Se o borrow checker reclama, o design está errado.

17. **Hipóteses são hipóteses.** Validar com spike antes de comprometer.

---

*Versão: 3.0.0 | Revisão: Auditoria técnica — 10 lacunas identificadas e incorporadas*
*Este documento é vivo. Atualize-o conforme decisões são validadas na prática.*
