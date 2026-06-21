use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiError {
    #[error("llm error: {0}")]
    Llm(String),
    #[error("embedding error: {0}")]
    Embedding(String),
    #[error("copilot error: {0}")]
    Copilot(String),
    #[error("investigation error: {0}")]
    Investigation(String),
    #[error("correlation error: {0}")]
    Correlation(String),
    #[error("knowledge graph error: {0}")]
    KnowledgeGraph(String),
    #[error("rag error: {0}")]
    Rag(String),
    #[error("security error: {0}")]
    Security(String),
    #[error("provider error: {0}")]
    Provider(String),
    #[error("{0}")]
    Other(String),
}

pub type AiResult<T> = std::result::Result<T, AiError>;
