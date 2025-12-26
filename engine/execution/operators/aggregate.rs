use crate::execution::tuple::{Tuple, Value};
use crate::language::intent::{AggregateIntent, ExpressionIntent};
use anyhow::Result;
use std::collections::HashMap;

pub struct HashAggregate {
    input: Vec<Tuple>,
    group_by: Vec<ExpressionIntent>,
    aggregates: Vec<AggregateIntent>,
    groups: HashMap<String, AggregateState>,
    finalized: bool,
    result_iter: std::vec::IntoIter<Tuple>,
}

impl HashAggregate {
    pub fn new(
        input: Vec<Tuple>,
        group_by: Vec<ExpressionIntent>,
        aggregates: Vec<AggregateIntent>,
    ) -> Self {
        let mut groups = HashMap::new();

        for tuple in &input {
            let group_key = Self::compute_group_key(&group_by, tuple);
            let state = groups.entry(group_key).or_insert_with(AggregateState::new);
            state.accumulate(&aggregates, tuple);
        }

        Self {
            input,
            group_by,
            aggregates,
            groups,
            finalized: false,
            result_iter: Vec::new().into_iter(),
        }
    }

    pub fn next(&mut self) -> Result<Option<Tuple>> {
        if !self.finalized {
            self.finalize()?;
            self.finalized = true;
        }

        Ok(self.result_iter.next())
    }

    fn finalize(&mut self) -> Result<()> {
        let mut results = Vec::new();

        for (_group_key, state) in &self.groups {
            let mut tuple = Tuple::new();

            for agg in &self.aggregates {
                let value = state.finalize(&agg.function);
                let col_name = agg.alias.as_ref().unwrap_or(&agg.function).clone();
                tuple.insert(col_name, value);
            }

            results.push(tuple);
        }

        self.result_iter = results.into_iter();
        Ok(())
    }

    fn compute_group_key(_group_by: &[ExpressionIntent], _tuple: &Tuple) -> String {
        String::from("default_group")
    }
}

struct AggregateState {
    count: i64,
    sum: f64,
    min: Option<f64>,
    max: Option<f64>,
}

impl AggregateState {
    fn new() -> Self {
        Self {
            count: 0,
            sum: 0.0,
            min: None,
            max: None,
        }
    }

    fn accumulate(&mut self, _aggregates: &[AggregateIntent], _tuple: &Tuple) {
        self.count += 1;
        self.sum += 1.0;

        if self.min.is_none() {
            self.min = Some(1.0);
        }

        if self.max.is_none() {
            self.max = Some(1.0);
        }
    }

    fn finalize(&self, function: &str) -> Value {
        match function.to_lowercase().as_str() {
            "count" => Value::Integer(self.count),
            "sum" => Value::Float(self.sum),
            "avg" => {
                if self.count > 0 {
                    Value::Float(self.sum / self.count as f64)
                } else {
                    Value::Null
                }
            }
            "min" => self.min.map(Value::Float).unwrap_or(Value::Null),
            "max" => self.max.map(Value::Float).unwrap_or(Value::Null),
            _ => Value::Null,
        }
    }
}
