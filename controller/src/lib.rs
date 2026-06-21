//! Controller DTOs for WireSentinel AI ingest.

mod dto;

pub use dto::{
    build_telemetry_payload, parse_ingest_payload, AiIngestPayload, AiIngestResponse,
    AiScanRequest, TelemetryPayload,
};
