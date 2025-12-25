use crate::execution::tuple::{Tuple, Value};
use crate::language::intent::{AssignmentIntent, ConstantValue};
use anyhow::Result;

pub struct Insert {
    table: String,
    columns: Vec<String>,
    values: Vec<Vec<ConstantValue>>,
}

impl Insert {
    pub fn new(table: String, columns: Vec<String>, values: Vec<Vec<ConstantValue>>) -> Self {
        Self {
            table,
            columns,
            values,
        }
    }

    pub fn execute(&self) -> Result<usize> {
        Ok(self.values.len())
    }
}

pub struct Update {
    table: String,
    assignments: Vec<AssignmentIntent>,
}

impl Update {
    pub fn new(table: String, assignments: Vec<AssignmentIntent>) -> Self {
        Self {
            table,
            assignments,
        }
    }

    pub fn execute(&self) -> Result<usize> {
        Ok(1)
    }
}

pub struct Delete {
    table: String,
}

impl Delete {
    pub fn new(table: String) -> Self {
        Self { table }
    }

    pub fn execute(&self) -> Result<usize> {
        Ok(1)
    }
}
