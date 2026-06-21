use chrono::Utc;
use parking_lot::RwLock;
use shared_types::{AiReportFormat, ExecutiveReport, ServiceEventInner};
use uuid::Uuid;

use ai_core::{AiEventEmitter, AiResult};

struct ReportingState {
    reports: Vec<ExecutiveReport>,
}

impl Default for ReportingState {
    fn default() -> Self {
        Self { reports: Vec::new() }
    }
}

/// Generates executive security reports in multiple formats.
pub struct ExecutiveReportingEngine<E: AiEventEmitter> {
    emitter: E,
    state: RwLock<ReportingState>,
}

impl<E: AiEventEmitter> ExecutiveReportingEngine<E> {
    pub fn new(emitter: E) -> Self {
        Self {
            emitter,
            state: RwLock::new(ReportingState::default()),
        }
    }

    pub fn report_count(&self) -> usize {
        self.state.read().reports.len()
    }

    pub fn generate(
        &self,
        tenant_id: Uuid,
        title: &str,
        format: AiReportFormat,
        risk_score: f64,
        highlights: &[&str],
    ) -> AiResult<ExecutiveReport> {
        let content = match format {
            AiReportFormat::Json => serde_json::json!({
                "title": title,
                "risk_score": risk_score,
                "highlights": highlights,
            })
            .to_string(),
            AiReportFormat::Markdown => format!(
                "# {}\n\nRisk score: {:.1}\n\n## Highlights\n{}",
                title,
                risk_score,
                highlights
                    .iter()
                    .map(|h| format!("- {}", h))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
            AiReportFormat::Pdf => format!(
                "PDF-STUB:{}:risk={:.1}:items={}",
                title,
                risk_score,
                highlights.len()
            ),
        };

        let report = ExecutiveReport {
            id: Uuid::new_v4(),
            tenant_id,
            title: title.to_string(),
            format,
            content,
            risk_score,
            generated_at: Utc::now(),
        };

        self.state.write().reports.push(report.clone());
        self.emitter
            .emit(shared_types::ServiceEvent::now(ServiceEventInner::ExecutiveReportGenerated {
                report: report.clone(),
            }));

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_core::CollectingEmitter;

    #[test]
    fn generates_markdown_report() {
        let emitter = CollectingEmitter::new();
        let engine = ExecutiveReportingEngine::new(&emitter);
        let report = engine
            .generate(
                Uuid::new_v4(),
                "Weekly Summary",
                AiReportFormat::Markdown,
                42.0,
                &["2 incidents", "1 critical vuln"],
            )
            .unwrap();
        assert!(report.content.contains("# Weekly Summary"));
    }

    #[test]
    fn emits_executive_report_event() {
        let emitter = CollectingEmitter::new();
        let engine = ExecutiveReportingEngine::new(&emitter);
        engine
            .generate(
                Uuid::new_v4(),
                "JSON Report",
                AiReportFormat::Json,
                10.0,
                &["ok"],
            )
            .unwrap();
        assert!(!emitter.drain().is_empty());
    }
}
