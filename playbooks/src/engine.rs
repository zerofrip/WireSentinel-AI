use chrono::Utc;
use parking_lot::RwLock;
use shared_types::{PlaybookKind, PlaybookStepSuggestion, PlaybookSuggestion, ServiceEventInner};
use uuid::Uuid;

use ai_core::{AiEventEmitter, AiResult};

struct PlaybookState {
    suggestions: Vec<PlaybookSuggestion>,
}

impl Default for PlaybookState {
    fn default() -> Self {
        Self { suggestions: Vec::new() }
    }
}

/// Generates XDR-compatible playbook suggestions.
pub struct PlaybookAssistant<E: AiEventEmitter> {
    emitter: E,
    state: RwLock<PlaybookState>,
}

impl<E: AiEventEmitter> PlaybookAssistant<E> {
    pub fn new(emitter: E) -> Self {
        Self {
            emitter,
            state: RwLock::new(PlaybookState::default()),
        }
    }

    pub fn suggestion_count(&self) -> usize {
        self.state.read().suggestions.len()
    }

    pub fn suggest(&self, tenant_id: Uuid, incident_title: &str) -> AiResult<PlaybookSuggestion> {
        let steps = vec![
            PlaybookStepSuggestion {
                step_order: 1,
                kind: PlaybookKind::NotifyTeam,
                description: format!("Notify SOC about {incident_title}"),
                automated: true,
            },
            PlaybookStepSuggestion {
                step_order: 2,
                kind: PlaybookKind::QuarantineDevice,
                description: "Quarantine affected endpoint".into(),
                automated: false,
            },
            PlaybookStepSuggestion {
                step_order: 3,
                kind: PlaybookKind::EscalateIncident,
                description: "Escalate if containment fails".into(),
                automated: true,
            },
        ];

        let suggestion = PlaybookSuggestion {
            id: Uuid::new_v4(),
            tenant_id,
            title: format!("Response playbook for {incident_title}"),
            steps,
            confidence: 0.8,
            generated_at: Utc::now(),
        };

        self.state.write().suggestions.push(suggestion.clone());
        self.emitter.emit(shared_types::ServiceEvent::now(
            ServiceEventInner::AiRecommendationGenerated {
                recommendation: shared_types::AiRecommendation {
                    id: suggestion.id,
                    tenant_id,
                    kind: shared_types::AiRecommendationKind::Playbook,
                    title: suggestion.title.clone(),
                    body: format!("{} steps", suggestion.steps.len()),
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

    #[test]
    fn playbook_uses_xdr_kinds() {
        let emitter = CollectingEmitter::new();
        let assistant = PlaybookAssistant::new(&emitter);
        let suggestion = assistant
            .suggest(Uuid::new_v4(), "ransomware")
            .unwrap();
        assert_eq!(suggestion.steps[0].kind, PlaybookKind::NotifyTeam);
        assert_eq!(suggestion.steps.len(), 3);
    }

    #[test]
    fn emits_recommendation() {
        let emitter = CollectingEmitter::new();
        let assistant = PlaybookAssistant::new(&emitter);
        assistant.suggest(Uuid::new_v4(), "phishing").unwrap();
        assert!(!emitter.drain().is_empty());
    }
}
