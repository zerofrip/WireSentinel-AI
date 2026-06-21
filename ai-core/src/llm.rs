use async_trait::async_trait;
use shared_types::AiProvider;

use crate::{AiError, AiResult};

/// Async LLM completion provider.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, prompt: &str) -> AiResult<String>;
    fn provider(&self) -> AiProvider;
}

/// Deterministic mock LLM for tests and offline mode.
pub struct MockLlmProvider {
    provider: AiProvider,
}

impl MockLlmProvider {
    pub fn new() -> Self {
        Self {
            provider: AiProvider::Mock,
        }
    }
}

impl Default for MockLlmProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmProvider for MockLlmProvider {
    async fn complete(&self, prompt: &str) -> AiResult<String> {
        let lower = prompt.to_lowercase();
        if lower.contains("investigate") {
            Ok("Investigation summary: suspicious lateral movement detected.".into())
        } else if lower.contains("detect") {
            Ok("Suggested detection: monitor privileged login anomalies.".into())
        } else {
            Ok(format!("Mock response for: {}", prompt.chars().take(80).collect::<String>()))
        }
    }

    fn provider(&self) -> AiProvider {
        self.provider
    }
}

/// Validates provider access against tenant policy.
pub fn ensure_provider_allowed(
    policy: &shared_types::AiSecurityPolicy,
    provider: AiProvider,
) -> AiResult<()> {
    if policy
        .allowed_providers
        .iter()
        .any(|p| *p == provider)
    {
        Ok(())
    } else {
        Err(AiError::Provider(format!("{:?} not allowed", provider)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_llm_returns_response() {
        let llm = MockLlmProvider::new();
        let out = llm.complete("investigate incident").await.unwrap();
        assert!(out.contains("Investigation"));
    }

    #[test]
    fn provider_allowed_when_listed() {
        let policy = shared_types::AiSecurityPolicy::default();
        ensure_provider_allowed(&policy, AiProvider::Mock).unwrap();
    }
}
