use giulia_lexer::token::{Token, SpannedToken};
use giulia_ast::node::*;
use giulia_ast::types::TypeExpr;
use giulia_ast::policies::{ErrorPolicy, EventPolicy, AiPolicy, EventPriorityLevel, ChannelDecl};
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