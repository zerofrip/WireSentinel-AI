use chrono::Utc;
use parking_lot::RwLock;
use shared_types::{
    AiIntentKind, AiProvider, AiSecurityPolicy, CopilotQuery, CopilotResponse, ServiceEventInner,
};

use ai_core::{AiEventEmitter, AiResult, AiSecurityPolicyEngine, LlmProvider, MockLlmProvider};

struct CopilotState {
    queries: u64,
}

impl Default for CopilotState {
    fn default() -> Self {
        Self { queries: 0 }
    }
}

/// Natural-language security copilot.
pub struct SecurityCopilot<E: AiEventEmitter> {
    emitter: E,
    policy_engine: AiSecurityPolicyEngine,
    llm: MockLlmProvider,
    state: RwLock<CopilotState>,
}

impl<E: AiEventEmitter> SecurityCopilot<E> {
    pub fn new(emitter: E) -> Self {
        Self {
            emitter,
            policy_engine: AiSecurityPolicyEngine::new(),
            llm: MockLlmProvider::new(),
            state: RwLock::new(CopilotState::default()),
        }
    }

    pub fn query_count(&self) -> u64 {
        self.state.read().queries
    }

    pub async fn query(
        &self,
        policy: &AiSecurityPolicy,
        query: CopilotQuery,
    ) -> AiResult<CopilotResponse> {
        self.policy_engine.validate_prompt(&self.emitter, policy, &query.prompt)?;
        self.policy_engine
            .validate_provider(&self.emitter, policy, AiProvider::Mock)?;

        let intent = classify_intent(&query.prompt);
        let answer = self.llm.complete(&query.prompt).await?;
        let response = CopilotResponse {
            query_id: query.id,
            intent,
            answer,
            confidence: 0.85,
            provider: AiProvider::Mock,
            generated_at: Utc::now(),
        };

        self.state.write().queries += 1;
        self.emitter
            .emit(shared_types::ServiceEvent::now(ServiceEventInner::CopilotQueryExecuted {
                response: response.clone(),
            }));

        Ok(response)
    }
}

pub fn classify_intent(prompt: &str) -> AiIntentKind {
    let lower = prompt.to_lowercase();
    if lower.contains("investigate") || lower.contains("incident") {
        AiIntentKind::Investigate
    } else if lower.contains("detect") || lower.contains("rule") {
        AiIntentKind::Detect
    } else if lower.contains("policy") {
        AiIntentKind::Policy
    } else if lower.contains("threat") || lower.contains("intel") {
        AiIntentKind::ThreatIntel
    } else if lower.contains("playbook") {
        AiIntentKind::Playbook
    } else if lower.contains("report") {
        AiIntentKind::Report
    } else {
        AiIntentKind::General
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_core::CollectingEmitter;
    use uuid::Uuid;

    #[test]
    fn classifies_investigation_intent() {
        assert_eq!(
            classify_intent("please investigate this alert"),
            AiIntentKind::Investigate
        );
    }

    #[tokio::test]
    async fn query_emits_event() {
        let emitter = CollectingEmitter::new();
        let copilot = SecurityCopilot::new(&emitter);
        let policy = AiSecurityPolicy::default();
        let query = CopilotQuery {
            id: Uuid::new_v4(),
            tenant_id: policy.tenant_id,
            user_id: Uuid::new_v4(),
            prompt: "investigate suspicious login".into(),
            context_ids: vec![],
            submitted_at: Utc::now(),
        };
        copilot.query(&policy, query).await.unwrap();
        assert_eq!(copilot.query_count(), 1);
        assert!(!emitter.drain().is_empty());
    }
}
