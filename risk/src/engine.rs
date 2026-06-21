use chrono::Utc;
use parking_lot::RwLock;
use shared_types::{AiRiskScore, ServiceEventInner};
use uuid::Uuid;

use ai_core::{AiEventEmitter, AiResult};

struct RiskState {
    scores: Vec<AiRiskScore>,
}

impl Default for RiskState {
    fn default() -> Self {
        Self { scores: Vec::new() }
    }
}

/// Computes and tracks AI-derived tenant risk scores.
pub struct AiRiskEngine<E: AiEventEmitter> {
    emitter: E,
    state: RwLock<RiskState>,
}

impl<E: AiEventEmitter> AiRiskEngine<E> {
    pub fn new(emitter: E) -> Self {
        Self {
            emitter,
            state: RwLock::new(RiskState::default()),
        }
    }

    pub fn latest_score(&self, tenant_id: Uuid) -> Option<f64> {
        self.state
            .read()
            .scores
            .iter()
            .rev()
            .find(|s| s.tenant_id == tenant_id)
            .map(|s| s.score)
    }

    pub fn update_score(
        &self,
        tenant_id: Uuid,
        delta: f64,
        factors: Vec<String>,
    ) -> AiResult<AiRiskScore> {
        let previous = self.latest_score(tenant_id).unwrap_or(0.0);
        let score = (previous + delta).clamp(0.0, 100.0);
        let record = AiRiskScore {
            tenant_id,
            score,
            previous_score: previous,
            factors,
            computed_at: Utc::now(),
        };

        self.state.write().scores.push(record.clone());
        self.emitter
            .emit(shared_types::ServiceEvent::now(ServiceEventInner::AiRiskScoreUpdated {
                score: record.clone(),
            }));

        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_core::CollectingEmitter;

    #[test]
    fn updates_risk_score() {
        let emitter = CollectingEmitter::new();
        let engine = AiRiskEngine::new(&emitter);
        let tenant = Uuid::new_v4();
        engine
            .update_score(tenant, 25.0, vec!["new detections".into()])
            .unwrap();
        assert_eq!(engine.latest_score(tenant), Some(25.0));
    }

    #[test]
    fn emits_ai_risk_score_updated() {
        let emitter = CollectingEmitter::new();
        let engine = AiRiskEngine::new(&emitter);
        engine
            .update_score(Uuid::new_v4(), 10.0, vec![])
            .unwrap();
        assert!(!emitter.drain().is_empty());
    }
}
