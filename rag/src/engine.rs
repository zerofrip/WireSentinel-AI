use chrono::Utc;
use embeddings::{EmbeddingProvider, InMemoryEmbeddingStore, MockEmbeddingProvider};
use parking_lot::RwLock;
use shared_types::{RagChunk, RagDocument, RagRetrievalResult};
use uuid::Uuid;

use ai_core::AiResult;

const CHUNK_SIZE: usize = 256;

struct RagState {
    documents: Vec<RagDocument>,
    chunks: Vec<RagChunk>,
}

impl Default for RagState {
    fn default() -> Self {
        Self {
            documents: Vec::new(),
            chunks: Vec::new(),
        }
    }
}

/// Retrieval-augmented generation over security documents.
pub struct SecurityRagEngine {
    embedder: MockEmbeddingProvider,
    store: InMemoryEmbeddingStore,
    state: RwLock<RagState>,
}

impl SecurityRagEngine {
    pub fn new() -> Self {
        Self {
            embedder: MockEmbeddingProvider::new(16),
            store: InMemoryEmbeddingStore::new(),
            state: RwLock::new(RagState::default()),
        }
    }

    pub fn document_count(&self) -> usize {
        self.state.read().documents.len()
    }

    pub fn chunk_count(&self) -> usize {
        self.state.read().chunks.len()
    }

    pub async fn index_document(
        &self,
        tenant_id: Uuid,
        title: &str,
        source: &str,
        content: &str,
    ) -> AiResult<RagDocument> {
        let document = RagDocument {
            id: Uuid::new_v4(),
            tenant_id,
            title: title.to_string(),
            source: source.to_string(),
            indexed_at: Utc::now(),
        };

        for (idx, piece) in content.as_bytes().chunks(CHUNK_SIZE).enumerate() {
            let text = String::from_utf8_lossy(piece).to_string();
            let chunk = RagChunk {
                id: Uuid::new_v4(),
                document_id: document.id,
                content: text.clone(),
                embedding_id: None,
                chunk_index: idx as u32,
            };
            let vector = self.embedder.embed(&text).await?;
            let record = self.store.store(tenant_id, &chunk, vector, self.embedder.model());
            let mut stored = chunk;
            stored.embedding_id = Some(record.id);
            self.state.write().chunks.push(stored);
        }

        self.state.write().documents.push(document.clone());
        Ok(document)
    }

    pub async fn retrieve(&self, query: &str, limit: usize) -> AiResult<Vec<RagRetrievalResult>> {
        let query_vec = self.embedder.embed(query).await?;
        let mut scored: Vec<RagRetrievalResult> = self
            .state
            .read()
            .chunks
            .iter()
            .filter_map(|chunk| {
                let record = self
                    .store
                    .all()
                    .into_iter()
                    .find(|r| Some(r.id) == chunk.embedding_id)?;
                let score = cosine(&query_vec, &record.vector);
                Some(RagRetrievalResult {
                    chunk: chunk.clone(),
                    score,
                })
            })
            .collect();

        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);
        Ok(scored)
    }
}

impl Default for SecurityRagEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn cosine(a: &[f32], b: &[f32]) -> f64 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        (dot / (na * nb)) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn indexes_and_retrieves_chunks() {
        let engine = SecurityRagEngine::new();
        engine
            .index_document(
                Uuid::new_v4(),
                "runbook",
                "wiki",
                "isolate host and reset credentials for compromised accounts",
            )
            .await
            .unwrap();
        assert_eq!(engine.document_count(), 1);
        assert!(engine.chunk_count() >= 1);
        let hits = engine.retrieve("reset credentials", 3).await.unwrap();
        assert!(!hits.is_empty());
    }

    #[tokio::test]
    async fn empty_query_returns_results_vector() {
        let engine = SecurityRagEngine::new();
        let hits = engine.retrieve("nothing", 1).await.unwrap();
        assert!(hits.is_empty());
    }
}
