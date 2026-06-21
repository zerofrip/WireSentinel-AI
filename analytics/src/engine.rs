use chrono::Utc;
use parking_lot::RwLock;
use shared_types::AiAnalyticsSummary;
use uuid::Uuid;

struct AnalyticsState {
    queries: u64,
    investigations: u64,
    correlations: u64,
    recommendations: u64,
    risk_total: f64,
    risk_samples: u64,
}

impl Default for AnalyticsState {
    fn default() -> Self {
        Self {
            queries: 0,
            investigations: 0,
            correlations: 0,
            recommendations: 0,
            risk_total: 0.0,
            risk_samples: 0,
        }
    }
}

/// Aggregates AI platform usage metrics.
pub struct SecurityAnalyticsAssistant {
    state: RwLock<AnalyticsState>,
}

impl SecurityAnalyticsAssistant {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(AnalyticsState::default()),
        }
    }

    pub fn record_query(&self) {
        self.state.write().queries += 1;
    }

    pub fn record_investigation(&self) {
        self.state.write().investigations += 1;
    }

    pub fn record_correlation(&self) {
        self.state.write().correlations += 1;
    }

    pub fn record_recommendation(&self) {
        self.state.write().recommendations += 1;
    }

    pub fn record_risk_score(&self, score: f64) {
        let mut state = self.state.write();
        state.risk_total += score;
        state.risk_samples += 1;
    }

    pub fn summarize(&self, tenant_id: Uuid) -> AiAnalyticsSummary {
        let state = self.state.read();
        let average_risk = if state.risk_samples == 0 {
            0.0
        } else {
            state.risk_total / state.risk_samples as f64
        };

        AiAnalyticsSummary {
            tenant_id,
            total_queries: state.queries,
            investigations_completed: state.investigations,
            threats_correlated: state.correlations,
            recommendations_generated: state.recommendations,
            average_risk_score: average_risk,
            computed_at: Utc::now(),
        }
    }
}

impl Default for SecurityAnalyticsAssistant {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_recorded_metrics() {
        let analytics = SecurityAnalyticsAssistant::new();
        analytics.record_query();
        analytics.record_query();
        analytics.record_investigation();
        analytics.record_risk_score(50.0);
        analytics.record_risk_score(70.0);

        let summary = analytics.summarize(Uuid::new_v4());
        assert_eq!(summary.total_queries, 2);
        assert_eq!(summary.investigations_completed, 1);
        assert!((summary.average_risk_score - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn empty_analytics_has_zero_averages() {
        let analytics = SecurityAnalyticsAssistant::new();
        let summary = analytics.summarize(Uuid::new_v4());
        assert_eq!(summary.average_risk_score, 0.0);
    }
}
