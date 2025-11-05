use crate::rules::{factory::RuleFactory, registry::RuleRegistry, Rule};
use std::sync::Arc;

pub struct RulePool {
    factory: Arc<RuleFactory>,
    registry: Arc<RuleRegistry>,
}

impl RulePool {
    pub fn new() -> Self {
        Self {
            factory: Arc::new(RuleFactory::new()),
            registry: Arc::new(RuleRegistry::new()),
        }
    }

    pub fn get_rule(&self, rule_id: &str) -> Option<Box<dyn Rule>> {
        self.factory.create_rule(rule_id)
    }

    pub fn get_enabled_rules(&self) -> Vec<Box<dyn Rule>> {
        let enabled_rule_ids = self.registry.get_default_enabled_rules();
        enabled_rule_ids
            .iter()
            .filter_map(|id| self.get_rule(id))
            .collect()
    }

    pub fn get_rules_by_ids(&self, rule_ids: &[String]) -> Vec<Box<dyn Rule>> {
        rule_ids.iter().filter_map(|id| self.get_rule(id)).collect()
    }

    pub fn registry(&self) -> &RuleRegistry {
        &self.registry
    }
}

impl Default for RulePool {
    fn default() -> Self {
        Self::new()
    }
}
