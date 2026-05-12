use crate::policies::{AiPolicy, ChannelDecl, ErrorPolicy, EventPolicy, EventPriorityLevel};
use crate::types::TypeExpr;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Span {
    pub start: usize,
    pub end:   usize,
}

// ─── PROGRAMA ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Program {
    pub stmts: Vec<Stmt>,
    pub span:  Span,
}

// ─── STATEMENTS ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Stmt {
    AgentDecl(AgentDecl),
    FnDecl(FnDecl),
    LetStmt(LetStmt),
    AssignStmt(AssignStmt),
    IfStmt(IfStmt),
    WhileStmt(WhileStmt),
    ForStmt(ForStmt),
    ReturnStmt(ReturnStmt),
    EffectStmt(EffectStmt),
    SendStmt(SendStmt),
    ExprStmt(ExprStmt),
}

#[derive(Debug, Clone)]
pub struct AgentDecl {
    pub name:         String,
    pub version:      Option<u64>,

    pub capabilities: Vec<UseDecl>,
    pub handlers:     Vec<HandlerDecl>,
    pub functions:    Vec<FnDecl>,

    pub error_policy: Option<ErrorPolicy>,
    pub event_policy: Option<EventPolicy>,
    pub ai_policy:    Option<AiPolicy>,
    pub channels:     Vec<ChannelDecl>,

    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct UseDecl {
    pub capability: String,
    pub span:       Span,
}

#[derive(Debug, Clone)]
pub struct HandlerDecl {
    pub event:      EventPattern,
    pub priority:   EventPriorityLevel,
    pub concurrent: bool,
    pub body:       Block,
    pub span:       Span,
}

#[derive(Debug, Clone)]
pub enum EventPattern {
    Start,
    Stop,
    Timer(Expr),
    Message(MessageTarget),
    Speech,
    Image,
    SensorChange(String),
    Network(String),
    Idle(Expr),
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum MessageTarget {
    Simple(String),
    Typed { agent: String, channel: String },
}

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name:        String,
    pub params:      Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub body:        Block,
    pub span:        Span,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name:            String,
    pub type_annotation: Option<TypeExpr>,
    pub span:            Span,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span:  Span,
}

// ─── STATEMENTS CONCRETOS ────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct LetStmt {
    pub name:     String,
    pub type_ann: Option<TypeExpr>,
    pub value:    Expr,
    pub span:     Span,
}

#[derive(Debug, Clone)]
pub struct AssignStmt {
    pub name:  String,
    pub value: Expr,
    pub span:  Span,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition:   Expr,
    pub then_branch: Block,
    pub else_branch: Option<Box<ElseBranch>>,
    pub span:        Span,
}

#[derive(Debug, Clone)]
pub enum ElseBranch {
    Block(Block),
    If(IfStmt),
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body:      Block,
    pub span:      Span,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub var:      String,
    pub iterable: Expr,
    pub body:     Block,
    pub span:     Span,
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub value: Option<Expr>,
    pub span:  Span,
}

#[derive(Debug, Clone)]
pub struct EffectStmt {
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SendStmt {
    pub target_agent: String,
    pub channel:      Option<String>,
    pub topic:        Option<String>,
    pub payload:      Expr,
    pub span:         Span,
}

#[derive(Debug, Clone)]
pub struct ExprStmt {
    pub expr: Expr,
    pub span: Span,
}

// ─── EXPRESSÕES ──────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal, Span),
    Identifier(String, Span),
    BinOp {
        op:    BinOp,
        left:  Box<Expr>,
        right: Box<Expr>,
        span:  Span,
    },
    UnaryOp {
        op:      UnaryOp,
        operand: Box<Expr>,
        span:    Span,
    },
    Call {
        callee: Box<Expr>,
        args:   Vec<Expr>,
        span:   Span,
    },
    FieldAccess {
        object: Box<Expr>,
        field:  String,
        span:   Span,
    },
    Index {
        object: Box<Expr>,
        index:  Box<Expr>,
        span:   Span,
    },
    List(Vec<Expr>, Span),
    Map(Vec<(Expr, Expr)>, Span),
    FString(Vec<FStringSegment>, Span),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(_, s)          => *s,
            Expr::Identifier(_, s)       => *s,
            Expr::BinOp    { span, .. }  => *span,
            Expr::UnaryOp  { span, .. }  => *span,
            Expr::Call     { span, .. }  => *span,
            Expr::FieldAccess { span, .. } => *span,
            Expr::Index    { span, .. }  => *span,
            Expr::List     (_, s)        => *s,
            Expr::Map      (_, s)        => *s,
            Expr::FString  (_, s)        => *s,
        }
    }
}

// ─── F-STRING ────────────────────────────────────────────────────

/// Segmento de uma f-string: texto literal ou expressão com format spec opcional.
#[derive(Debug, Clone, PartialEq)]
pub enum FStringSegment {
    Lit(String),
    Expr(Expr, Option<String>),
}

// ─── LITERAIS ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Scientific(f64),
    String(String),
    Bool(bool),
    Null,
    List(Vec<Expr>),
    Map(Vec<(Expr, Expr)>),
}

// ─── OPERADORES ──────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Rem,
    Eq, NotEq, Lt, Gt, LtEq, GtEq,
    And, Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}
