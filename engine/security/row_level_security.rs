use crate::execution::tuple::Tuple;
use crate::language::intent::FilterIntent;
use anyhow::Result;
use std::collections::HashMap;

pub struct RowLevelSecurityPolicy {
    pub table: String,
    pub policy_name: String,
    pub filter: FilterIntent,
    pub roles: Vec<String>,
}

pub struct RLSManager {
    policies: HashMap<String, Vec<RowLevelSecurityPolicy>>,
}

impl RLSManager {
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
        }
    }

    pub fn add_policy(&mut self, policy: RowLevelSecurityPolicy) {
        self.policies
            .entry(policy.table.clone())
            .or_insert_with(Vec::new)
            .push(policy);
    }

    pub fn remove_policy(&mut self, table: &str, policy_name: &str) {
        if let Some(policies) = self.policies.get_mut(table) {
            policies.retain(|p| p.policy_name != policy_name);
        }
    }

    pub fn get_policies(&self, table: &str, role: &str) -> Vec<&RowLevelSecurityPolicy> {
        if let Some(policies) = self.policies.get(table) {
            policies
                .iter()
                .filter(|p| p.roles.contains(&role.to_string()) || p.roles.is_empty())
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn apply_policies(
        &self,
        table: &str,
        role: &str,
        tuples: Vec<Tuple>,
    ) -> Result<Vec<Tuple>> {
        let policies = self.get_policies(table, role);
        
        if policies.is_empty() {
            return Ok(tuples);
        }

        let mut filtered = tuples;
        
        for policy in policies {
            filtered = filtered
                .into_iter()
                .filter(|tuple| self.evaluate_policy_filter(&policy.filter, tuple))
                .collect();
        }

        Ok(filtered)
    }

    fn evaluate_policy_filter(&self, _filter: &FilterIntent, _tuple: &Tuple) -> bool {
        true
    }

    pub fn list_policies(&self, table: &str) -> Vec<String> {
        if let Some(policies) = self.policies.get(table) {
            policies.iter().map(|p| p.policy_name.clone()).collect()
        } else {
            Vec::new()
        }
    }
          }
