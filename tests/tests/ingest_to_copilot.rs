use ai_controller::{build_telemetry_payload, AiIngestPayload};
use ai_sdk::AiPlatform;
use uuid::Uuid;

#[test]
fn ingest_payload_serializes() {
    let payload = AiIngestPayload::empty(Uuid::new_v4(), Uuid::new_v4());
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("tenant_id"));
}

#[tokio::test]
async fn telemetry_reflects_platform_activity() {
    let platform = AiPlatform::new();
    platform.analytics.record_query();
    platform.analytics.record_recommendation();

    let summary = platform.analytics.summarize(Uuid::new_v4());
    let telemetry = build_telemetry_payload(
        Uuid::new_v4(),
        summary.tenant_id,
        summary.total_queries as u32,
        summary.investigations_completed as u32,
        summary.threats_correlated as u32,
        platform.rag.document_count() as u32,
        summary.recommendations_generated as u32,
        summary.average_risk_score,
    );

    assert_eq!(telemetry.copilot_queries, 1);
    assert_eq!(telemetry.recommendations, 1);
}
