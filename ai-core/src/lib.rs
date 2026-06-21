//! Core AI abstractions for WireSentinel Phase 19.

mod error;
mod emitter;
mod engine;
mod llm;

pub use error::{AiError, AiResult};
pub use emitter::{AiEventEmitter, CollectingEmitter, NullEmitter};
pub use engine::{AiSecurityPolicyEngine, ContextAggregator};
pub use llm::{ensure_provider_allowed, LlmProvider, MockLlmProvider};
pub use shared_types::{
    AiAnalyticsSummary, AiContextBundle, AiIntentKind, AiProvider, AiRecommendation,
    AiRecommendationKind, AiReportFormat, AiRiskScore, AiSecurityPolicy,
    AiSecurityViolationDetail, AiSeverity, AiTelemetryPayload, AttackNarrative,
    CopilotQuery, CopilotResponse, CorrelatedThreat, DetectionSuggestion, EmbeddingRecord,
    ExecutiveReport, InvestigationReport, KnowledgeGraphEdge, KnowledgeGraphNode, PlaybookStepSuggestion,
    PlaybookSuggestion, PolicySuggestion, RagChunk, RagDocument, RagRetrievalResult,
    RootCauseAnalysis, SecurityContextEntry, ThreatIntelligenceReport,
};
