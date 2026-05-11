#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    Int,
    Float,
    String,
    Bool,
    Null,
    List(Box<TypeExpr>),
    Map(Box<TypeExpr>, Box<TypeExpr>),
    Named(String),
    Optional(Box<TypeExpr>),
}
