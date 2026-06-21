use parking_lot::RwLock;
use shared_types::{
    AiProvider, AiSecurityPolicy, AiSecurityViolationDetail, SecurityContextEntry, ServiceEvent,
    ServiceEventInner,
};

use crate::{AiError, AiEventEmitter, AiResult};

/// Aggregates security context entries for LLM prompts.
pub struct ContextAggregator {
    entries: RwLock<Vec<SecurityContextEntry>>,
}

impl ContextAggregator {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
        }
    }

    pub fn ingest(&self, entry: SecurityContextEntry) {
        self.entries.write().push(entry);
    }

    pub fn count(&self) -> usize {
        self.entries.read().len()
    }

    pub fn render_prompt_context(&self) -> String {
        self.entries
            .read()
            .iter()
            .map(|e| format!("{}={} ({})", e.key, e.value, e.source))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for ContextAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// Validates AI operations against tenant security policy.
pub struct AiSecurityPolicyEngine;

impl AiSecurityPolicyEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_prompt<E: AiEventEmitter>(
        &self,
        emitter: &E,
        policy: &AiSecurityPolicy,
        prompt: &str,
    ) -> AiResult<()> {
        if prompt.len() as u32 > policy.max_prompt_length {
            Self::violation(emitter, "prompt", "length exceeded", "copilot");
            emitter.emit(ServiceEvent::now(ServiceEventInner::PromptBlocked {
                tenant_id: policy.tenant_id,
                reason: "prompt too long".into(),
            }));
            return Err(AiError::Security("prompt length exceeded".into()));
        }

        let lower = prompt.to_lowercase();
        for pattern in &policy.blocked_prompt_patterns {
            if lower.contains(&pattern.to_lowercase()) {
                Self::violation(emitter, "prompt", "blocked pattern", pattern);
                emitter.emit(ServiceEvent::now(ServiceEventInner::PromptBlocked {
                    tenant_id: policy.tenant_id,
                    reason: format!("blocked pattern: {}", pattern),
                }));
                return Err(AiError::Security("blocked prompt pattern".into()));
            }
        }

        Ok(())
    }

    pub fn validate_provider<E: AiEventEmitter>(
        &self,
        emitter: &E,
        policy: &AiSecurityPolicy,
        provider: AiProvider,
    ) -> AiResult<()> {
        if policy.allowed_providers.iter().any(|p| *p == provider) {
            Ok(())
        } else {
            Self::violation(emitter, "provider", "access denied", &format!("{:?}", provider));
            emitter.emit(ServiceEvent::now(ServiceEventInner::ProviderAccessDenied {
                tenant_id: policy.tenant_id,
                provider: format!("{:?}", provider),
            }));
            Err(AiError::Security("provider not permitted".into()))
        }
    }

    fn violation<E: AiEventEmitter>(emitter: &E, violation_type: &str, detail: &str, resource: &str) {
        let _detail = AiSecurityViolationDetail {
            violation_type: violation_type.to_string(),
            detail: detail.to_string(),
            resource: resource.to_string(),
        };
        emitter.emit(ServiceEvent::now(ServiceEventInner::AiSecurityViolation {
            violation_type: violation_type.to_string(),
            detail: format!("{}: {}", detail, resource),
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::CollectingEmitter;

    #[test]
    fn aggregator_renders_context() {
        let agg = ContextAggregator::new();
        agg.ingest(SecurityContextEntry {
            key: "host".into(),
            value: "srv-1".into(),
            source: "edr".into(),
            observed_at: Utc::now(),
        });
        assert!(agg.render_prompt_context().contains("srv-1"));
    }

    #[test]
    fn blocks_long_prompt() {
        let emitter = CollectingEmitter::new();
        let engine = AiSecurityPolicyEngine::new();
        let policy = AiSecurityPolicy {
            max_prompt_length: 5,
            ..AiSecurityPolicy::default()
        };
        assert!(engine.validate_prompt(&emitter, &policy, "too long prompt").is_err());
        assert!(!emitter.drain().is_empty());
    }
}
