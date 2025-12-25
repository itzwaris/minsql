use crate::execution::tuple::{Tuple, Value};
use anyhow::Result;

pub struct SeqScan {
    table: String,
    columns: Vec<String>,
    position: usize,
    data: Vec<Tuple>,
}

impl SeqScan {
    pub fn new(table: String, columns: Vec<String>) -> Self {
        let data = Self::generate_mock_data(&table, &columns);
        
        Self {
            table,
            columns,
            position: 0,
            data,
        }
    }

    pub fn next(&mut self) -> Result<Option<Tuple>> {
        if self.position >= self.data.len() {
            return Ok(None);
        }

        let tuple = self.data[self.position].clone();
        self.position += 1;
        Ok(Some(tuple))
    }

    fn generate_mock_data(table: &str, columns: &[String]) -> Vec<Tuple> {
        let mut data = Vec::new();

        for i in 0..10 {
            let mut tuple = Tuple::new();
            
            for col in columns {
                match col.as_str() {
                    "id" => tuple.insert(col.clone(), Value::Integer(i as i64)),
                    "name" => tuple.insert(col.clone(), Value::String(format!("user_{}", i))),
                    "age" => tuple.insert(col.clone(), Value::Integer(20 + i as i64)),
                    _ => tuple.insert(col.clone(), Value::Null),
                }
            }

            data.push(tuple);
        }

        data
    }
}

pub struct IndexScan {
    table: String,
    index: String,
    columns: Vec<String>,
    position: usize,
}

impl IndexScan {
    pub fn new(table: String, index: String, columns: Vec<String>) -> Self {
        Self {
            table,
            index,
            columns,
            position: 0,
        }
    }

    pub fn next(&mut self) -> Result<Option<Tuple>> {
        Ok(None)
    }
}
