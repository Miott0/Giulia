use giulia_ast::node::*;
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
        uses: vec![],
        handlers: vec![],
        fns: vec![],
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
        Stmt::AssignStmt(a) => {
            assert_eq!(a.name, "x");
        }
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
        Stmt::IfStmt(i) => {
            assert!(i.else_branch.is_none());
        }
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
        Stmt::IfStmt(i) => {
            assert!(i.else_branch.is_some());
        }
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
        Stmt::ForStmt(f) => {
            assert_eq!(f.var, "i");
        }
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
        Stmt::ReturnStmt(r) => {
            assert!(r.value.is_some());
        }
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
        Stmt::ReturnStmt(r) => {
            assert!(r.value.is_none());
        }
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
fn agent_decl_contains_uses_handlers_fns() {
    let agent = AgentDecl {
        name: "main".into(),
        uses: vec![
            UseDecl { capability: "http".into(), span: dummy_span() },
            UseDecl { capability: "storage".into(), span: dummy_span() },
        ],
        handlers: vec![
            HandlerDecl {
                event: EventPattern::Start,
                body: dummy_block(),
                span: dummy_span(),
            },
        ],
        fns: vec![
            FnDecl {
                name: "helper".into(),
                params: vec![],
                return_type: Some(TypeExpr::Int),
                body: dummy_block(),
                span: dummy_span(),
            },
        ],
        span: dummy_span(),
    };

    assert_eq!(agent.uses.len(), 2);
    assert_eq!(agent.uses[0].capability, "http");
    assert_eq!(agent.uses[1].capability, "storage");

    assert_eq!(agent.handlers.len(), 1);
    assert!(matches!(agent.handlers[0].event, EventPattern::Start));

    assert_eq!(agent.fns.len(), 1);
    assert_eq!(agent.fns[0].name, "helper");
    assert_eq!(agent.fns[0].return_type, Some(TypeExpr::Int));
}

#[test]
fn agent_decl_span() {
    let agent = AgentDecl {
        name: "main".into(),
        uses: vec![],
        handlers: vec![],
        fns: vec![],
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
            Param {
                name: "a".into(),
                type_annotation: Some(TypeExpr::Int),
                span: dummy_span(),
            },
            Param {
                name: "b".into(),
                type_annotation: Some(TypeExpr::Int),
                span: dummy_span(),
            },
        ],
        return_type: Some(TypeExpr::Int),
        body: Block {
            stmts: vec![Stmt::ReturnStmt(ReturnStmt {
                value: None,
                span: dummy_span(),
            })],
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
    let start = EventPattern::Start;
    let stop = EventPattern::Stop;
    assert!(matches!(start, EventPattern::Start));
    assert!(matches!(stop, EventPattern::Stop));
}

#[test]
fn event_pattern_timer() {
    let timer = EventPattern::Timer(Expr::Literal(Literal::Int(1000), dummy_span()));
    assert!(matches!(timer, EventPattern::Timer(_)));
}

#[test]
fn event_pattern_message() {
    let msg = EventPattern::Message("sensor/temp".into());
    assert!(matches!(msg, EventPattern::Message(_)));
}

#[test]
fn event_pattern_custom() {
    let custom = EventPattern::Custom("my_event".into());
    assert!(matches!(custom, EventPattern::Custom(_)));
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

    // If this compiles, all variants exist
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
    let _int    = Literal::Int(42);
    let _float  = Literal::Float(3.14);
    let _string = Literal::String("texto".into());
    let _bool   = Literal::Bool(true);
    let _null   = Literal::Null;
    let _list   = Literal::List(vec![]);
    let _map    = Literal::Map(vec![]);

    // If this compiles, all variants exist
}

#[test]
fn literal_partial_eq() {
    assert_eq!(Literal::Int(1), Literal::Int(1));
    assert_ne!(Literal::Int(1), Literal::Int(2));
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
                Expr::BinOp { op: BinOp::Mul, .. } => { /* (2 * 3) + 4 */ }
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

// ─── No unwrap/panic in nodes ────────────────────────────────────

// This test is a compile-time assertion:
// if any node had unwrap/panic in its construction, the tests above would fail.
// We also verify debug formatting doesn't panic.
#[test]
fn debug_format_does_not_panic() {
    let prog = Program { stmts: vec![], span: dummy_span() };
    let _debug = format!("{:?}", prog);
    let expr = Expr::Literal(Literal::Int(1), dummy_span());
    let _debug = format!("{:?}", expr);
    let t = TypeExpr::Int;
    let _debug = format!("{:?}", t);
}
