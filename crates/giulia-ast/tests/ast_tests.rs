use giulia_ast::node::*;
use giulia_ast::policies::*;
use giulia_ast::types::TypeExpr;

fn dummy_span() -> Span {
    Span { start: 0, end: 0 }
}

fn dummy_block() -> Block {
    Block { stmts: vec![], span: dummy_span() }
}

// ─── Span ────────────────────────────────────────────────────────

#[test]
fn span_construction() {
    let s = Span { start: 5, end: 42 };
    assert_eq!(s.start, 5);
    assert_eq!(s.end, 42);
}

#[test]
fn span_default_is_zero() {
    let s = Span::default();
    assert_eq!(s.start, 0);
    assert_eq!(s.end, 0);
}

#[test]
fn span_copy_works() {
    let a = Span { start: 1, end: 10 };
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn span_partial_eq() {
    let a = Span { start: 1, end: 5 };
    let b = Span { start: 1, end: 5 };
    let c = Span { start: 2, end: 5 };
    assert_eq!(a, b);
    assert_ne!(a, c);
}

// ─── Program ─────────────────────────────────────────────────────

#[test]
fn program_has_span() {
    let prog = Program { stmts: vec![], span: Span { start: 0, end: 10 } };
    assert_eq!(prog.span.start, 0);
    assert_eq!(prog.span.end, 10);
}

#[test]
fn program_with_stmts() {
    let stmt = Stmt::ExprStmt(ExprStmt {
        expr: Expr::Literal(Literal::Int(42), dummy_span()),
        span: dummy_span(),
    });
    let prog = Program {
        stmts: vec![stmt],
        span: Span { start: 0, end: 5 },
    };
    assert_eq!(prog.stmts.len(), 1);
}

// ─── Stmt ────────────────────────────────────────────────────────

#[test]
fn stmt_agent_decl() {
    let agent = AgentDecl {
        name: "main".into(),
        version: None,
        capabilities: vec![],
        handlers: vec![],
        functions: vec![],
        error_policy: None,
        event_policy: None,
        ai_policy: None,
        channels: vec![],
        span: dummy_span(),
    };
    let stmt = Stmt::AgentDecl(agent);
    match stmt {
        Stmt::AgentDecl(a) => assert_eq!(a.name, "main"),
        _ => panic!("expected AgentDecl"),
    }
}

#[test]
fn stmt_fn_decl() {
    let func = FnDecl {
        name: "foo".into(),
        params: vec![],
        return_type: None,
        body: dummy_block(),
        span: dummy_span(),
    };
    let stmt = Stmt::FnDecl(func);
    match stmt {
        Stmt::FnDecl(f) => assert_eq!(f.name, "foo"),
        _ => panic!("expected FnDecl"),
    }
}

#[test]
fn stmt_let() {
    let stmt = Stmt::LetStmt(LetStmt {
        name: "x".into(),
        type_ann: None,
        value: Expr::Literal(Literal::Int(1), dummy_span()),
        span: dummy_span(),
    });
    match &stmt {
        Stmt::LetStmt(ls) => {
            assert_eq!(ls.name, "x");
            assert!(ls.type_ann.is_none());
        }
        _ => panic!("expected LetStmt"),
    }
}

#[test]
fn stmt_assign() {
    let stmt = Stmt::AssignStmt(AssignStmt {
        name: "x".into(),
        value: Expr::Literal(Literal::Int(2), dummy_span()),
        span: dummy_span(),
    });
    match &stmt {
        Stmt::AssignStmt(a) => assert_eq!(a.name, "x"),
        _ => panic!("expected AssignStmt"),
    }
}

#[test]
fn stmt_if() {
    let stmt = Stmt::IfStmt(IfStmt {
        condition: Expr::Literal(Literal::Bool(true), dummy_span()),
        then_branch: dummy_block(),
        else_branch: None,
        span: dummy_span(),
    });
    match &stmt {
        Stmt::IfStmt(i) => assert!(i.else_branch.is_none()),
        _ => panic!("expected IfStmt"),
    }
}

#[test]
fn stmt_if_else() {
    let stmt = Stmt::IfStmt(IfStmt {
        condition: Expr::Literal(Literal::Bool(true), dummy_span()),
        then_branch: dummy_block(),
        else_branch: Some(Box::new(ElseBranch::Block(dummy_block()))),
        span: dummy_span(),
    });
    match &stmt {
        Stmt::IfStmt(i) => assert!(i.else_branch.is_some()),
        _ => panic!("expected IfStmt"),
    }
}

#[test]
fn stmt_while() {
    let stmt = Stmt::WhileStmt(WhileStmt {
        condition: Expr::Literal(Literal::Bool(true), dummy_span()),
        body: dummy_block(),
        span: dummy_span(),
    });
    match &stmt {
        Stmt::WhileStmt(w) => {
            assert!(matches!(w.condition, Expr::Literal(Literal::Bool(true), _)));
        }
        _ => panic!("expected WhileStmt"),
    }
}

#[test]
fn stmt_for() {
    let stmt = Stmt::ForStmt(ForStmt {
        var: "i".into(),
        iterable: Expr::Identifier("items".into(), dummy_span()),
        body: dummy_block(),
        span: dummy_span(),
    });
    match &stmt {
        Stmt::ForStmt(f) => assert_eq!(f.var, "i"),
        _ => panic!("expected ForStmt"),
    }
}

#[test]
fn stmt_return_with_value() {
    let stmt = Stmt::ReturnStmt(ReturnStmt {
        value: Some(Expr::Literal(Literal::Null, dummy_span())),
        span: dummy_span(),
    });
    match &stmt {
        Stmt::ReturnStmt(r) => assert!(r.value.is_some()),
        _ => panic!("expected ReturnStmt"),
    }
}

#[test]
fn stmt_return_void() {
    let stmt = Stmt::ReturnStmt(ReturnStmt {
        value: None,
        span: dummy_span(),
    });
    match &stmt {
        Stmt::ReturnStmt(r) => assert!(r.value.is_none()),
        _ => panic!("expected ReturnStmt"),
    }
}

#[test]
fn stmt_effect() {
    let stmt = Stmt::EffectStmt(EffectStmt {
        expr: Expr::Literal(Literal::Int(1), dummy_span()),
        span: dummy_span(),
    });
    match &stmt {
        Stmt::EffectStmt(e) => {
            assert!(matches!(e.expr, Expr::Literal(Literal::Int(1), _)));
        }
        _ => panic!("expected EffectStmt"),
    }
}

#[test]
fn stmt_send() {
    let stmt = Stmt::SendStmt(SendStmt {
        target_agent: "logger".into(),
        channel: Some("log".into()),
        topic: None,
        payload: Expr::Literal(Literal::String("hello".into()), dummy_span()),
        span: dummy_span(),
    });
    match &stmt {
        Stmt::SendStmt(s) => {
            assert_eq!(s.target_agent, "logger");
            assert_eq!(s.channel, Some("log".into()));
        }
        _ => panic!("expected SendStmt"),
    }
}

#[test]
fn stmt_expr() {
    let stmt = Stmt::ExprStmt(ExprStmt {
        expr: Expr::Literal(Literal::String("hello".into()), dummy_span()),
        span: dummy_span(),
    });
    match &stmt {
        Stmt::ExprStmt(e) => {
            assert!(matches!(&e.expr, Expr::Literal(Literal::String(s), _) if s == "hello"));
        }
        _ => panic!("expected ExprStmt"),
    }
}

// ─── AgentDecl ───────────────────────────────────────────────────

#[test]
fn agent_decl_contains_capabilities_handlers_functions() {
    let agent = AgentDecl {
        name: "main".into(),
        version: Some(1),
        capabilities: vec![
            UseDecl { capability: "http".into(), span: dummy_span() },
            UseDecl { capability: "storage".into(), span: dummy_span() },
        ],
        handlers: vec![
            HandlerDecl {
                event: EventPattern::Start,
                priority: EventPriorityLevel::Normal,
                concurrent: false,
                body: dummy_block(),
                span: dummy_span(),
            },
        ],
        functions: vec![
            FnDecl {
                name: "helper".into(),
                params: vec![],
                return_type: Some(TypeExpr::Int),
                body: dummy_block(),
                span: dummy_span(),
            },
        ],
        error_policy: Some(ErrorPolicy {
            strategy: Some("restart_handler".into()),
            max_retries: Some(3),
            backoff_ms: Some(100),
            on_exhaust: Some("stop_agent".into()),
        }),
        event_policy: Some(EventPolicy {
            on_overflow: Some("drop_oldest".into()),
            critical_queue: Some(100),
            normal_queue: Some(50),
            low_queue: Some(10),
        }),
        ai_policy: Some(AiPolicy {
            timeout_ms: Some(5000),
            max_queue_during: Some(10),
            on_timeout: Some("return_fallback".into()),
            fallback_response: Some("default".into()),
            cache_identical: Some(true),
        }),
        channels: vec![
            ChannelDecl {
                name: "alerts".into(),
                chan_type: TypeExpr::String,
                span: dummy_span(),
            },
        ],
        span: dummy_span(),
    };

    assert_eq!(agent.version, Some(1));
    assert_eq!(agent.capabilities.len(), 2);
    assert_eq!(agent.capabilities[0].capability, "http");
    assert_eq!(agent.handlers.len(), 1);
    assert!(matches!(agent.handlers[0].event, EventPattern::Start));
    assert_eq!(agent.functions.len(), 1);
    assert_eq!(agent.functions[0].name, "helper");
    assert!(agent.error_policy.is_some());
    assert!(agent.event_policy.is_some());
    assert!(agent.ai_policy.is_some());
    assert_eq!(agent.channels.len(), 1);
}

#[test]
fn agent_decl_span() {
    let agent = AgentDecl {
        name: "main".into(),
        version: None,
        capabilities: vec![],
        handlers: vec![],
        functions: vec![],
        error_policy: None,
        event_policy: None,
        ai_policy: None,
        channels: vec![],
        span: Span { start: 0, end: 50 },
    };
    assert_eq!(agent.span.end, 50);
}

// ─── FnDecl ──────────────────────────────────────────────────────

#[test]
fn fn_decl_basic() {
    let func = FnDecl {
        name: "add".into(),
        params: vec![
            Param { name: "a".into(), type_annotation: Some(TypeExpr::Int), span: dummy_span() },
            Param { name: "b".into(), type_annotation: Some(TypeExpr::Int), span: dummy_span() },
        ],
        return_type: Some(TypeExpr::Int),
        body: Block {
            stmts: vec![Stmt::ReturnStmt(ReturnStmt { value: None, span: dummy_span() })],
            span: dummy_span(),
        },
        span: dummy_span(),
    };
    assert_eq!(func.name, "add");
    assert_eq!(func.params.len(), 2);
    assert_eq!(func.return_type, Some(TypeExpr::Int));
    assert_eq!(func.body.stmts.len(), 1);
}

// ─── EventPattern ────────────────────────────────────────────────

#[test]
fn event_pattern_start_stop() {
    assert!(matches!(EventPattern::Start, EventPattern::Start));
    assert!(matches!(EventPattern::Stop, EventPattern::Stop));
}

#[test]
fn event_pattern_timer() {
    let t = EventPattern::Timer(Expr::Literal(Literal::Int(1000), dummy_span()));
    assert!(matches!(t, EventPattern::Timer(_)));
}

#[test]
fn event_pattern_message_simple() {
    let m = EventPattern::Message(MessageTarget::Simple("topic".into()));
    assert!(matches!(m, EventPattern::Message(_)));
}

#[test]
fn event_pattern_message_typed() {
    let m = EventPattern::Message(MessageTarget::Typed {
        agent: "sensor".into(),
        channel: "temperature".into(),
    });
    match m {
        EventPattern::Message(MessageTarget::Typed { agent, channel }) => {
            assert_eq!(agent, "sensor");
            assert_eq!(channel, "temperature");
        }
        _ => panic!("expected Typed message"),
    }
}

#[test]
fn event_pattern_speech() {
    assert!(matches!(EventPattern::Speech, EventPattern::Speech));
}

#[test]
fn event_pattern_image() {
    assert!(matches!(EventPattern::Image, EventPattern::Image));
}

#[test]
fn event_pattern_sensor_change() {
    let s = EventPattern::SensorChange("gas_level".into());
    assert!(matches!(s, EventPattern::SensorChange(_)));
}

#[test]
fn event_pattern_network() {
    let n = EventPattern::Network("mqtt".into());
    assert!(matches!(n, EventPattern::Network(_)));
}

#[test]
fn event_pattern_idle() {
    let i = EventPattern::Idle(Expr::Literal(Literal::Int(5000), dummy_span()));
    assert!(matches!(i, EventPattern::Idle(_)));
}

#[test]
fn event_pattern_custom() {
    let c = EventPattern::Custom("my_event".into());
    assert!(matches!(c, EventPattern::Custom(_)));
}

// ─── MessageTarget ───────────────────────────────────────────────

#[test]
fn message_target_simple() {
    let m = MessageTarget::Simple("notifications".into());
    match m {
        MessageTarget::Simple(t) => assert_eq!(t, "notifications"),
        _ => panic!("expected Simple"),
    }
}

#[test]
fn message_target_typed() {
    let m = MessageTarget::Typed {
        agent: "alice".into(),
        channel: "talk".into(),
    };
    match m {
        MessageTarget::Typed { agent, channel } => {
            assert_eq!(agent, "alice");
            assert_eq!(channel, "talk");
        }
        _ => panic!("expected Typed"),
    }
}

// ─── SendStmt ────────────────────────────────────────────────────

#[test]
fn send_stmt_legacy() {
    let s = SendStmt {
        target_agent: "remote".into(),
        channel: None,
        topic: Some("cmd".into()),
        payload: Expr::Literal(Literal::Null, dummy_span()),
        span: dummy_span(),
    };
    assert_eq!(s.target_agent, "remote");
    assert_eq!(s.topic, Some("cmd".into()));
    assert!(s.channel.is_none());
}

#[test]
fn send_stmt_typed() {
    let s = SendStmt {
        target_agent: "sensor".into(),
        channel: Some("data".into()),
        topic: None,
        payload: Expr::Literal(Literal::Int(42), dummy_span()),
        span: dummy_span(),
    };
    assert_eq!(s.channel, Some("data".into()));
}

// ─── TypeExpr ────────────────────────────────────────────────────

#[test]
fn type_expr_all_variants() {
    let _int     = TypeExpr::Int;
    let _float   = TypeExpr::Float;
    let _string  = TypeExpr::String;
    let _bool    = TypeExpr::Bool;
    let _null    = TypeExpr::Null;
    let _list    = TypeExpr::List(Box::new(TypeExpr::Int));
    let _map     = TypeExpr::Map(Box::new(TypeExpr::String), Box::new(TypeExpr::Int));
    let _named   = TypeExpr::Named("MeuTipo".into());
    let _opt     = TypeExpr::Optional(Box::new(TypeExpr::Int));
}

#[test]
fn type_expr_list_nested() {
    let t = TypeExpr::List(Box::new(TypeExpr::List(Box::new(TypeExpr::Int))));
    assert!(matches!(t, TypeExpr::List(_)));
}

#[test]
fn type_expr_map_key_value() {
    let t = TypeExpr::Map(Box::new(TypeExpr::String), Box::new(TypeExpr::Float));
    assert!(matches!(t, TypeExpr::Map(_, _)));
}

#[test]
fn type_expr_named() {
    let t = TypeExpr::Named("MyType".into());
    assert_eq!(t, TypeExpr::Named("MyType".into()));
    assert_ne!(t, TypeExpr::Named("Other".into()));
}

// ─── Expr::span() ────────────────────────────────────────────────

#[test]
fn expr_span_literal() {
    let s = Span { start: 1, end: 5 };
    let e = Expr::Literal(Literal::Int(42), s);
    assert_eq!(e.span(), s);
}

#[test]
fn expr_span_identifier() {
    let s = Span { start: 10, end: 18 };
    let e = Expr::Identifier("foo".into(), s);
    assert_eq!(e.span(), s);
}

#[test]
fn expr_span_binop() {
    let s = Span { start: 0, end: 3 };
    let e = Expr::BinOp {
        op: BinOp::Add,
        left: Box::new(Expr::Literal(Literal::Int(1), dummy_span())),
        right: Box::new(Expr::Literal(Literal::Int(2), dummy_span())),
        span: s,
    };
    assert_eq!(e.span(), s);
}

#[test]
fn expr_span_unaryop() {
    let s = Span { start: 0, end: 2 };
    let e = Expr::UnaryOp {
        op: UnaryOp::Neg,
        operand: Box::new(Expr::Literal(Literal::Int(5), dummy_span())),
        span: s,
    };
    assert_eq!(e.span(), s);
}

#[test]
fn expr_span_call() {
    let s = Span { start: 0, end: 7 };
    let e = Expr::Call {
        callee: Box::new(Expr::Identifier("f".into(), dummy_span())),
        args: vec![Expr::Literal(Literal::Int(1), dummy_span())],
        span: s,
    };
    assert_eq!(e.span(), s);
}

#[test]
fn expr_span_field_access() {
    let s = Span { start: 0, end: 7 };
    let e = Expr::FieldAccess {
        object: Box::new(Expr::Identifier("obj".into(), dummy_span())),
        field: "prop".into(),
        span: s,
    };
    assert_eq!(e.span(), s);
}

#[test]
fn expr_span_index() {
    let s = Span { start: 0, end: 6 };
    let e = Expr::Index {
        object: Box::new(Expr::Identifier("a".into(), dummy_span())),
        index: Box::new(Expr::Literal(Literal::Int(0), dummy_span())),
        span: s,
    };
    assert_eq!(e.span(), s);
}

#[test]
fn expr_span_list() {
    let s = Span { start: 0, end: 5 };
    let e = Expr::List(vec![], s);
    assert_eq!(e.span(), s);
}

#[test]
fn expr_span_map() {
    let s = Span { start: 0, end: 11 };
    let e = Expr::Map(vec![
        (Expr::Literal(Literal::String("k".into()), dummy_span()),
         Expr::Literal(Literal::Int(1), dummy_span())),
    ], s);
    assert_eq!(e.span(), s);
}

// ─── Literal ─────────────────────────────────────────────────────

#[test]
fn literal_all_variants() {
    let _int       = Literal::Int(42);
    let _float     = Literal::Float(3.14);
    let _scientific = Literal::Scientific(1e4);
    let _string    = Literal::String("texto".into());
    let _bool      = Literal::Bool(true);
    let _null      = Literal::Null;
    let _list      = Literal::List(vec![]);
    let _map       = Literal::Map(vec![]);
}

#[test]
fn literal_partial_eq() {
    assert_eq!(Literal::Int(1), Literal::Int(1));
    assert_ne!(Literal::Int(1), Literal::Int(2));
    assert_eq!(Literal::Scientific(1.0), Literal::Scientific(1.0));
    assert_eq!(Literal::Bool(true), Literal::Bool(true));
    assert_eq!(Literal::String("a".into()), Literal::String("a".into()));
    assert_eq!(Literal::Null, Literal::Null);
}

// ─── BinOp / UnaryOp ─────────────────────────────────────────────

#[test]
fn binop_all_operators() {
    assert_eq!(BinOp::Add, BinOp::Add);
    assert_eq!(BinOp::Sub, BinOp::Sub);
    assert_eq!(BinOp::Mul, BinOp::Mul);
    assert_eq!(BinOp::Div, BinOp::Div);
    assert_eq!(BinOp::Rem, BinOp::Rem);
    assert_eq!(BinOp::Eq, BinOp::Eq);
    assert_eq!(BinOp::NotEq, BinOp::NotEq);
    assert_eq!(BinOp::Lt, BinOp::Lt);
    assert_eq!(BinOp::Gt, BinOp::Gt);
    assert_eq!(BinOp::LtEq, BinOp::LtEq);
    assert_eq!(BinOp::GtEq, BinOp::GtEq);
    assert_eq!(BinOp::And, BinOp::And);
    assert_eq!(BinOp::Or, BinOp::Or);
    assert_ne!(BinOp::Add, BinOp::Sub);
}

#[test]
fn unaryop_all_operators() {
    assert_eq!(UnaryOp::Neg, UnaryOp::Neg);
    assert_eq!(UnaryOp::Not, UnaryOp::Not);
    assert_ne!(UnaryOp::Neg, UnaryOp::Not);
}

// ─── ElseBranch ──────────────────────────────────────────────────

#[test]
fn else_branch_block() {
    let b = ElseBranch::Block(dummy_block());
    match b {
        ElseBranch::Block(block) => assert_eq!(block.stmts.len(), 0),
        _ => panic!("expected Block"),
    }
}

#[test]
fn else_branch_if() {
    let b = ElseBranch::If(IfStmt {
        condition: Expr::Literal(Literal::Bool(false), dummy_span()),
        then_branch: dummy_block(),
        else_branch: None,
        span: dummy_span(),
    });
    match b {
        ElseBranch::If(if_stmt) => {
            assert!(matches!(if_stmt.condition, Expr::Literal(Literal::Bool(false), _)));
        }
        _ => panic!("expected If"),
    }
}

// ─── Policies ────────────────────────────────────────────────────

#[test]
fn error_policy_default() {
    let p = ErrorPolicy::default();
    assert!(p.strategy.is_none());
    assert!(p.max_retries.is_none());
}

#[test]
fn error_policy_configured() {
    let p = ErrorPolicy {
        strategy: Some("restart_handler".into()),
        max_retries: Some(3),
        backoff_ms: Some(100),
        on_exhaust: Some("stop".into()),
    };
    assert_eq!(p.strategy, Some("restart_handler".into()));
}

#[test]
fn event_policy_default() {
    let p = EventPolicy::default();
    assert!(p.on_overflow.is_none());
}

#[test]
fn ai_policy_default() {
    let p = AiPolicy::default();
    assert!(p.timeout_ms.is_none());
}

#[test]
fn event_priority_level_default_is_normal() {
    let p = EventPriorityLevel::default();
    assert_eq!(p, EventPriorityLevel::Normal);
}

#[test]
fn event_priority_level_partial_eq() {
    assert_eq!(EventPriorityLevel::Critical, EventPriorityLevel::Critical);
    assert_ne!(EventPriorityLevel::Critical, EventPriorityLevel::Low);
}

#[test]
fn channel_decl_basic() {
    let c = ChannelDecl {
        name: "events".into(),
        chan_type: TypeExpr::Int,
        span: dummy_span(),
    };
    assert_eq!(c.name, "events");
    assert_eq!(c.chan_type, TypeExpr::Int);
}

// ─── Composite expressions ───────────────────────────────────────

#[test]
fn nested_binop_expression() {
    let expr = Expr::BinOp {
        op: BinOp::Add,
        left: Box::new(Expr::BinOp {
            op: BinOp::Mul,
            left: Box::new(Expr::Literal(Literal::Int(2), dummy_span())),
            right: Box::new(Expr::Literal(Literal::Int(3), dummy_span())),
            span: dummy_span(),
        }),
        right: Box::new(Expr::Literal(Literal::Int(4), dummy_span())),
        span: dummy_span(),
    };

    match expr {
        Expr::BinOp { op: BinOp::Add, left, right: _, .. } => {
            match *left {
                Expr::BinOp { op: BinOp::Mul, .. } => {}
                _ => panic!("expected Mul inside Add"),
            }
        }
        _ => panic!("expected BinOp Add"),
    }
}

#[test]
fn call_with_args() {
    let call = Expr::Call {
        callee: Box::new(Expr::Identifier("foo".into(), dummy_span())),
        args: vec![
            Expr::Literal(Literal::Int(1), dummy_span()),
            Expr::Literal(Literal::String("bar".into()), dummy_span()),
        ],
        span: dummy_span(),
    };
    match &call {
        Expr::Call { callee, args, .. } => {
            assert!(matches!(**callee, Expr::Identifier(_, _)));
            assert_eq!(args.len(), 2);
        }
        _ => panic!("expected Call"),
    }
}

#[test]
fn chained_field_access() {
    let expr = Expr::FieldAccess {
        object: Box::new(Expr::FieldAccess {
            object: Box::new(Expr::Identifier("obj".into(), dummy_span())),
            field: "inner".into(),
            span: dummy_span(),
        }),
        field: "value".into(),
        span: dummy_span(),
    };
    match expr {
        Expr::FieldAccess { object, field, .. } => {
            assert_eq!(field, "value");
            match *object {
                Expr::FieldAccess { field: inner_field, .. } => {
                    assert_eq!(inner_field, "inner");
                }
                _ => panic!("expected nested FieldAccess"),
            }
        }
        _ => panic!("expected FieldAccess"),
    }
}

#[test]
fn handler_decl_with_priority_and_concurrent() {
    let h = HandlerDecl {
        event: EventPattern::Timer(Expr::Literal(Literal::Int(1000), dummy_span())),
        priority: EventPriorityLevel::High,
        concurrent: true,
        body: dummy_block(),
        span: dummy_span(),
    };
    assert_eq!(h.priority, EventPriorityLevel::High);
    assert!(h.concurrent);
}

// ─── Programa exemplo do checklist ──────────────────────────────

#[test]
fn exemplo_agent_main_do_checklist() {
    let prog = Program {
        stmts: vec![
            Stmt::AgentDecl(AgentDecl {
                name: "main".into(),
                version: None,
                capabilities: vec![],
                handlers: vec![
                    HandlerDecl {
                        event: EventPattern::Start,
                        priority: EventPriorityLevel::High,
                        concurrent: false,
                        body: Block {
                            stmts: vec![
                                Stmt::LetStmt(LetStmt {
                                    name: "x".into(),
                                    type_ann: Some(TypeExpr::Int),
                                    value: Expr::BinOp {
                                        op: BinOp::Add,
                                        left: Box::new(Expr::Literal(Literal::Int(1), dummy_span())),
                                        right: Box::new(Expr::Literal(Literal::Int(2), dummy_span())),
                                        span: dummy_span(),
                                    },
                                    span: dummy_span(),
                                }),
                                Stmt::IfStmt(IfStmt {
                                    condition: Expr::BinOp {
                                        op: BinOp::Gt,
                                        left: Box::new(Expr::Identifier("x".into(), dummy_span())),
                                        right: Box::new(Expr::Literal(Literal::Int(2), dummy_span())),
                                        span: dummy_span(),
                                    },
                                    then_branch: Block {
                                        stmts: vec![
                                            Stmt::ExprStmt(ExprStmt {
                                                expr: Expr::Call {
                                                    callee: Box::new(Expr::Identifier("print".into(), dummy_span())),
                                                    args: vec![Expr::Identifier("x".into(), dummy_span())],
                                                    span: dummy_span(),
                                                },
                                                span: dummy_span(),
                                            }),
                                        ],
                                        span: dummy_span(),
                                    },
                                    else_branch: None,
                                    span: dummy_span(),
                                }),
                            ],
                            span: dummy_span(),
                        },
                        span: dummy_span(),
                    },
                ],
                functions: vec![],
                error_policy: Some(ErrorPolicy {
                    strategy: Some("restart_handler".into()),
                    max_retries: Some(3),
                    backoff_ms: None,
                    on_exhaust: None,
                }),
                event_policy: None,
                ai_policy: None,
                channels: vec![],
                span: dummy_span(),
            }),
        ],
        span: dummy_span(),
    };

    assert_eq!(prog.stmts.len(), 1);
    match &prog.stmts[0] {
        Stmt::AgentDecl(a) => {
            assert_eq!(a.name, "main");
            assert_eq!(a.handlers.len(), 1);
            assert_eq!(a.handlers[0].priority, EventPriorityLevel::High);
            assert!(a.handlers[0].concurrent == false);
            assert_eq!(a.handlers[0].body.stmts.len(), 2);
            assert!(a.error_policy.is_some());
            let ep = a.error_policy.as_ref().unwrap();
            assert_eq!(ep.strategy, Some("restart_handler".into()));
            assert_eq!(ep.max_retries, Some(3));
            assert!(a.event_policy.is_none());
            assert!(a.ai_policy.is_none());
            assert!(a.channels.is_empty());
        }
        _ => panic!("expected AgentDecl"),
    }
}

// ─── Debug ───────────────────────────────────────────────────────

#[test]
fn debug_format_does_not_panic() {
    let prog = Program { stmts: vec![], span: dummy_span() };
    let _debug = format!("{:?}", prog);
    let expr = Expr::Literal(Literal::Int(1), dummy_span());
    let _debug = format!("{:?}", expr);
    let t = TypeExpr::Int;
    let _debug = format!("{:?}", t);
    let p = ErrorPolicy::default();
    let _debug = format!("{:?}", p);
}
