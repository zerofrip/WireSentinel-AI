use chrono::Utc;
use serde::{Deserialize, Serialize};
use shared_types::{AiContextBundle, AiTelemetryPayload, SecurityContextEntry};
use uuid::Uuid;

pub use shared_types::{AiSecurityPolicy, AiTelemetryPayload as TelemetryPayload};

/// Controller ingest payload wrapping AI context batches.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiIngestPayload {
    pub tenant_id: Uuid,
    pub agent_id: Uuid,
    pub context_entries: Vec<SecurityContextEntry>,
    pub security_policy: Option<AiSecurityPolicy>,
    pub ingested_at: chrono::DateTime<Utc>,
}

impl AiIngestPayload {
    pub fn empty(tenant_id: Uuid, agent_id: Uuid) -> Self {
        Self {
            tenant_id,
            agent_id,
            context_entries: Vec::new(),
            security_policy: None,
            ingested_at: Utc::now(),
        }
    }

    pub fn event_count(&self) -> u32 {
        self.context_entries.len() as u32
    }
}

/// Acknowledgement returned to Controller after ingest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiIngestResponse {
    pub accepted: bool,
    pub events_processed: u32,
    pub message: String,
}

/// Scan bundle request from controller.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiScanRequest {
    pub tenant_id: Uuid,
    pub bundle: AiContextBundle,
}

pub fn parse_ingest_payload(json: &str) -> Result<AiIngestPayload, serde_json::Error> {
    serde_json::from_str(json)
}

pub fn build_telemetry_payload(
    agent_id: Uuid,
    tenant_id: Uuid,
    copilot_queries: u32,
    investigations: u32,
    correlations: u32,
    rag_documents: u32,
    recommendations: u32,
    ai_risk_score: f64,
) -> AiTelemetryPayload {
    AiTelemetryPayload {
        agent_id,
        tenant_id,
        reported_at: Utc::now(),
        copilot_queries,
        investigations,
        correlations,
        rag_documents,
        recommendations,
        ai_risk_score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_payload_has_zero_events() {
        let payload = AiIngestPayload::empty(Uuid::new_v4(), Uuid::new_v4());
        assert_eq!(payload.event_count(), 0);
    }

    #[test]
    fn roundtrips_json() {
        let payload = AiIngestPayload::empty(Uuid::new_v4(), Uuid::new_v4());
        let json = serde_json::to_string(&payload).unwrap();
        let parsed = parse_ingest_payload(&json).unwrap();
        assert_eq!(parsed.tenant_id, payload.tenant_id);
    }
}
