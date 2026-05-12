use crate::node::Span;

use crate::types::TypeExpr;

#[derive(Debug, Clone, Default)]
pub struct ErrorPolicy {
    pub strategy:    Option<String>,
    pub max_retries: Option<u32>,
    pub backoff_ms:  Option<u64>,
    pub on_exhaust:  Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct EventPolicy {
    pub on_overflow:    Option<String>,
    pub critical_queue: Option<usize>,
    pub normal_queue:   Option<usize>,
    pub low_queue:      Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct AiPolicy {
    pub timeout_ms:        Option<u64>,
    pub max_queue_during:  Option<usize>,
    pub on_timeout:        Option<String>,
    pub fallback_response: Option<String>,
    pub cache_identical:   Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum EventPriorityLevel {
    Critical,
    High,
    #[default]
    Normal,
    Low,
}

#[derive(Debug, Clone)]
pub struct ChannelDecl {
    pub name:      String,
    pub chan_type: TypeExpr,
    pub span:      Span,
}
