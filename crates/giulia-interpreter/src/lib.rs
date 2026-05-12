mod environment;
mod error;
mod value;

use std::sync::Arc;

pub use error::RuntimeError;
pub use value::Value;

use environment::Environment;
use giulia_ast::node::*;

// ─── SOURCE -> LINE/COL ──────────────────────────────────────────

fn line_col(source: &str, pos: usize) -> (usize, usize) {
    let pos = pos.min(source.len());
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in source.char_indices() {
        if i >= pos {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

// ─── AGENT DEFINITION / INSTANCE ─────────────────────────────────

pub struct AgentDefinition {
    pub name:     String,
    pub handlers: Vec<HandlerDecl>,
    pub functions: Vec<FnDecl>,
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

pub struct AgentInstance {
    pub definition: Arc<AgentDefinition>,
    pub env:        Environment,
    pub source:     Arc<String>,
}

impl AgentInstance {
    fn exec_block(&mut self, block: &Block) -> Result<Option<Value>, RuntimeError> {
        self.env.push_scope();
        for stmt in &block.stmts {
            if let Some(val) = self.exec_stmt(stmt)? {
                self.env.pop_scope();
                return Ok(Some(val));
            }
        }
        self.env.pop_scope();
        Ok(None)
    }

    fn exec_stmt(&mut self, stmt: &Stmt) -> Result<Option<Value>, RuntimeError> {
        match stmt {
            Stmt::LetStmt(s) => {
                let val = self.eval_expr(&s.value)?;
                self.env.define(s.name.clone(), val);
                Ok(None)
            }
            Stmt::AssignStmt(s) => {
                let val = self.eval_expr(&s.value)?;
                if !self.env.assign(&s.name, val.clone()) {
                    let (l, c) = line_col(&self.source, s.span.start);
                    return Err(RuntimeError::VariableNotFound {
                        name: s.name.clone(),
                        line: l, column: c,
                    });
                }
                Ok(None)
            }
            Stmt::IfStmt(s) => {
                let cond = self.eval_expr(&s.condition)?;
                if is_truthy(&cond) {
                    return self.exec_block(&s.then_branch);
                }
                if let Some(else_branch) = &s.else_branch {
                    match else_branch.as_ref() {
                        ElseBranch::Block(b) => self.exec_block(b),
                        ElseBranch::If(inner) => {
                            self.exec_stmt(&Stmt::IfStmt(inner.clone()))
                        }
                    }
                } else {
                    Ok(None)
                }
            }
            Stmt::WhileStmt(s) => {
                loop {
                    let cond = self.eval_expr(&s.condition)?;
                    if !is_truthy(&cond) {
                        break;
                    }
                    if let Some(val) = self.exec_block(&s.body)? {
                        return Ok(Some(val));
                    }
                }
                Ok(None)
            }
            Stmt::ForStmt(s) => {
                let iterable = self.eval_expr(&s.iterable)?;
                let items = match &iterable {
                    Value::List(items) => items.clone(),
                    other => {
                        let (l, c) = line_col(&self.source, s.span.start);
                        return Err(RuntimeError::TypeError {
                            message: format!("expected List, got {}", other.type_name()),
                            line: l, column: c,
                        });
                    }
                };
                for item in items {
                    self.env.push_scope();
                    self.env.define(s.var.clone(), item);
                    for inner in &s.body.stmts {
                        if let Some(val) = self.exec_stmt(inner)? {
                            self.env.pop_scope();
                            return Ok(Some(val));
                        }
                    }
                    self.env.pop_scope();
                }
                Ok(None)
            }
            Stmt::ReturnStmt(s) => {
                let val = if let Some(expr) = &s.value {
                    self.eval_expr(expr)?
                } else {
                    Value::Null
                };
                Err(RuntimeError::Return(Box::new(val)))
            }
            Stmt::EffectStmt(s) => {
                self.eval_expr(&s.expr)?;
                Ok(None)
            }
            Stmt::SendStmt(_) => Ok(None),
            Stmt::ExprStmt(s) => {
                self.eval_expr(&s.expr)?;
                Ok(None)
            }
            Stmt::AgentDecl(_) | Stmt::FnDecl(_) => Ok(None),
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(lit, _span) => Ok(literal_to_value(lit)),
            Expr::Identifier(name, span) => {
                match self.env.get(name) {
                    Some(val) => Ok(val.clone()),
                    None => {
                        let (l, c) = line_col(&self.source, span.start);
                        Err(RuntimeError::VariableNotFound {
                            name: name.clone(),
                            line: l, column: c,
                        })
                    }
                }
            }
            Expr::BinOp { op, left, right, span } => {
                let lv = self.eval_expr(left)?;
                let rv = self.eval_expr(right)?;
                let (line, col) = line_col(&self.source, span.start);
                eval_binop(&lv, &rv, op, line, col)
            }
            Expr::UnaryOp { op, operand, span } => {
                let val = self.eval_expr(operand)?;
                let (line, col) = line_col(&self.source, span.start);
                eval_unary(&val, op, line, col)
            }
            Expr::Call { callee, args, span } => {
                let (line, col) = line_col(&self.source, span.start);
                if let Expr::Identifier(name, _) = callee.as_ref() {
                    if let Some(result) = self.try_native(name, args)? {
                        return Ok(result);
                    }
                }
                let callee_val = self.eval_expr(callee)?;
                match callee_val {
                    Value::Function(fn_decl) => {
                        let mut arg_vals = Vec::new();
                        for arg in args {
                            arg_vals.push(self.eval_expr(arg)?);
                        }
                        self.call_user_fn(&fn_decl, arg_vals, line, col)
                    }
                    other => Err(RuntimeError::TypeError {
                        message: format!("cannot call {}", other.type_name()),
                        line, column: col,
                    }),
                }
            }
            Expr::FieldAccess { object, field, span } => {
                let obj = self.eval_expr(object)?;
                let (l, c) = line_col(&self.source, span.start);
                Err(RuntimeError::TypeError {
                    message: format!("field access `{}` not supported on {}", field, obj.type_name()),
                    line: l, column: c,
                })
            }
            Expr::Index { object, index, span } => {
                let obj = self.eval_expr(object)?;
                let idx = self.eval_expr(index)?;
                let (l, c) = line_col(&self.source, span.start);
                match (&obj, &idx) {
                    (Value::List(items), Value::Int(i)) => {
                        let i = *i;
                        if i < 0 || i as usize >= items.len() {
                            return Err(RuntimeError::TypeError {
                                message: format!("index {} out of bounds (len {})", i, items.len()),
                                line: l, column: c,
                            });
                        }
                        Ok(items[i as usize].clone())
                    }
                    (o, iv) => Err(RuntimeError::TypeError {
                        message: format!("cannot index {} with {}", o.type_name(), iv.type_name()),
                        line: l, column: c,
                    }),
                }
            }
            Expr::List(items, _span) => {
                let mut vals = Vec::new();
                for item in items {
                    vals.push(self.eval_expr(item)?);
                }
                Ok(Value::List(vals))
            }
            Expr::Map(_entries, span) => {
                let (l, c) = line_col(&self.source, span.start);
                Err(RuntimeError::TypeError {
                    message: "Map literals not supported in Phase 1".into(),
                    line: l, column: c,
                })
            }
            Expr::FString(segments, _span) => {
                let mut result = String::new();
                for seg in segments {
                    match seg {
                        FStringSegment::Lit(s) => result.push_str(s),
                        FStringSegment::Expr(expr, fmt) => {
                            let val = self.eval_expr(expr)?;
                            result.push_str(&format_value(&val, fmt.as_deref()));
                        }
                    }
                }
                Ok(Value::String(result))
            }
        }
    }

    fn try_native(&mut self, name: &str, args: &[Expr]) -> Result<Option<Value>, RuntimeError> {
        match name {
            "print" => {
                let vals: Result<Vec<Value>, RuntimeError> = args.iter().map(|a| self.eval_expr(a)).collect();
                let vals = vals?;
                if vals.is_empty() {
                    println!();
                } else {
                    let s = vals.iter().map(|v| v.to_string_value()).collect::<Vec<_>>().join(" ");
                    println!("{s}");
                }
                Ok(Some(Value::Null))
            }
            "len" => {
                if args.len() != 1 {
                    let (l, c) = line_col(&self.source, 0);
                    return Err(RuntimeError::NativeError {
                        message: "len expects 1 argument".into(),
                        line: l, column: c,
                    });
                }
                let val = self.eval_expr(&args[0])?;
                match &val {
                    Value::String(s) => Ok(Some(Value::Int(s.len() as i64))),
                    Value::List(items) => Ok(Some(Value::Int(items.len() as i64))),
                    other => {
                        let (l, c) = line_col(&self.source, args[0].span().start);
                        Err(RuntimeError::TypeError {
                            message: format!("len expects String or List, got {}", other.type_name()),
                            line: l, column: c,
                        })
                    }
                }
            }
            "type_of" => {
                if args.len() != 1 {
                    let (l, c) = line_col(&self.source, 0);
                    return Err(RuntimeError::NativeError {
                        message: "type_of expects 1 argument".into(),
                        line: l, column: c,
                    });
                }
                let val = self.eval_expr(&args[0])?;
                Ok(Some(Value::String(val.type_name())))
            }
            "to_str" => {
                if args.len() != 1 {
                    let (l, c) = line_col(&self.source, 0);
                    return Err(RuntimeError::NativeError {
                        message: "to_str expects 1 argument".into(),
                        line: l, column: c,
                    });
                }
                let val = self.eval_expr(&args[0])?;
                Ok(Some(Value::String(val.to_string_value())))
            }
            _ => Ok(None),
        }
    }

    fn call_user_fn(&mut self, fn_decl: &FnDecl, args: Vec<Value>, line: usize, col: usize) -> Result<Value, RuntimeError> {
        if args.len() != fn_decl.params.len() {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "function `{}` expects {} arguments, got {}",
                    fn_decl.name, fn_decl.params.len(), args.len(),
                ),
                line, column: col,
            });
        }
        self.env.push_scope();
        for (param, arg) in fn_decl.params.iter().zip(args) {
            self.env.define(param.name.clone(), arg);
        }
        let result = match self.exec_block(&fn_decl.body) {
            Ok(Some(val)) => Ok(val),
            Ok(None) => Ok(Value::Null),
            Err(RuntimeError::Return(val)) => Ok(*val),
            Err(e) => Err(e),
        };
        self.env.pop_scope();
        result
    }
}

// ─── VALUE HELPERS ───────────────────────────────────────────────

fn literal_to_value(lit: &Literal) -> Value {
    match lit {
        Literal::Int(i)         => Value::Int(*i),
        Literal::Float(f)       => Value::Float(*f),
        Literal::Scientific(f)  => Value::Float(*f),
        Literal::String(s)      => Value::String(s.clone()),
        Literal::Bool(b)        => Value::Bool(*b),
        Literal::Null           => Value::Null,
        Literal::List(_)        => Value::List(vec![]),
        Literal::Map(_)         => Value::Null,
    }
}

fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Bool(b) => *b,
        Value::Null    => false,
        Value::Int(i)  => *i != 0,
        _              => true,
    }
}

fn eval_binop(l: &Value, r: &Value, op: &BinOp, line: usize, col: usize) -> Result<Value, RuntimeError> {
    match op {
        BinOp::Eq => {
            if l.type_name() != r.type_name() {
                return Ok(Value::Bool(false));
            }
            Ok(Value::Bool(l == r))
        }
        BinOp::NotEq => {
            if l.type_name() != r.type_name() {
                return Ok(Value::Bool(true));
            }
            Ok(Value::Bool(l != r))
        }
        BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => {
            let ord = match (l, r) {
                (Value::Int(a), Value::Int(b)) => Some(a.cmp(b)),
                (Value::Float(a), Value::Float(b)) => Some(a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)),
                (Value::String(a), Value::String(b)) => Some(a.cmp(b)),
                _ => None,
            };
            match ord {
                Some(cmp) => Ok(Value::Bool(match op {
                    BinOp::Lt => cmp.is_lt(),
                    BinOp::Gt => cmp.is_gt(),
                    BinOp::LtEq => cmp.is_le(),
                    BinOp::GtEq => cmp.is_ge(),
                    _ => unreachable!(),
                })),
                None => Err(RuntimeError::TypeError {
                    message: format!("cannot compare {} with {}", l.type_name(), r.type_name()),
                    line, column: col,
                }),
            }
        }
        _ => {
            match (l, r) {
                (Value::Int(a), Value::Int(b)) => {
                    let result = match op {
                        BinOp::Add => Value::Int(a + b),
                        BinOp::Sub => Value::Int(a - b),
                        BinOp::Mul => Value::Int(a * b),
                        BinOp::Div => {
                            if *b == 0 { return Err(RuntimeError::DivisionByZero { line, column: col }); }
                            Value::Int(a / b)
                        }
                        BinOp::Rem => {
                            if *b == 0 { return Err(RuntimeError::DivisionByZero { line, column: col }); }
                            Value::Int(a % b)
                        }
                        BinOp::And => if is_truthy(l) { Value::Int(*b) } else { Value::Int(*a) },
                        BinOp::Or => if is_truthy(l) { Value::Int(*a) } else { Value::Int(*b) },
                        _ => return Err(RuntimeError::TypeError {
                            message: format!("operator {:?} not supported for Int", op),
                            line, column: col,
                        }),
                    };
                    Ok(result)
                }
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(match op {
                    BinOp::Add => a + b,
                    BinOp::Sub => a - b,
                    BinOp::Mul => a * b,
                    BinOp::Div => {
                        if *b == 0.0 { return Err(RuntimeError::DivisionByZero { line, column: col }); }
                        a / b
                    }
                    _ => return Err(RuntimeError::TypeError {
                        message: format!("operator {:?} not supported for Float", op),
                        line, column: col,
                    }),
                })),
                (Value::Int(a), Value::Float(b)) => {
                    let af = *a as f64;
                    Ok(Value::Float(match op {
                        BinOp::Add => af + b,
                        BinOp::Sub => af - b,
                        BinOp::Mul => af * b,
                        BinOp::Div => {
                            if *b == 0.0 { return Err(RuntimeError::DivisionByZero { line, column: col }); }
                            af / b
                        }
                        _ => return Err(RuntimeError::TypeError {
                            message: "type mismatch".into(), line, column: col,
                        }),
                    }))
                }
                (Value::Float(a), Value::Int(b)) => {
                    let bf = *b as f64;
                    Ok(Value::Float(match op {
                        BinOp::Add => a + bf,
                        BinOp::Sub => a - bf,
                        BinOp::Mul => a * bf,
                        BinOp::Div => {
                            if bf == 0.0 { return Err(RuntimeError::DivisionByZero { line, column: col }); }
                            a / bf
                        }
                        _ => return Err(RuntimeError::TypeError {
                            message: "type mismatch".into(), line, column: col,
                        }),
                    }))
                }
                (Value::String(a), Value::String(b)) => match op {
                    BinOp::Add => Ok(Value::String(format!("{a}{b}"))),
                    _ => Err(RuntimeError::TypeError {
                        message: format!("operator {:?} not supported for String", op),
                        line, column: col,
                    }),
                },
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(match op {
                    BinOp::And => *a && *b,
                    BinOp::Or  => *a || *b,
                    _ => return Err(RuntimeError::TypeError {
                        message: format!("operator {:?} not supported for Bool", op),
                        line, column: col,
                    }),
                })),
                _ => Err(RuntimeError::TypeError {
                    message: format!("cannot apply {:?} to {} and {}", op, l.type_name(), r.type_name()),
                    line, column: col,
                }),
            }
        }
    }
}

fn eval_unary(val: &Value, op: &UnaryOp, line: usize, col: usize) -> Result<Value, RuntimeError> {
    match op {
        UnaryOp::Neg => match val {
            Value::Int(i) => Ok(Value::Int(-i)),
            Value::Float(f) => Ok(Value::Float(-f)),
            _ => Err(RuntimeError::TypeError {
                message: format!("cannot negate {}", val.type_name()),
                line, column: col,
            }),
        },
        UnaryOp::Not => Ok(Value::Bool(!is_truthy(val))),
    }
}

// ─── FORMAT VALUE (f-string format spec) ────────────────────────

fn format_value(val: &Value, fmt: Option<&str>) -> String {
    match (val, fmt) {
        (Value::Float(f), Some(fmt_spec)) => {
            if let Some(prec_str) = fmt_spec.strip_prefix('.').and_then(|s| s.strip_suffix('f')) {
                if let Ok(prec) = prec_str.parse::<usize>() {
                    return format!("{:.prec$}", f, prec = prec);
                }
            }
            f.to_string()
        }
        (Value::Int(i), Some("d")) => i.to_string(),
        (Value::Int(i), Some("x")) => format!("{i:x}"),
        (Value::Int(i), Some("X")) => format!("{i:X}"),
        (Value::Int(i), Some("b")) => format!("{i:b}"),
        (Value::Int(i), Some("o")) => format!("{i:o}"),
        _ => val.to_string_value(),
    }
}

// ─── PUBLIC RUNNER ───────────────────────────────────────────────

pub fn run(program: &Program, source: &str) -> Result<(), RuntimeError> {
    let source_arc = Arc::new(source.to_string());
    for stmt in &program.stmts {
        if let Stmt::AgentDecl(decl) = stmt {
            let definition = Arc::new(AgentDefinition::from_decl(decl));
            let mut instance = AgentInstance {
                definition: definition.clone(),
                env: Environment::new_global(),
                source: source_arc.clone(),
            };

            for fn_decl in &definition.functions {
                instance.env.define(fn_decl.name.clone(), Value::Function(fn_decl.clone()));
            }

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
