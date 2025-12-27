use crate::execution::tuple::Tuple;
use crate::language::intent::FilterIntent;
use anyhow::Result;
use std::collections::HashMap;

pub struct HashJoin {
    left: Vec<Tuple>,
    right: Vec<Tuple>,
    condition: FilterIntent,
    hash_table: HashMap<String, Vec<Tuple>>,
    left_pos: usize,
    right_pos: usize,
}

impl HashJoin {
    pub fn new(left: Vec<Tuple>, right: Vec<Tuple>, condition: FilterIntent) -> Self {
        let mut hash_table: HashMap<String, Vec<Tuple>> = HashMap::new();

        for tuple in &right {
            let key = Self::extract_join_key(tuple);
            hash_table.entry(key).or_default().push(tuple.clone());
        }

        Self {
            left,
            right,
            condition,
            hash_table,
            left_pos: 0,
            right_pos: 0,
        }
    }

    pub fn next(&mut self) -> Result<Option<Tuple>> {
        while self.left_pos < self.left.len() {
            let left_tuple = &self.left[self.left_pos];
            let key = Self::extract_join_key(left_tuple);

            if let Some(matches) = self.hash_table.get(&key) {
                if self.right_pos < matches.len() {
                    let right_tuple = &matches[self.right_pos];
                    self.right_pos += 1;

                    let mut joined = left_tuple.clone();
                    for (k, v) in &right_tuple.values {
                        joined.insert(k.clone(), v.clone());
                    }

                    return Ok(Some(joined));
                }
            }

            self.left_pos += 1;
            self.right_pos = 0;
        }

        Ok(None)
    }

    fn extract_join_key(tuple: &Tuple) -> String {
        tuple
            .get("id")
            .map(|v| format!("{:?}", v))
            .unwrap_or_default()
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
        if self.left_pos >= self.left.len() {
            return Ok(None);
        }

        if self.right_pos < self.right.len() {
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

        self.next()
    }
            }
