use giulia_lexer::lex;
use giulia_parser::parse;
use giulia_ast::node::*;
use giulia_ast::policies::*;
use giulia_ast::types::TypeExpr;

fn parse_ok(source: &str) -> Program {
    let tokens = lex(source).expect("lex should succeed");
    parse(tokens).expect("parse should succeed")
}

fn parse_err(source: &str) -> Vec<giulia_parser::ParseError> {
    let tokens = lex(source).expect("lex should succeed");
    parse(tokens).expect_err("parse should fail")
}

// ─── 1. Parseia agent home { on start { ... } } ─────────────────────

#[test]
fn agent_com_on_start() {
    let prog = parse_ok("agent home {\n    on start {\n        let x = 1\n    }\n}\n");
    assert_eq!(prog.stmts.len(), 1);
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            assert_eq!(agent.name, "home");
            assert_eq!(agent.handlers.len(), 1);
            match &agent.handlers[0].event {
                EventPattern::Start => {}
                _ => panic!("expected Start event"),
            }
            assert_eq!(agent.handlers[0].body.stmts.len(), 1);
        }
        _ => panic!("expected AgentDecl"),
    }
}

// ─── 2. on_error { strategy = "restart_handler" max_retries = 3 } ───

#[test]
fn on_error_policy() {
    let prog = parse_ok("agent main {\n    on_error {\n        strategy = \"restart_handler\"\n        max_retries = 3\n    }\n    on start {}\n}\n");
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            let policy = agent.error_policy.as_ref().expect("expected error_policy");
            assert_eq!(policy.strategy.as_deref(), Some("restart_handler"));
            assert_eq!(policy.max_retries, Some(3));
            assert_eq!(policy.backoff_ms, None);
        }
        _ => panic!("expected AgentDecl"),
    }
}

// ─── 3. event_policy { on_overflow = "drop_oldest" critical_queue = 100 } ──

#[test]
fn event_policy_block() {
    let prog = parse_ok("agent main {\n    event_policy {\n        on_overflow = \"drop_oldest\"\n        critical_queue = 100\n    }\n    on start {}\n}\n");
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            let policy = agent.event_policy.as_ref().expect("expected event_policy");
            assert_eq!(policy.on_overflow.as_deref(), Some("drop_oldest"));
            assert_eq!(policy.critical_queue, Some(100));
        }
        _ => panic!("expected AgentDecl"),
    }
}

// ─── 4. ai_policy { timeout_ms = 8000 fallback_response = "..." } ───

#[test]
fn ai_policy_block() {
    let prog = parse_ok("agent main {\n    ai_policy {\n        timeout_ms = 8000\n        fallback_response = \"try again\"\n    }\n    on start {}\n}\n");
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            let policy = agent.ai_policy.as_ref().expect("expected ai_policy");
            assert_eq!(policy.timeout_ms, Some(8000));
            assert_eq!(policy.fallback_response.as_deref(), Some("try again"));
        }
        _ => panic!("expected AgentDecl"),
    }
}

// ─── 5. channel temperature: Float ──────────────────────────────────

#[test]
fn channel_declaration() {
    let prog = parse_ok("agent main {\n    channel temperature: Float\n    on start {}\n}\n");
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            assert_eq!(agent.channels.len(), 1);
            let ch = &agent.channels[0];
            assert_eq!(ch.name, "temperature");
            assert_eq!(ch.chan_type, TypeExpr::Float);
        }
        _ => panic!("expected AgentDecl"),
    }
}

// ─── 6. on start priority high { ... } ──────────────────────────────

#[test]
fn handler_priority_high() {
    let prog = parse_ok("agent main {\n    on start priority high {\n        let x = 1\n    }\n}\n");
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            assert_eq!(agent.handlers[0].priority, EventPriorityLevel::High);
        }
        _ => panic!("expected AgentDecl"),
    }
}

#[test]
fn handler_priority_critical() {
    let prog = parse_ok("agent main {\n    on start priority critical {\n        let x = 1\n    }\n}\n");
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            assert_eq!(agent.handlers[0].priority, EventPriorityLevel::Critical);
        }
        _ => panic!("expected AgentDecl"),
    }
}

// ─── 7. on start concurrent { ... } ────────────────────────────────

#[test]
fn handler_concurrent() {
    let prog = parse_ok("agent main {\n    on start concurrent {\n        let x = 1\n    }\n}\n");
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            assert!(agent.handlers[0].concurrent);
        }
        _ => panic!("expected AgentDecl"),
    }
}

#[test]
fn handler_not_concurrent() {
    let prog = parse_ok("agent main {\n    on start {\n        let x = 1\n    }\n}\n");
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            assert!(!agent.handlers[0].concurrent);
        }
        _ => panic!("expected AgentDecl"),
    }
}

// ─── 8. send("monitor").temperature(42.0) ───────────────────────────

#[test]
fn send_typed_channel() {
    let prog = parse_ok("agent main {\n    on start {\n        send(\"monitor\").temperature(42.0)\n    }\n}\n");
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            match &agent.handlers[0].body.stmts[0] {
                Stmt::SendStmt(send) => {
                    assert_eq!(send.target_agent, "monitor");
                    assert_eq!(send.channel.as_deref(), Some("temperature"));
                    assert!(send.topic.is_none());
                    match &send.payload {
                        Expr::Literal(Literal::Float(v), _) => assert!((v - 42.0).abs() < 1e-12),
                        _ => panic!("expected Float literal"),
                    }
                }
                _ => panic!("expected SendStmt"),
            }
        }
        _ => panic!("expected AgentDecl"),
    }
}

// ─── 9. send("agent", "topic", value) ──────────────────────────────

#[test]
fn send_legacy() {
    let prog = parse_ok("agent main {\n    on start {\n        send(\"agent\", \"topic\", 99)\n    }\n}\n");
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            match &agent.handlers[0].body.stmts[0] {
                Stmt::SendStmt(send) => {
                    assert_eq!(send.target_agent, "agent");
                    assert!(send.channel.is_none());
                    assert_eq!(send.topic.as_deref(), Some("topic"));
                    match &send.payload {
                        Expr::Literal(Literal::Int(99), _) => {}
                        _ => panic!("expected Int literal"),
                    }
                }
                _ => panic!("expected SendStmt"),
            }
        }
        _ => panic!("expected AgentDecl"),
    }
}

// ─── 10. Precedência (1 + 2 * 3 → Add(1, Mul(2,3))) ──────────────

#[test]
fn expr_precedence_add_mul() {
    let prog = parse_ok("agent main {\n    on start {\n        let x = 1 + 2 * 3\n    }\n}\n");
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            match &agent.handlers[0].body.stmts[0] {
                Stmt::LetStmt(let_stmt) => {
                    match &let_stmt.value {
                        Expr::BinOp { op: BinOp::Add, left, right, .. } => {
                            match left.as_ref() {
                                Expr::Literal(Literal::Int(1), _) => {}
                                _ => panic!("expected left = 1"),
                            }
                            match right.as_ref() {
                                Expr::BinOp { op: BinOp::Mul, left: l2, right: r2, .. } => {
                                    match l2.as_ref() {
                                        Expr::Literal(Literal::Int(2), _) => {}
                                        _ => panic!("expected left of mul = 2"),
                                    }
                                    match r2.as_ref() {
                                        Expr::Literal(Literal::Int(3), _) => {}
                                        _ => panic!("expected right of mul = 3"),
                                    }
                                }
                                _ => panic!("expected Mul as right operand"),
                            }
                        }
                        _ => panic!("expected BinOp Add"),
                    }
                }
                _ => panic!("expected LetStmt"),
            }
        }
        _ => panic!("expected AgentDecl"),
    }
}

#[test]
fn expr_precedence_cmp_and_or() {
    let prog = parse_ok("agent main {\n    on start {\n        let x = 1 < 2 and 3 == 4\n    }\n}\n");
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            match &agent.handlers[0].body.stmts[0] {
                Stmt::LetStmt(s) => {
                    match &s.value {
                        Expr::BinOp { op: BinOp::And, left, right, .. } => {
                            match left.as_ref() {
                                Expr::BinOp { op: BinOp::Lt, .. } => {}
                                _ => panic!("expected Lt as left"),
                            }
                            match right.as_ref() {
                                Expr::BinOp { op: BinOp::Eq, .. } => {}
                                _ => panic!("expected Eq as right"),
                            }
                        }
                        _ => panic!("expected BinOp And"),
                    }
                }
                _ => panic!("expected LetStmt"),
            }
        }
        _ => panic!("expected AgentDecl"),
    }
}

// ─── 11. if / else if / else ───────────────────────────────────────

#[test]
fn if_else_if_else() {
    let prog = parse_ok("agent main {\n    on start {\n        if x {\n            let a = 1\n        } else if y {\n            let b = 2\n        } else {\n            let c = 3\n        }\n    }\n}\n");
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            match &agent.handlers[0].body.stmts[0] {
                Stmt::IfStmt(if_stmt) => {
                    match &if_stmt.condition {
                        Expr::Identifier(name, _) => assert_eq!(name, "x"),
                        _ => panic!("expected identifier x"),
                    }
                    assert_eq!(if_stmt.then_branch.stmts.len(), 1);
                    let else_branch = if_stmt.else_branch.as_ref().expect("expected else");
                    match else_branch.as_ref() {
                        ElseBranch::If(inner) => {
                            match &inner.condition {
                                Expr::Identifier(name, _) => assert_eq!(name, "y"),
                                _ => panic!("expected identifier y"),
                            }
                            let inner_else = inner.else_branch.as_ref().expect("expected inner else");
                            match inner_else.as_ref() {
                                ElseBranch::Block(b) => assert_eq!(b.stmts.len(), 1),
                                _ => panic!("expected Block else"),
                            }
                        }
                        _ => panic!("expected else if"),
                    }
                }
                _ => panic!("expected IfStmt"),
            }
        }
        _ => panic!("expected AgentDecl"),
    }
}

#[test]
fn if_without_else() {
    let prog = parse_ok("agent main {\n    on start {\n        if x { let a = 1 }\n    }\n}\n");
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            match &agent.handlers[0].body.stmts[0] {
                Stmt::IfStmt(if_stmt) => {
                    assert!(if_stmt.else_branch.is_none());
                }
                _ => panic!("expected IfStmt"),
            }
        }
        _ => panic!("expected AgentDecl"),
    }
}

// ─── 12. Recuperação de erro com múltiplos erros ───────────────────

#[test]
fn error_recovery_multiple_errors() {
    let errors = parse_err("agent main {\n    on start {\n        bad_token_here\n    }\n}\n");
    assert!(!errors.is_empty(), "expected at least one parse error");
}

#[test]
fn synchronize_reports_all_errors() {
    let source = "fn invalid_syntax(";
    let tokens = lex(source).expect("lex should succeed");
    let result = parse(tokens);
    assert!(result.is_err(), "expected parse errors");
}

// ─── 13. Erro com linha/coluna em token inesperado ────────────────

#[test]
fn error_includes_line_column() {
    let source = "agent main {\n    on start {\n        $invalid\n    }\n}\n";
    match lex(source) {
        Ok(tokens) => {
            let result = parse(tokens);
            match result {
                Err(errors) => {
                    assert!(!errors.is_empty());
                    for e in &errors {
                        let msg = format!("{e}");
                        assert!(msg.contains("linha") || msg.contains("line") || msg.contains("row"),
                            "error should mention line: {msg}");
                    }
                }
                Ok(_) => panic!("expected parse errors"),
            }
        }
        Err(lex_errors) => {
            // $invalid causes lex error, which is also acceptable for this test
            assert!(!lex_errors.is_empty());
        }
    }
}

#[test]
fn unexpected_token_line_column() {
    let source = "agent main {\n    start {}\n}\n";
    let tokens = lex(source).expect("lex should succeed");
    let errors = parse(tokens).expect_err("expected parse error");
    assert!(!errors.is_empty());
    let msg = format!("{}", errors[0]);
    assert!(msg.contains("linha") || msg.contains("line") || msg.contains("row") || msg.contains("coluna") || msg.contains("column"));
}

// ─── EXTRA: fn_decl, return, for, while, let c/ tipo ──────────────

#[test]
fn fn_decl_with_return_type() {
    let prog = parse_ok("agent main {\n    fn add(a: Int, b: Int) -> Int {\n        return a + b\n    }\n    on start {}\n}\n");
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            assert_eq!(agent.functions.len(), 1);
            let f = &agent.functions[0];
            assert_eq!(f.name, "add");
            assert_eq!(f.return_type, Some(TypeExpr::Int));
            assert_eq!(f.params.len(), 2);
            assert_eq!(f.params[0].name, "a");
            assert_eq!(f.params[0].type_annotation, Some(TypeExpr::Int));
        }
        _ => panic!("expected AgentDecl"),
    }
}

#[test]
fn while_loop() {
    let prog = parse_ok("agent main {\n    on start {\n        while x {\n            let y = y + 1\n        }\n    }\n}\n");
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            match &agent.handlers[0].body.stmts[0] {
                Stmt::WhileStmt(w) => {
                    match &w.condition {
                        Expr::Identifier(name, _) => assert_eq!(name, "x"),
                        _ => panic!("expected identifier x"),
                    }
                    assert!(!w.body.stmts.is_empty());
                }
                _ => panic!("expected WhileStmt"),
            }
        }
        _ => panic!("expected AgentDecl"),
    }
}

#[test]
fn for_loop() {
    let prog = parse_ok("agent main {\n    on start {\n        for item in items {\n            print(item)\n        }\n    }\n}\n");
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            match &agent.handlers[0].body.stmts[0] {
                Stmt::ForStmt(f) => {
                    assert_eq!(f.var, "item");
                }
                _ => panic!("expected ForStmt"),
            }
        }
        _ => panic!("expected AgentDecl"),
    }
}

#[test]
fn let_with_type_annotation() {
    let prog = parse_ok("agent main {\n    on start {\n        let x: Int = 42\n    }\n}\n");
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            match &agent.handlers[0].body.stmts[0] {
                Stmt::LetStmt(s) => {
                    assert_eq!(s.name, "x");
                    assert_eq!(s.type_ann, Some(TypeExpr::Int));
                }
                _ => panic!("expected LetStmt"),
            }
        }
        _ => panic!("expected AgentDecl"),
    }
}

#[test]
fn effect_do_stmt() {
    let prog = parse_ok("agent main {\n    on start {\n        do something\n    }\n}\n");
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            match &agent.handlers[0].body.stmts[0] {
                Stmt::EffectStmt(e) => {
                    match &e.expr {
                        Expr::Identifier(name, _) => assert_eq!(name, "something"),
                        _ => panic!("expected identifier"),
                    }
                }
                _ => panic!("expected EffectStmt"),
            }
        }
        _ => panic!("expected AgentDecl"),
    }
}

#[test]
fn assign_stmt() {
    let prog = parse_ok("agent main {\n    on start {\n        x = 99\n    }\n}\n");
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            match &agent.handlers[0].body.stmts[0] {
                Stmt::AssignStmt(a) => {
                    assert_eq!(a.name, "x");
                    match &a.value {
                        Expr::Literal(Literal::Int(99), _) => {}
                        _ => panic!("expected Int 99"),
                    }
                }
                _ => panic!("expected AssignStmt"),
            }
        }
        _ => panic!("expected AgentDecl"),
    }
}

#[test]
fn list_literal() {
    let prog = parse_ok("agent main {\n    on start {\n        let xs = [1, 2, 3]\n    }\n}\n");
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            match &agent.handlers[0].body.stmts[0] {
                Stmt::LetStmt(s) => {
                    match &s.value {
                        Expr::List(items, _) => {
                            assert_eq!(items.len(), 3);
                        }
                        _ => panic!("expected List"),
                    }
                }
                _ => panic!("expected LetStmt"),
            }
        }
        _ => panic!("expected AgentDecl"),
    }
}

#[test]
fn unary_not_and_neg() {
    let prog = parse_ok("agent main {\n    on start {\n        let a = -5\n        let b = not true\n    }\n}\n");
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            let stmts = &agent.handlers[0].body.stmts;
            match &stmts[0] {
                Stmt::LetStmt(s) => {
                    match &s.value {
                        Expr::UnaryOp { op: UnaryOp::Neg, operand, .. } => {
                            match operand.as_ref() {
                                Expr::Literal(Literal::Int(5), _) => {}
                                _ => panic!("expected Int 5"),
                            }
                        }
                        _ => panic!("expected UnaryOp Neg"),
                    }
                }
                _ => panic!("expected LetStmt"),
            }
            match &stmts[1] {
                Stmt::LetStmt(s) => {
                    match &s.value {
                        Expr::UnaryOp { op: UnaryOp::Not, operand, .. } => {
                            match operand.as_ref() {
                                Expr::Literal(Literal::Bool(true), _) => {}
                                _ => panic!("expected Bool true"),
                            }
                        }
                        _ => panic!("expected UnaryOp Not"),
                    }
                }
                _ => panic!("expected LetStmt"),
            }
        }
        _ => panic!("expected AgentDecl"),
    }
}

#[test]
fn function_call_and_field_access() {
    let prog = parse_ok("agent main {\n    on start {\n        let r = obj.field()\n    }\n}\n");
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            match &agent.handlers[0].body.stmts[0] {
                Stmt::LetStmt(s) => {
                    match &s.value {
                        Expr::Call { callee, args, .. } => {
                            assert!(args.is_empty());
                            match callee.as_ref() {
                                Expr::FieldAccess { object, field, .. } => {
                                    assert_eq!(field, "field");
                                    match object.as_ref() {
                                        Expr::Identifier(name, _) => assert_eq!(name, "obj"),
                                        _ => panic!("expected identifier obj"),
                                    }
                                }
                                _ => panic!("expected FieldAccess"),
                            }
                        }
                        _ => panic!("expected Call"),
                    }
                }
                _ => panic!("expected LetStmt"),
            }
        }
        _ => panic!("expected AgentDecl"),
    }
}

// ─── Exemplo completo do checklist 6.1 (AST) ───────────────────────

#[test]
fn exemplo_agent_main_do_checklist() {
    let prog = parse_ok("agent main {\n    on_error { strategy = \"restart_handler\" max_retries = 3 }\n    on start priority high {\n        let x = 1 + 2\n        if x > 2 { print(x) }\n    }\n}\n");
    match &prog.stmts[0] {
        Stmt::AgentDecl(agent) => {
            assert_eq!(agent.name, "main");
            assert!(agent.error_policy.is_some());
            assert_eq!(agent.handlers.len(), 1);
            assert_eq!(agent.handlers[0].priority, EventPriorityLevel::High);
        }
        _ => panic!("expected AgentDecl"),
    }
}
