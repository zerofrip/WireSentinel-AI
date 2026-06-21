use chrono::Utc;
use shared_types::{AiSecurityPolicy, CopilotQuery, SecurityContextEntry};
use uuid::Uuid;
use ai_controller::AiIngestPayload;
use ai_sdk::AiPlatform;

#[tokio::test]
async fn end_to_end_ingest_and_copilot() {
    let platform = AiPlatform::new();
    let tenant = Uuid::new_v4();

    let payload = AiIngestPayload {
        tenant_id: tenant,
        agent_id: Uuid::new_v4(),
        context_entries: vec![SecurityContextEntry {
            key: "alert".into(),
            value: "brute-force".into(),
            source: "siem".into(),
            observed_at: Utc::now(),
        }],
        security_policy: Some(AiSecurityPolicy::default()),
        ingested_at: Utc::now(),
    };

    for entry in payload.context_entries {
        platform.context.ingest(entry);
    }

    assert!(platform.context.count() > 0);

    platform
        .copilot
        .query(
            payload.security_policy.as_ref().unwrap(),
            CopilotQuery {
                id: Uuid::new_v4(),
                tenant_id: tenant,
                user_id: Uuid::new_v4(),
                prompt: "investigate brute force alert".into(),
                context_ids: vec![],
                submitted_at: Utc::now(),
            },
        )
        .await
        .unwrap();

    assert!(!platform.drain_events().is_empty());
}

#[test]
fn platform_modules_initialized() {
    let platform = AiPlatform::new();
    assert_eq!(platform.rag.document_count(), 0);
    assert_eq!(platform.intelligence.report_count(), 0);
}
