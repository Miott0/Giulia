use giulia_ast::node::FnDecl;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Null,
    List(Vec<Value>),
    Function(FnDecl),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a),    Value::Int(b))    => a == b,
            (Value::Float(a),  Value::Float(b))  => a.to_bits() == b.to_bits(),
            (Value::Bool(a),   Value::Bool(b))   => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Null,      Value::Null)      => true,
            (Value::List(a),   Value::List(b))   => a == b,
            _ => false,
        }
    }
}

impl Value {
    pub fn type_name(&self) -> String {
        match self {
            Value::Int(_)      => "Int".into(),
            Value::Float(_)    => "Float".into(),
            Value::Bool(_)     => "Bool".into(),
            Value::String(_)   => "String".into(),
            Value::Null        => "Null".into(),
            Value::List(_)     => "List".into(),
            Value::Function(_) => "Function".into(),
        }
    }

    pub fn to_string_value(&self) -> String {
        match self {
            Value::Int(i)      => i.to_string(),
            Value::Float(f)    => f.to_string(),
            Value::Bool(b)     => b.to_string(),
            Value::String(s)   => s.clone(),
            Value::Null        => "null".into(),
            Value::List(items) => {
                let elems: Vec<String> = items.iter().map(|v| v.to_string_value()).collect();
                format!("[{}]", elems.join(", "))
            }
            Value::Function(f) => format!("<fn {}>", f.name),
        }
    }
}
