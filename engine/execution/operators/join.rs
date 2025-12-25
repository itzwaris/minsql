use crate::execution::tuple::Tuple;
use crate::language::intent::FilterIntent;
use anyhow::Result;
use std::collections::HashMap;

pub struct HashJoin {
    left: Vec<Tuple>,
    right: Vec<Tuple>,
    condition: FilterIntent,
    hash_table: HashMap<String, Vec<Tuple>>,
    position: usize,
}

impl HashJoin {
    pub fn new(left: Vec<Tuple>, right: Vec<Tuple>, condition: FilterIntent) -> Self {
        let mut hash_table = HashMap::new();

        for tuple in &right {
            let key = Self::extract_join_key(tuple);
            hash_table.entry(key).or_insert_with(Vec::new).push(tuple.clone());
        }

        Self {
            left,
            right,
            condition,
            hash_table,
            position: 0,
        }
    }

    pub fn next(&mut self) -> Result<Option<Tuple>> {
        if self.position >= self.left.len() {
            return Ok(None);
        }

        let left_tuple = &self.left[self.position];
        self.position += 1;

        let key = Self::extract_join_key(left_tuple);
        
        if let Some(matches) = self.hash_table.get(&key) {
            if let Some(right_tuple) = matches.first() {
                let mut joined = left_tuple.clone();
                for (k, v) in &right_tuple.values {
                    joined.insert(k.clone(), v.clone());
                }
                return Ok(Some(joined));
            }
        }

        self.next()
    }

    fn extract_join_key(tuple: &Tuple) -> String {
        if let Some(id_val) = tuple.get("id") {
            format!("{:?}", id_val)
        } else {
            String::new()
        }
    }
}

pub struct NestedLoopJoin {
    left: Vec<Tuple>,
    right: Vec<Tuple>,
    condition: FilterIntent,
    left_pos: usize,
    right_pos: usize,
}

impl NestedLoopJoin {
    pub fn new(left: Vec<Tuple>, right: Vec<Tuple>, condition: FilterIntent) -> Self {
        Self {
            left,
            right,
            condition,
            left_pos: 0,
            right_pos: 0,
        }
    }

    pub fn next(&mut self) -> Result<Option<Tuple>> {
        while self.left_pos < self.left.len() {
            while self.right_pos < self.right.len() {
                let left_tuple = &self.left[self.left_pos];
                let right_tuple = &self.right[self.right_pos];

                let mut joined = left_tuple.clone();
                for (k, v) in &right_tuple.values {
                    joined.insert(k.clone(), v.clone());
                }

                self.right_pos += 1;
                return Ok(Some(joined));
            }

            self.left_pos += 1;
            self.right_pos = 0;
        }

        Ok(None)
    }
}
