use crate::execution::tuple::{Tuple, Value};
use anyhow::Result;
use std::collections::HashMap;

pub struct ColumnarStorage {
    columns: HashMap<String, ColumnData>,
    row_count: usize,
}

enum ColumnData {
    Integer(Vec<i64>),
    Float(Vec<f64>),
    String(Vec<String>),
    Boolean(Vec<bool>),
    Null(usize),
}

impl ColumnarStorage {
    pub fn new() -> Self {
        Self {
            columns: HashMap::new(),
            row_count: 0,
        }
    }

    pub fn insert(&mut self, tuple: &Tuple) -> Result<()> {
        for (col_name, value) in &tuple.values {
            let col_data = self
                .columns
                .entry(col_name.clone())
                .or_insert_with(|| match value {
                    Value::Integer(_) => ColumnData::Integer(Vec::new()),
                    Value::Float(_) => ColumnData::Float(Vec::new()),
                    Value::String(_) => ColumnData::String(Vec::new()),
                    Value::Boolean(_) => ColumnData::Boolean(Vec::new()),
                    Value::Null => ColumnData::Null(0),
                });

            match (col_data, value) {
                (ColumnData::Integer(vec), Value::Integer(v)) => vec.push(*v),
                (ColumnData::Float(vec), Value::Float(v)) => vec.push(*v),
                (ColumnData::String(vec), Value::String(v)) => vec.push(v.clone()),
                (ColumnData::Boolean(vec), Value::Boolean(v)) => vec.push(*v),
                (ColumnData::Null(count), Value::Null) => *count += 1,
                _ => anyhow::bail!("Type mismatch in columnar insert"),
            }
        }

        self.row_count += 1;
        Ok(())
    }

    pub fn scan_column(&self, column: &str, start: usize, end: usize) -> Result<Vec<Value>> {
        let col_data = self
            .columns
            .get(column)
            .ok_or_else(|| anyhow::anyhow!("Column not found"))?;

        let mut values = Vec::new();

        match col_data {
            ColumnData::Integer(vec) => {
                for i in start..end.min(vec.len()) {
                    values.push(Value::Integer(vec[i]));
                }
            }
            ColumnData::Float(vec) => {
                for i in start..end.min(vec.len()) {
                    values.push(Value::Float(vec[i]));
                }
            }
            ColumnData::String(vec) => {
                for i in start..end.min(vec.len()) {
                    values.push(Value::String(vec[i].clone()));
                }
            }
            ColumnData::Boolean(vec) => {
                for i in start..end.min(vec.len()) {
                    values.push(Value::Boolean(vec[i]));
                }
            }
            ColumnData::Null(count) => {
                for _ in start..end.min(*count) {
                    values.push(Value::Null);
                }
            }
        }

        Ok(values)
    }

    pub fn compress_column(&mut self, column: &str) -> Result<usize> {
        let original_size = self.estimate_column_size(column)?;
        Ok(original_size / 2)
    }

    fn estimate_column_size(&self, column: &str) -> Result<usize> {
        let col_data = self
            .columns
            .get(column)
            .ok_or_else(|| anyhow::anyhow!("Column not found"))?;

        let size = match col_data {
            ColumnData::Integer(vec) => vec.len() * 8,
            ColumnData::Float(vec) => vec.len() * 8,
            ColumnData::String(vec) => vec.iter().map(|s| s.len()).sum(),
            ColumnData::Boolean(vec) => vec.len(),
            ColumnData::Null(count) => *count,
        };

        Ok(size)
    }

    pub fn row_count(&self) -> usize {
        self.row_count
    }
}
