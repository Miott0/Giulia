use giulia_lexer::lex;
use giulia_parser::parse;
use giulia_interpreter::{run, RuntimeError};

fn exec(source: &str) -> Result<(), RuntimeError> {
    let tokens = lex(source).expect("lex should succeed");
    let program = parse(tokens).expect("parse should succeed");
    run(&program, source)
}

fn exec_ok(source: &str) {
    exec(source).expect("exec should succeed");
}

fn exec_err(source: &str) -> RuntimeError {
    exec(source).expect_err("exec should fail")
}

// ─── 1. AgentDefinition e AgentInstance ──────────────────────────
// Tested implicitly via run() — the run function creates both.

// ─── 2. let x = expr ─────────────────────────────────────────────

#[test]
fn let_stmt_eval_and_store() {
    exec_ok("agent main { on start { let x = 42 } }");
}

// ─── 3. if / else ────────────────────────────────────────────────

#[test]
fn if_true_branch() {
    exec_ok(r#"agent main { on start { if true { print("yes") } else { print("no") } } }"#);
}

// ─── 4. while loop ───────────────────────────────────────────────

#[test]
fn while_loop_iterates() {
    exec_ok("agent main { on start { let i = 0 while i < 3 { print(i) i = i + 1 } } }");
}

// ─── 5. for x in list ────────────────────────────────────────────

#[test]
fn for_in_list() {
    exec_ok("agent main { on start { let items = [1, 2, 3] for item in items { print(item) } } }");
}

// ─── 6. Funções com escopo léxico ────────────────────────────────

#[test]
fn function_call_with_return() {
    exec_ok("agent main { fn dobro(n) { return n * 2 } on start { print(dobro(21)) } }");
}

// ─── 7. return ───────────────────────────────────────────────────

#[test]
fn return_in_function() {
    exec_ok("agent main { fn add(a, b) { return a + b } on start { print(add(3, 7)) } }");
}

// ─── 8. print, len, type_of, to_str ──────────────────────────────

#[test]
fn native_print() {
    exec_ok(r#"agent main { on start { print("hello") } }"#);
}

#[test]
fn native_len_string() {
    exec_ok(r#"agent main { on start { print(len("hello")) } }"#);
}

#[test]
fn native_len_list() {
    exec_ok("agent main { on start { print(len([1, 2, 3])) } }");
}

#[test]
fn native_type_of() {
    exec_ok("agent main { on start { print(type_of(42)) } }");
}

#[test]
fn native_to_str() {
    exec_ok("agent main { on start { print(to_str(42)) } }");
}

// ─── 9. Handler on start executado ───────────────────────────────

#[test]
fn on_start_executed() {
    exec_ok(r#"agent main { on start { print("started") } }"#);
}

// ─── 10-12. error_policy, event_policy, channel, send ignorados ──

#[test]
fn on_error_ignored_no_crash() {
    exec_ok("agent main { on_error { strategy = \"restart_handler\" max_retries = 3 } on start { print(\"ok\") } }");
}

#[test]
fn event_policy_ignored_no_crash() {
    exec_ok("agent main { event_policy { on_overflow = \"drop_oldest\" critical_queue = 100 } on start { print(\"ok\") } }");
}

#[test]
fn ai_policy_ignored_no_crash() {
    exec_ok("agent main { ai_policy { timeout_ms = 8000 fallback_response = \"...\" } on start { print(\"ok\") } }");
}

#[test]
fn channel_ignored_no_crash() {
    exec_ok("agent main { channel temperature: Float on start { print(\"ok\") } }");
}

#[test]
fn send_ignored_no_crash() {
    exec_ok("agent main { on start { send(\"monitor\").temperature(42.0) } }");
}

#[test]
fn send_legacy_ignored_no_crash() {
    exec_ok(r#"agent main { on start { send("agent", "topic", 99) } }"#);
}

// ─── 13. Erro linha/coluna variável não existe ───────────────────

#[test]
fn error_variable_not_found() {
    let err = exec_err("agent main { on start { let x = y_nao_existe } }");
    let msg = format!("{err}");
    assert!(msg.contains("y_nao_existe"), "error should mention variable name: {msg}");
    assert!(msg.contains("line") || msg.contains("linha"), "error should mention line: {msg}");
}

// ─── 14. Erro linha/coluna operação inválida ─────────────────────

#[test]
fn error_type_mismatch() {
    let err = exec_err("agent main { on start { let x = 42 + true } }");
    let msg = format!("{err}");
    assert!(msg.contains("line") || msg.contains("linha"), "error should mention line: {msg}");
    assert!(msg.contains("42") || msg.contains("Int") || msg.contains("Bool") || msg.contains("true"),
        "error should describe the type error: {msg}");
}

// ─── TESTES DO SPEC (12 scripts de validação) ────────────────────

#[test]
fn spec_test_1_hello() {
    exec_ok(r#"agent main { on start { print("Hello, Cognitive Runtime") } }"#);
}

#[test]
fn spec_test_2_arithmetic() {
    exec_ok("agent main { on start { let x = 10 let y = 20 print(x + y) } }");
}

#[test]
fn spec_test_3_conditional() {
    exec_ok(r#"agent main { on start { let x = 15 if x > 10 { print("maior") } else { print("menor") } } }"#);
}

#[test]
fn spec_test_4_while() {
    exec_ok("agent main { on start { let i = 0 while i < 5 { print(i) i = i + 1 } } }");
}

#[test]
fn spec_test_5_fn_with_types() {
    exec_ok("agent main { fn soma(a: Int, b: Int) -> Int { return a + b } on start { print(soma(3, 7)) } }");
}

#[test]
fn spec_test_6_fn_without_types() {
    exec_ok("agent main { fn dobro(n) { return n * 2 } on start { print(dobro(21)) } }");
}

#[test]
fn spec_test_7_capability_ignored() {
    exec_ok(r#"agent home { use lights on start { print("home iniciado") } }"#);
}

#[test]
fn spec_test_8_for_in_list() {
    exec_ok("agent main { on start { let items = [1, 2, 3] for item in items { print(item) } } }");
}

#[test]
fn spec_test_9_error_line() {
    let err = exec_err("agent main { on start { let x = y_nao_existe } }");
    let msg = format!("{err}");
    assert!(msg.contains("y_nao_existe"), "error should mention variable: {msg}");
}

#[test]
fn spec_test_10_on_error_no_crash() {
    exec_ok("agent main { on_error { strategy = \"restart_handler\" max_retries = 3 backoff_ms = 500 } on start { print(\"on_error aceito sem crash\") } }");
}

#[test]
fn spec_test_11_event_policy_no_crash() {
    exec_ok("agent main { event_policy { on_overflow = \"drop_oldest\" critical_queue = 100 } on start { print(\"event_policy aceito sem crash\") } }");
}

#[test]
fn spec_test_12_priority_and_channel_no_crash() {
    exec_ok("agent main { channel temperature: Float on start priority high { print(\"priority e channel aceitos\") } }");
}

// ─── EXTRA: expressões e operações ───────────────────────────────

#[test]
fn arithmetic_operations() {
    exec_ok("agent main { on start { let a = 1 + 2 * 3 print(a) } }");
}

#[test]
fn comparison_operations() {
    exec_ok("agent main { on start { let a = 1 < 2 and 3 == 3 print(a) } }");
}

#[test]
fn unary_operations() {
    exec_ok("agent main { on start { let a = -5 let b = not false print(a) print(b) } }");
}

#[test]
fn list_index() {
    exec_ok("agent main { on start { let xs = [10, 20, 30] print(xs[1]) } }");
}

#[test]
fn string_concatenation() {
    exec_ok(r#"agent main { on start { let s = "hello" + " world" print(s) } }"#);
}

#[test]
fn assign_variable() {
    exec_ok("agent main { on start { let x = 1 x = x + 1 print(x) } }");
}

#[test]
fn nested_if_else() {
    exec_ok(r#"agent main { on start { if false { print("a") } else if true { print("b") } else { print("c") } } }"#);
}

#[test]
fn division_by_zero_error() {
    let err = exec_err("agent main { on start { let x = 1 / 0 } }");
    let msg = format!("{err}");
    assert!(msg.contains("zero") || msg.contains("division"), "error should mention division by zero: {msg}");
}

#[test]
fn empty_agent_no_crash() {
    exec_ok("agent main { on start { } }");
}
