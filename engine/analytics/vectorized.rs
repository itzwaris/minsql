use crate::execution::tuple::{Tuple, Value};
use anyhow::Result;

const VECTOR_SIZE: usize = 1024;

pub struct VectorBatch {
    tuples: Vec<Tuple>,
}

impl VectorBatch {
    pub fn new() -> Self {
        Self {
            tuples: Vec::with_capacity(VECTOR_SIZE),
        }
    }

    pub fn add(&mut self, tuple: Tuple) -> bool {
        if self.tuples.len() >= VECTOR_SIZE {
            return false;
        }
        self.tuples.push(tuple);
        true
    }

    pub fn is_full(&self) -> bool {
        self.tuples.len() >= VECTOR_SIZE
    }

    pub fn len(&self) -> usize {
        self.tuples.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Tuple> {
        self.tuples.iter()
    }

    pub fn clear(&mut self) {
        self.tuples.clear();
    }
}

pub struct VectorizedExecutor;

impl VectorizedExecutor {
    pub fn new() -> Self {
        Self
    }

    pub fn filter_batch(
        &self,
        batch: &VectorBatch,
        predicate: impl Fn(&Tuple) -> bool,
    ) -> VectorBatch {
        let mut result = VectorBatch::new();

        for tuple in batch.iter() {
            if predicate(tuple) {
                result.add(tuple.clone());
            }
        }

        result
    }

    pub fn project_batch(&self, batch: &VectorBatch, columns: &[String]) -> VectorBatch {
        let mut result = VectorBatch::new();

        for tuple in batch.iter() {
            let mut projected = Tuple::new();
            for col in columns {
                if let Some(val) = tuple.get(col) {
                    projected.insert(col.clone(), val.clone());
                }
            }
            result.add(projected);
        }

        result
    }

    pub fn aggregate_batch(&self, batch: &VectorBatch, column: &str) -> Result<Value> {
        let mut sum = 0.0;
        let mut count = 0;

        for tuple in batch.iter() {
            if let Some(value) = tuple.get(column) {
                match value {
                    Value::Integer(i) => {
                        sum += *i as f64;
                        count += 1;
                    }
                    Value::Float(f) => {
                        sum += f;
                        count += 1;
                    }
                    _ => {}
                }
            }
        }

        if count > 0 {
            Ok(Value::Float(sum / count as f64))
        } else {
            Ok(Value::Null)
        }
    }

    pub fn join_batches(
        &self,
        left: &VectorBatch,
        right: &VectorBatch,
        left_key: &str,
        right_key: &str,
    ) -> VectorBatch {
        let mut result = VectorBatch::new();

        for left_tuple in left.iter() {
            for right_tuple in right.iter() {
                if let (Some(left_val), Some(right_val)) =
                    (left_tuple.get(left_key), right_tuple.get(right_key))
                {
                    if self.values_equal(left_val, right_val) {
                        let mut joined = left_tuple.clone();
                        for (k, v) in &right_tuple.values {
                            joined.insert(k.clone(), v.clone());
                        }
                        if !result.add(joined) {
                            break;
                        }
                    }
                }
            }
        }

        result
    }

    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Integer(x), Value::Integer(y)) => x == y,
            (Value::String(x), Value::String(y)) => x == y,
            (Value::Boolean(x), Value::Boolean(y)) => x == y,
            _ => false,
        }
    }
}
