use crate::types::TypeExpr;

//posicao no source code, presente em todos os nos
//sem span na ha mensagens de erro uteis

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Span{
    pub start:  usize,
    pub end:    usize,
}

//PROGRAMA

#[derive(Debug, Clone)]
pub struct Program{
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

//STATEMENTS

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
    EffectStmt(EffectStmt),//'do expr' marcardor de side effect(Fase 4, sintaxe existe na fase 1, mas é apenas um marcador)
    ExprStmt(ExprStmt),
}

#[derive(Debug, Clone)]
pub struct AgentDecl{
    pub name:       String,
    pub uses:       Vec<UseDecl>, //capbilities usadas pelo agente
    pub handlers:   Vec<HandlerDecl>, //funcoes que o agente pode executar
    pub fns:        Vec<FnDecl>, //funcoes auxiliares do agente
    pub span:       Span,
}

#[derive(Debug, Clone)]
pub struct UseDecl {
    pub capability: String,
    pub span:       Span,
}

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HandlerDecl {
    pub event:  EventPattern,
    pub body:   Block,
    pub span:   Span,
}

#[derive(Debug, Clone)]
pub enum EventPattern{
    Start, 
    Stop, 
    Timer(Expr), //on timer(1000)
    Message(String), //on message("topic")
    Custom(String), //on my_event
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name:               String,
    pub type_annotation:    Option<TypeExpr>,
    pub span:               Span,
}   

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,    
}

#[derive(Debug, Clone)]
pub struct LetStmt {
    pub name: String,
    pub type_ann: Option<TypeExpr>,
    pub value: Expr, 
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AssignStmt {
    pub name: String,
    pub value: Expr,
    pub span: Span,    
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
    pub body: Block,
    pub span: Span,        
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
    pub span: Span,        
}

#[derive(Debug, Clone)]
pub struct EffectStmt {
    pub expr: Expr,
    pub span: Span,    
}

#[derive(Debug, Clone)]
pub struct ExprStmt {
    pub expr: Expr,
    pub span: Span,    
}

//EXPRESSOES

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
}

impl Expr {
    /// Retorna o Span do nó raiz desta expressão.
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(_, s)          => *s,
            Expr::Identifier(_, s)       => *s,
            Expr::BinOp    { span, .. }  => *span,
            Expr::UnaryOp  { span, .. }  => *span,
            Expr::Call     { span, .. }  => *span,
            Expr::FieldAccess { span, .. }=> *span,
            Expr::Index    { span, .. }  => *span,
            Expr::List     (_, s)        => *s,
            Expr::Map      (_, s)        => *s,
        }
    }
}


//LITERAIS
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    List(Vec<Expr>),
    Map(Vec<(Expr, Expr)>),
}

//OPERADORES

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Rem,
    Eq, NotEq,
    Lt, Gt, LtEq, GtEq,
    And, Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,  // -x
    Not,  // not x
}