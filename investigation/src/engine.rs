use chrono::Utc;
use parking_lot::RwLock;
use shared_types::{
    AiRecommendation, AiRecommendationKind, AiSeverity, AttackNarrative, InvestigationReport,
    RootCauseAnalysis, ServiceEventInner,
};
use uuid::Uuid;

use ai_core::{AiEventEmitter, AiResult, LlmProvider, MockLlmProvider};

struct InvestigationState {
    reports: Vec<InvestigationReport>,
}

impl Default for InvestigationState {
    fn default() -> Self {
        Self { reports: Vec::new() }
    }
}

/// AI-driven incident investigation engine.
pub struct AiInvestigationEngine<E: AiEventEmitter> {
    emitter: E,
    llm: MockLlmProvider,
    state: RwLock<InvestigationState>,
}

impl<E: AiEventEmitter> AiInvestigationEngine<E> {
    pub fn new(emitter: E) -> Self {
        Self {
            emitter,
            llm: MockLlmProvider::new(),
            state: RwLock::new(InvestigationState::default()),
        }
    }

    pub fn report_count(&self) -> usize {
        self.state.read().reports.len()
    }

    pub async fn investigate(
        &self,
        tenant_id: Uuid,
        incident_id: Option<Uuid>,
        title: &str,
        evidence: &str,
    ) -> AiResult<InvestigationReport> {
        let _summary = self.llm.complete(&format!("investigate: {evidence}")).await?;
        let report = InvestigationReport {
            id: Uuid::new_v4(),
            tenant_id,
            incident_id,
            title: title.to_string(),
            summary: format!("AI investigation of: {evidence}"),
            severity: AiSeverity::High,
            narrative: AttackNarrative {
                stages: vec!["initial access".into(), "lateral movement".into()],
                techniques: vec!["T1078".into()],
                affected_assets: vec!["host-1".into()],
            },
            root_cause: RootCauseAnalysis {
                primary_cause: "Compromised credentials".into(),
                contributing_factors: vec!["Missing MFA".into()],
                confidence: 0.78,
            },
            recommendations: vec![AiRecommendation {
                id: Uuid::new_v4(),
                tenant_id,
                kind: AiRecommendationKind::Remediation,
                title: "Reset credentials".into(),
                body: "Force password reset for affected accounts.".into(),
                confidence: 0.9,
                generated_at: Utc::now(),
            }],
            generated_at: Utc::now(),
        };

        self.state.write().reports.push(report.clone());
        self.emitter
            .emit(shared_types::ServiceEvent::now(ServiceEventInner::InvestigationCompleted {
                report: report.clone(),
            }));

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_core::CollectingEmitter;

    #[tokio::test]
    async fn investigation_emits_completed_event() {
        let emitter = CollectingEmitter::new();
        let engine = AiInvestigationEngine::new(&emitter);
        engine
            .investigate(Uuid::new_v4(), None, "test", "suspicious login")
            .await
            .unwrap();
        assert_eq!(engine.report_count(), 1);
        assert!(!emitter.drain().is_empty());
    }

    #[tokio::test]
    async fn report_contains_root_cause() {
        let emitter = CollectingEmitter::new();
        let engine = AiInvestigationEngine::new(&emitter);
        let report = engine
            .investigate(Uuid::new_v4(), None, "rca", "beaconing")
            .await
            .unwrap();
        assert!(!report.root_cause.primary_cause.is_empty());
    }
}
