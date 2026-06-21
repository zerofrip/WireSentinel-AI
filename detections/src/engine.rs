use chrono::Utc;
use parking_lot::RwLock;
use shared_types::{AiSeverity, DetectionSuggestion, ServiceEventInner};
use uuid::Uuid;

use ai_core::{AiEventEmitter, AiResult, LlmProvider, MockLlmProvider};

struct DetectionState {
    suggestions: Vec<DetectionSuggestion>,
}

impl Default for DetectionState {
    fn default() -> Self {
        Self { suggestions: Vec::new() }
    }
}

/// Generates AI-assisted detection rule suggestions.
pub struct DetectionAssistant<E: AiEventEmitter> {
    emitter: E,
    llm: MockLlmProvider,
    state: RwLock<DetectionState>,
}

impl<E: AiEventEmitter> DetectionAssistant<E> {
    pub fn new(emitter: E) -> Self {
        Self {
            emitter,
            llm: MockLlmProvider::new(),
            state: RwLock::new(DetectionState::default()),
        }
    }

    pub fn suggestion_count(&self) -> usize {
        self.state.read().suggestions.len()
    }

    pub async fn suggest(&self, tenant_id: Uuid, context: &str) -> AiResult<DetectionSuggestion> {
        let body = self.llm.complete(&format!("detect rule for: {context}")).await?;
        let suggestion = DetectionSuggestion {
            id: Uuid::new_v4(),
            tenant_id,
            title: "AI detection suggestion".into(),
            rule_body: body,
            severity: AiSeverity::Medium,
            confidence: 0.72,
            rationale: format!("Derived from context: {context}"),
            generated_at: Utc::now(),
        };

        self.state.write().suggestions.push(suggestion.clone());
        self.emitter.emit(shared_types::ServiceEvent::now(
            ServiceEventInner::AiRecommendationGenerated {
                recommendation: shared_types::AiRecommendation {
                    id: suggestion.id,
                    tenant_id,
                    kind: shared_types::AiRecommendationKind::Detection,
                    title: suggestion.title.clone(),
                    body: suggestion.rule_body.clone(),
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
    async fn generates_detection_suggestion() {
        let emitter = CollectingEmitter::new();
        let assistant = DetectionAssistant::new(&emitter);
        assistant
            .suggest(Uuid::new_v4(), "failed logins")
            .await
            .unwrap();
        assert_eq!(assistant.suggestion_count(), 1);
    }

    #[tokio::test]
    async fn emits_recommendation_event() {
        let emitter = CollectingEmitter::new();
        let assistant = DetectionAssistant::new(&emitter);
        assistant.suggest(Uuid::new_v4(), "beaconing").await.unwrap();
        assert!(!emitter.drain().is_empty());
    }
}
