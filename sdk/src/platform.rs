use std::sync::Arc;

use analytics::SecurityAnalyticsAssistant;
use copilot::SecurityCopilot;
use correlation::ThreatCorrelationEngine;
use detections::DetectionAssistant;
use investigation::AiInvestigationEngine;
use intelligence::ThreatIntelAssistant;
use knowledge_graph::SecurityKnowledgeGraph;
use playbooks::PlaybookAssistant;
use policies::PolicyAssistant;
use rag::SecurityRagEngine;
use reporting::ExecutiveReportingEngine;
use risk::AiRiskEngine;
use ai_core::{CollectingEmitter, ContextAggregator};

/// Facade bundling all AI engines behind a shared event emitter.
pub struct AiPlatform {
    pub emitter: Arc<CollectingEmitter>,
    pub context: ContextAggregator,
    pub copilot: SecurityCopilot<Arc<CollectingEmitter>>,
    pub investigation: AiInvestigationEngine<Arc<CollectingEmitter>>,
    pub correlation: ThreatCorrelationEngine<Arc<CollectingEmitter>>,
    pub knowledge_graph: SecurityKnowledgeGraph,
    pub rag: SecurityRagEngine,
    pub detections: DetectionAssistant<Arc<CollectingEmitter>>,
    pub playbooks: PlaybookAssistant<Arc<CollectingEmitter>>,
    pub policies: PolicyAssistant<Arc<CollectingEmitter>>,
    pub intelligence: ThreatIntelAssistant,
    pub analytics: SecurityAnalyticsAssistant,
    pub reporting: ExecutiveReportingEngine<Arc<CollectingEmitter>>,
    pub risk: AiRiskEngine<Arc<CollectingEmitter>>,
}

impl AiPlatform {
    pub fn new() -> Self {
        let emitter = Arc::new(CollectingEmitter::new());

        Self {
            copilot: SecurityCopilot::new(emitter.clone()),
            investigation: AiInvestigationEngine::new(emitter.clone()),
            correlation: ThreatCorrelationEngine::new(emitter.clone()),
            detections: DetectionAssistant::new(emitter.clone()),
            playbooks: PlaybookAssistant::new(emitter.clone()),
            policies: PolicyAssistant::new(emitter.clone()),
            reporting: ExecutiveReportingEngine::new(emitter.clone()),
            risk: AiRiskEngine::new(emitter.clone()),
            context: ContextAggregator::new(),
            knowledge_graph: SecurityKnowledgeGraph::new(),
            rag: SecurityRagEngine::new(),
            intelligence: ThreatIntelAssistant::new(),
            analytics: SecurityAnalyticsAssistant::new(),
            emitter,
        }
    }

    pub fn drain_events(&self) -> Vec<shared_types::ServiceEvent> {
        self.emitter.drain()
    }
}

impl Default for AiPlatform {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use shared_types::{AiSecurityPolicy, CopilotQuery};
    use uuid::Uuid;

    #[test]
    fn platform_bundles_engines() {
        let platform = AiPlatform::new();
        assert_eq!(platform.copilot.query_count(), 0);
        assert_eq!(platform.knowledge_graph.node_count(), 0);
    }

    #[tokio::test]
    async fn shared_emitter_collects_events() {
        let platform = AiPlatform::new();
        let policy = AiSecurityPolicy::default();
        platform
            .copilot
            .query(
                &policy,
                CopilotQuery {
                    id: Uuid::new_v4(),
                    tenant_id: Uuid::new_v4(),
                    user_id: Uuid::new_v4(),
                    prompt: "investigate alert".into(),
                    context_ids: vec![],
                    submitted_at: Utc::now(),
                },
            )
            .await
            .unwrap();
        assert!(!platform.drain_events().is_empty());
    }
}
