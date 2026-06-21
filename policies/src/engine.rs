use chrono::Utc;
use parking_lot::RwLock;
use shared_types::{AiSeverity, PolicySuggestion, ServiceEventInner};
use uuid::Uuid;

use ai_core::{AiEventEmitter, AiResult, LlmProvider, MockLlmProvider};

struct PolicyState {
    suggestions: Vec<PolicySuggestion>,
}

impl Default for PolicyState {
    fn default() -> Self {
        Self { suggestions: Vec::new() }
    }
}

/// Generates AI-assisted security policy suggestions.
pub struct PolicyAssistant<E: AiEventEmitter> {
    emitter: E,
    llm: MockLlmProvider,
    state: RwLock<PolicyState>,
}

impl<E: AiEventEmitter> PolicyAssistant<E> {
    pub fn new(emitter: E) -> Self {
        Self {
            emitter,
            llm: MockLlmProvider::new(),
            state: RwLock::new(PolicyState::default()),
        }
    }

    pub fn suggestion_count(&self) -> usize {
        self.state.read().suggestions.len()
    }

    pub async fn suggest(&self, tenant_id: Uuid, gap: &str) -> AiResult<PolicySuggestion> {
        let body = self.llm.complete(&format!("policy for: {gap}")).await?;
        let suggestion = PolicySuggestion {
            id: Uuid::new_v4(),
            tenant_id,
            title: format!("Policy improvement for {gap}"),
            policy_body: body,
            severity: AiSeverity::Medium,
            confidence: 0.7,
            rationale: "AI-generated from compliance gap analysis".into(),
            generated_at: Utc::now(),
        };

        self.state.write().suggestions.push(suggestion.clone());
        self.emitter.emit(shared_types::ServiceEvent::now(
            ServiceEventInner::AiRecommendationGenerated {
                recommendation: shared_types::AiRecommendation {
                    id: suggestion.id,
                    tenant_id,
                    kind: shared_types::AiRecommendationKind::Policy,
                    title: suggestion.title.clone(),
                    body: suggestion.policy_body.clone(),
                    confidence: suggestion.confidence,
                    generated_at: suggestion.generated_at,
                },
            },
        ));

        Ok(suggestion)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_core::CollectingEmitter;

    #[tokio::test]
    async fn creates_policy_suggestion() {
        let emitter = CollectingEmitter::new();
        let assistant = PolicyAssistant::new(&emitter);
        assistant.suggest(Uuid::new_v4(), "MFA gap").await.unwrap();
        assert_eq!(assistant.suggestion_count(), 1);
    }

    #[tokio::test]
    async fn emits_event_on_suggest() {
        let emitter = CollectingEmitter::new();
        let assistant = PolicyAssistant::new(&emitter);
        assistant.suggest(Uuid::new_v4(), "DLP").await.unwrap();
        assert!(!emitter.drain().is_empty());
    }
}
