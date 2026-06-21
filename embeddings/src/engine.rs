use async_trait::async_trait;
use chrono::Utc;
use parking_lot::RwLock;
use shared_types::{EmbeddingRecord, RagChunk};
use uuid::Uuid;

use ai_core::AiResult;

/// Generates embedding vectors for text chunks.
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> AiResult<Vec<f32>>;
    fn dimensions(&self) -> u32;
    fn model(&self) -> &str;
}

/// Deterministic mock embedding provider.
pub struct MockEmbeddingProvider {
    dimensions: u32,
}

impl MockEmbeddingProvider {
    pub fn new(dimensions: u32) -> Self {
        Self { dimensions }
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbeddingProvider {
    async fn embed(&self, text: &str) -> AiResult<Vec<f32>> {
        let mut vector = Vec::with_capacity(self.dimensions as usize);
        let seed = text.bytes().fold(0u64, |acc, b| acc.wrapping_add(b as u64));
        for i in 0..self.dimensions {
            let v = ((seed.wrapping_mul(i as u64 + 1)) % 1000) as f32 / 1000.0;
            vector.push(v);
        }
        Ok(vector)
    }

    fn dimensions(&self) -> u32 {
        self.dimensions
    }

    fn model(&self) -> &str {
        "mock-embed-v1"
    }
}

struct StoreState {
    records: Vec<EmbeddingRecord>,
}

impl Default for StoreState {
    fn default() -> Self {
        Self {
            records: Vec::new(),
        }
    }
}

/// In-memory embedding store keyed by record id.
pub struct InMemoryEmbeddingStore {
    state: RwLock<StoreState>,
}

impl InMemoryEmbeddingStore {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(StoreState::default()),
        }
    }

    pub fn store(&self, tenant_id: Uuid, _chunk: &RagChunk, vector: Vec<f32>, model: &str) -> EmbeddingRecord {
        let record = EmbeddingRecord {
            id: Uuid::new_v4(),
            tenant_id,
            model: model.to_string(),
            dimensions: vector.len() as u32,
            vector,
            created_at: Utc::now(),
        };
        self.state.write().records.push(record.clone());
        record
    }

    pub fn count(&self) -> usize {
        self.state.read().records.len()
    }

    pub fn all(&self) -> Vec<EmbeddingRecord> {
        self.state.read().records.clone()
    }
}

impl Default for InMemoryEmbeddingStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_embed_produces_vector() {
        let provider = MockEmbeddingProvider::new(8);
        let v = provider.embed("hello").await.unwrap();
        assert_eq!(v.len(), 8);
    }

    #[tokio::test]
    async fn store_persists_record() {
        let store = InMemoryEmbeddingStore::new();
        let chunk = RagChunk {
            id: Uuid::new_v4(),
            document_id: Uuid::new_v4(),
            content: "test".into(),
            embedding_id: None,
            chunk_index: 0,
        };
        store.store(Uuid::new_v4(), &chunk, vec![0.1; 4], "mock");
        assert_eq!(store.count(), 1);
    }
}
