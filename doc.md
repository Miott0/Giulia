# GCRL — Giulia Cognitive Runtime Language

## Extensão: `.gcrl`

## Comentários

`: comentário de linha única`

```gcrl
/: comentario
multilinha :/
```

## Literais

| Tipo | Exemplos |
|------|----------|
| Inteiro | `42`, `0`, `-10` |
| Float | `3.14`, `-0.5` |
| Científico | `1e4`, `2.5e-3` |
| String | `"hello"`, `"mundo"` |
| Bool | `true`, `false` |
| Null | `null` |
| Lista | `[1, 2, 3]` |

## Operadores

| Categoria | Operadores |
|-----------|-----------|
| Aritmética | `+` `-` `*` `/` `%` |
| Comparação | `==` `!=` `<` `>` `<=` `>=` |
| Lógica | `and` `or` `not` |
| Atribuição | `=` |

Precedência (maior para menor):
1. `not` `-` (unário)
2. `*` `/` `%`
3. `+` `-`
4. `<` `>` `<=` `>=`
5. `==` `!=`
6. `and`
7. `or`

## Identificadores

`letras_numeros_e_underscores`, começando com letra ou `_`.

## F-Strings (Python-style)

Strings com prefixo `f` suportam interpolação de expressões com `{expr}`:

```gcrl
f"10 + 20 = {x + y}"
f"{nome} tem {idade} anos"
```

### Format specs

Suportam os mesmos formatos que Python para `int` e `float`:

| Formato | Exemplo | Resultado |
|---------|---------|-----------|
| `:.Nf` | `{pi:.2f}` | `3.14` |
| `:d` | `{n:d}` | `42` |
| `:x` | `{n:x}` | `2a` |
| `:X` | `{n:X}` | `2A` |
| `:b` | `{n:b}` | `101010` |
| `:o` | `{n:o}` | `52` |

### Chaves literais

Use `{{` e `}}` para escapar:

```gcrl
f"{{chaves}}"   /: resulta em "{chaves}" :/
```

## Estrutura de um programa

Um programa GCRL contém declarações de **agents** e/ou **funções** e/ou **variáveis** no nível superior.

## Agent

```gcrl
agent nome {
    use capability_name

    on start { ... }
    on stop { ... }
    on timer(expr) { ... }
    on message("topico") { ... }
    on speech { ... }
    on image { ... }
    on sensor_change("sensor_id") { ... }
    on network("evento") { ... }
    on idle(expr) { ... }

    fn nome_funcao(param1: Tipo, param2) -> Tipo { ... }

    channel nome_canal: Tipo

    on_error { strategy = "restart_handler" max_retries = 3 }
    event_policy { on_overflow = "drop_oldest" critical_queue = 100 }
    ai_policy { timeout_ms = 8000 fallback_response = "..." }
}
```

## Handlers

### on start
Executado quando o agent inicia:
```gcrl
on start {
    print("iniciado")
}
```

### on stop
Executado quando o agent para:
```gcrl
on stop {
    print("parando")
}
```

### on timer
Executado em intervalo:
```gcrl
on timer(5000) {
    print("5 segundos passaram")
}
```

### on message
Recebe mensagens:
```gcrl
/: simples :/
on message("topico") {
    print("mensagem recebida")
}

/: tipado (agent.canal) :/
on message(sensor.temperature) {
    print("temperatura recebida")
}
```

### Prioridade
```gcrl
on start priority critical { ... }
on start priority high { ... }
on start priority normal { ... }  /: padrão :/
on start priority low { ... }
```

### Concorrência
```gcrl
on start concurrent { ... }
on message("dados") concurrent { ... }
```

## Statements

### let
```gcrl
let x = 10
let nome: String = "Giulia"
let soma = 1 + 2
```

### assign
```gcrl
x = x + 1
nome = "outro"
```

### if / else if / else
```gcrl
if x > 10 {
    print("maior")
} else if x > 5 {
    print("medio")
} else {
    print("menor ou igual")
}
```

### while
```gcrl
let i = 0
while i < 5 {
    print(i)
    i = i + 1
}
```

### for
```gcrl
let items = [10, 20, 30]
for item in items {
    print(item)
}
```

### return
```gcrl
fn soma(a, b) {
    return a + b
}
```

### do (efeito colateral)
```gcrl
do alguma_acao()
```

### send
```gcrl
/: canal tipado :/
send("monitor").temperatura(42.0)

/: legado (topic string) :/
send("agent", "topic", value)
```

## Expressões

```gcrl
1 + 2 * 3          /: = 7 (multiplicacao primeiro) :/
(1 + 2) * 3        /: = 9 :/
not true           /: = false :/
-x                 /: negacao :/
xs[0]              /: indexacao :/
obj.field()        /: chamada de metodo :/
```

## Funções

```gcrl
fn soma(a: Int, b: Int) -> Int {
    return a + b
}

/: sem tipos :/
fn dobro(n) {
    return n * 2
}

fn saudacao(nome) {
    print("Ola, " + nome)
}
```

## Funções Nativas

| Função | Descrição |
|--------|-----------|
| `print(...)` | Imprime no stdout |
| `len(lista_ou_string)` | Retorna o tamanho |
| `type_of(valor)` | Retorna o nome do tipo como string |
| `to_str(valor)` | Converte para string |

## Políticas

### on_error
```gcrl
on_error {
    strategy    = "restart_handler"
    max_retries = 3
    backoff_ms  = 500
    on_exhaust  = "stop_agent"
}
```

### event_policy
```gcrl
event_policy {
    on_overflow    = "drop_oldest"
    critical_queue = 100
    normal_queue   = 50
    low_queue      = 20
}
```

### ai_policy
```gcrl
ai_policy {
    timeout_ms        = 8000
    max_queue_during  = 5
    on_timeout        = "return_fallback"
    fallback_response = "nao entendi"
    cache_identical   = true
}
```

## Canais
```gcrl
channel temperature: Float
channel status: String
```

---

# Exemplos por Nível de Complexidade

> Antes de executar qualquer exemplo, instale o comando `giulia`:
> ```bash
> cd /caminho/para/Giulia
> cargo install --path crates/giulia-cli
> ```
> Agora `giulia` está disponível como comando global. Basta rodar:
> ```bash
> giulia run exemplo.gcrl
> ```

## Básico

### Hello World
```gcrl
agent main {
    on start {
        print("Hello, Cognitive Runtime")
    }
}
```
`giulia run hello.gcrl` → `Hello, Cognitive Runtime`

### Aritmética simples
```gcrl
agent main {
    on start {
        let x = 10
        let y = 20
        print(x + y)
    }
}
```
`giulia run` → `30`

### Variáveis e tipos
```gcrl
agent main {
    on start {
        let nome = "Giulia"
        let idade = 1
        let altura = 0.5
        print(nome)
        print(idade)
        print(altura)
    }
}
```
`giulia run` → `Giulia 1 0.5`

### Print com múltiplos argumentos
```gcrl
agent main {
    on start {
        print("um", 2, true, null, [1, 2])
    }
}
```
`giulia run` → `um 2 true null [1, 2]`

### Uso de `type_of` e `to_str`
```gcrl
agent main {
    on start {
        print(type_of(42))
        print(type_of("texto"))
        print(to_str(3.14))
    }
}
```
`giulia run` → `Int String 3.14`

---

## Médio

### Condicional if/else
```gcrl
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
```
`giulia run` → `maior`

### If/else if/else
```gcrl
agent main {
    on start {
        let nota = 7
        if nota >= 9 {
            print("A")
        } else if nota >= 7 {
            print("B")
        } else {
            print("C")
        }
    }
}
```
`giulia run` → `B`

### Loop while
```gcrl
agent main {
    on start {
        let i = 0
        while i < 5 {
            print(i)
            i = i + 1
        }
    }
}
```
`giulia run` → `0 1 2 3 4`

### Loop for em lista
```gcrl
agent main {
    on start {
        let items = [1, 2, 3]
        for item in items {
            print(item)
        }
    }
}
```
`giulia run` → `1 2 3`

### Função com retorno
```gcrl
agent main {
    fn soma(a: Int, b: Int) -> Int {
        return a + b
    }
    on start {
        print(soma(3, 7))
    }
}
```
`giulia run` → `10`

### Função sem tipos
```gcrl
agent main {
    fn dobro(n) {
        return n * 2
    }
    on start {
        print(dobro(21))
    }
}
```
`giulia run` → `42`

### Operadores de comparação e lógicos
```gcrl
agent main {
    on start {
        let a = 10
        let b = 20
        print(a == b)
        print(a < b)
        print(a < b and b < 30)
        print(not (a == b))
    }
}
```
`giulia run` → `false true true true`

### Acesso a lista por índice
```gcrl
agent main {
    on start {
        let items = [10, 20, 30]
        print(items[0])
        print(items[2])
    }
}
```
`giulia run` → `10 30`

### F-strings básicas
```gcrl
agent main {
    on start {
        let nome = "Giulia"
        let idade = 1
        print(f"nome: {nome}, idade: {idade}")
    }
}
```
`giulia run` → `nome: Giulia, idade: 1`

### F-strings com expressões
```gcrl
agent main {
    on start {
        let x = 10
        let y = 20
        print(f"{x} + {y} = {x + y}")
    }
}
```
`giulia run` → `10 + 20 = 30`

---

## Complexo

### F-strings com format specs
```gcrl
agent main {
    on start {
        let pi = 3.1415926535
        let n = 255
        print(f"float:  {pi:.2f}")
        print(f"hex:    {n:x}")
        print(f"HEX:    {n:X}")
        print(f"bin:    {n:b}")
        print(f"octal:  {n:o}")
        print(f"int:    {n:d}")
    }
}
```
`giulia run` → `float: 3.14  hex: ff  HEX: FF  bin: 11111111  octal: 377  int: 255`

### F-strings com chaves literais
```gcrl
agent main {
    on start {
        let x = 42
        print(f"{{x}} = {x}")
        print(f"{{{{chaves}}}}")
    }
}
```
`giulia run` → `{x} = 42  {{chaves}}`

### Combinação: for + if + funções
```gcrl
agent main {
    fn eh_par(n) {
        return n % 2 == 0
    }

    on start {
        let nums = [1, 2, 3, 4, 5, 6]
        for n in nums {
            if eh_par(n) {
                print(f"{n} eh par")
            } else {
                print(f"{n} eh impar")
            }
        }
    }
}
```
`giulia run` → `1 eh impar  2 eh par  3 eh impar  4 eh par  5 eh impar  6 eh par`

### Combinação: cálculo com formatação
```gcrl
agent main {
    fn fatorial(n) {
        if n <= 1 {
            return 1
        }
        return n * fatorial(n - 1)
    }

    on start {
        let i = 0
        while i <= 10 {
            print(f"{i}! = {fatorial(i)}")
            i = i + 1
        }
    }
}
```
`giulia run` → `0! = 1  1! = 1  2! = 2  3! = 6  4! = 24  5! = 120  6! = 720  7! = 5040  8! = 40320  9! = 362880  10! = 3628800`

### Tabela de multiplicação formatada
```gcrl
agent main {
    on start {
        let i = 1
        while i <= 10 {
            let j = 1
            while j <= 10 {
                print(f"{i} x {j} = {i * j}")
                j = j + 1
            }
            i = i + 1
        }
    }
}
```
`giulia run` → tabuada do 1 ao 10

### Contagem regressiva com condição composta
```gcrl
agent main {
    on start {
        let i = 10
        while i >= 0 {
            if i > 0 {
                print(f"{i}...")
            } else {
                print("fogo!")
            }
            i = i - 1
        }
    }
}
```
`giulia run` → `10... 9... 8... ... 1... fogo!`

### Capacidade declarada + políticas (parseadas, sem execução)
```gcrl
agent main {
    use camera
    use speech

    on_error {
        strategy    = "restart_handler"
        max_retries = 3
        backoff_ms  = 500
    }
    event_policy {
        on_overflow    = "drop_oldest"
        critical_queue = 100
    }
    ai_policy {
        timeout_ms        = 5000
        fallback_response = "nao entendi"
    }
    channel temperature: Float

    fn processar(valor) {
        return valor * 2
    }

    on start priority high {
        let leituras = [22.5, 23.1, 21.8]
        for leitura in leituras {
            print(f"temperatura processada: {processar(leitura):.1f}")
        }
        print(f"tipos: {type_of(leituras)}, total: {len(leituras)}")
    }
}
```
`giulia run` → `temperatura processada: 45.0  temperatura processada: 46.2  temperatura processada: 43.6  tipos: List, total: 3`

---

## Instalação

```bash
# 1. Instalar Rust (uma vez):
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Compilar e instalar o binário globalmente (uma vez):
cd /caminho/para/Giulia
cargo install --path crates/giulia-cli

# 3. Pronto — agora giulia funciona como comando global:
giulia run hello.gcrl
```

Isso instala o binário em `~/.cargo/bin/giulia` (já no PATH se usou rustup). Depois disso, `giulia` vira comando global igual `python` — compila uma vez, usa pra sempre.

## CLI

```bash
giulia run arquivo.gcrl       # Executa o script
giulia check arquivo.gcrl     # Mostra tokens + AST (sem executar)
giulia repl                   # Modo interativo
giulia --version              # Versão
```

## Erros

Todos os erros (léxicos, sintáticos, runtime) incluem **linha e coluna** exatas.

```gcrl
agent main {
    on start {
        let x = @invalido   /: erro lexico na linha 3, coluna 17 :/
        let y = unknown_var /: erro runtime na linha 4, coluna 17 :/
    }
}
```
