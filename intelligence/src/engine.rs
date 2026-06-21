use chrono::Utc;
use parking_lot::RwLock;
use shared_types::{AiSeverity, ThreatIntelligenceReport};
use uuid::Uuid;

use ai_core::{AiResult, LlmProvider, MockLlmProvider};

struct IntelState {
    reports: Vec<ThreatIntelligenceReport>,
}

impl Default for IntelState {
    fn default() -> Self {
        Self { reports: Vec::new() }
    }
}

/// Enriches indicators with AI-generated threat intelligence.
pub struct ThreatIntelAssistant {
    llm: MockLlmProvider,
    state: RwLock<IntelState>,
}

impl ThreatIntelAssistant {
    pub fn new() -> Self {
        Self {
            llm: MockLlmProvider::new(),
            state: RwLock::new(IntelState::default()),
        }
    }

    pub fn report_count(&self) -> usize {
        self.state.read().reports.len()
    }

    pub async fn enrich(
        &self,
        tenant_id: Uuid,
        indicator: &str,
        indicator_kind: &str,
    ) -> AiResult<ThreatIntelligenceReport> {
        let summary = self
            .llm
            .complete(&format!("threat intel for {indicator_kind}: {indicator}"))
            .await?;
        let report = ThreatIntelligenceReport {
            id: Uuid::new_v4(),
            tenant_id,
            indicator: indicator.to_string(),
            indicator_kind: indicator_kind.to_string(),
            summary,
            severity: AiSeverity::High,
            sources: vec!["mock-feed".into()],
            generated_at: Utc::now(),
        };
        self.state.write().reports.push(report.clone());
        Ok(report)
    }
}

impl Default for ThreatIntelAssistant {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn enriches_indicator() {
        let assistant = ThreatIntelAssistant::new();
        assistant
            .enrich(Uuid::new_v4(), "203.0.113.5", "ip")
            .await
            .unwrap();
        assert_eq!(assistant.report_count(), 1);
    }

    #[tokio::test]
    async fn report_has_summary() {
        let assistant = ThreatIntelAssistant::new();
        let report = assistant
            .enrich(Uuid::new_v4(), "evil.example", "domain")
            .await
            .unwrap();
        assert!(!report.summary.is_empty());
    }
}
