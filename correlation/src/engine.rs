use chrono::Utc;
use parking_lot::RwLock;
use shared_types::{AiSeverity, CorrelatedThreat, ServiceEventInner};
use uuid::Uuid;

use ai_core::{AiEventEmitter, AiResult};

struct CorrelationState {
    threats: Vec<CorrelatedThreat>,
}

impl Default for CorrelationState {
    fn default() -> Self {
        Self { threats: Vec::new() }
    }
}

/// Correlates related security events into unified threats.
pub struct ThreatCorrelationEngine<E: AiEventEmitter> {
    emitter: E,
    state: RwLock<CorrelationState>,
}

impl<E: AiEventEmitter> ThreatCorrelationEngine<E> {
    pub fn new(emitter: E) -> Self {
        Self {
            emitter,
            state: RwLock::new(CorrelationState::default()),
        }
    }

    pub fn threat_count(&self) -> usize {
        self.state.read().threats.len()
    }

    pub fn correlate(
        &self,
        tenant_id: Uuid,
        title: &str,
        source_events: Vec<Uuid>,
    ) -> AiResult<CorrelatedThreat> {
        if source_events.len() < 2 {
            return Err(ai_core::AiError::Correlation(
                "need at least two source events".into(),
            ));
        }

        let score = (source_events.len() as f64 * 0.25).min(1.0);
        let threat = CorrelatedThreat {
            id: Uuid::new_v4(),
            tenant_id,
            title: title.to_string(),
            severity: if score > 0.5 {
                AiSeverity::High
            } else {
                AiSeverity::Medium
            },
            source_events,
            correlation_score: score,
            detected_at: Utc::now(),
        };

        self.state.write().threats.push(threat.clone());
        self.emitter
            .emit(shared_types::ServiceEvent::now(ServiceEventInner::ThreatCorrelated {
                threat: threat.clone(),
            }));

        Ok(threat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_core::CollectingEmitter;

    #[test]
    fn requires_multiple_events() {
        let emitter = CollectingEmitter::new();
        let engine = ThreatCorrelationEngine::new(&emitter);
        assert!(engine
            .correlate(Uuid::new_v4(), "solo", vec![Uuid::new_v4()])
            .is_err());
    }

    #[test]
    fn correlates_and_emits() {
        let emitter = CollectingEmitter::new();
        let engine = ThreatCorrelationEngine::new(&emitter);
        engine
            .correlate(
                Uuid::new_v4(),
                "campaign",
                vec![Uuid::new_v4(), Uuid::new_v4()],
            )
            .unwrap();
        assert_eq!(engine.threat_count(), 1);
        assert!(!emitter.drain().is_empty());
    }
}
