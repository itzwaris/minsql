use crate::execution::expression::ExpressionEvaluator;
use crate::execution::operators::scan::SeqScan;
use crate::execution::sandbox::{QueryLimits, Sandbox};
use crate::execution::tuple::Tuple;
use crate::ffi::storage::StorageEngine;
use crate::planner::physical::PhysicalPlan;
use crate::language::ast::ColumnDefinition;
use anyhow::Result;
use futures::future::BoxFuture;
use futures::FutureExt;
use std::collections::HashMap;
use serde_json::json;

pub struct ExecutionEngine<'a> {
    storage: &'a StorageEngine,
    evaluator: ExpressionEvaluator,
}

impl<'a> ExecutionEngine<'a> {
    pub fn new(storage: &'a StorageEngine) -> Self {
        Self {
            storage,
            evaluator: ExpressionEvaluator::new(),
        }
    }

    pub async fn execute(&mut self, plan: PhysicalPlan) -> Result<Vec<Tuple>> {
        let sandbox = Sandbox::new(QueryLimits::default());
        self.execute_with_sandbox(plan, sandbox).await
    }

    fn execute_with_sandbox<'b>(&'b mut self, plan: PhysicalPlan, sandbox: Sandbox) -> BoxFuture<'b, Result<Vec<Tuple>>> {
        async move {
            let mut sandbox = sandbox;
            sandbox.check()?;

            match plan {
                PhysicalPlan::SeqScan { table, columns } => {
                    let mut scan = SeqScan::new(table, columns);
                    let mut results = Vec::new();

                    while let Some(tuple) = scan.next()? {
                        sandbox.check()?;
                        results.push(tuple);
                    }

                    Ok(results)
                }
                PhysicalPlan::Filter { predicate, input } => {
                    let tuples = self.execute_with_sandbox(*input, sandbox).await?;
                    let mut results = Vec::new();

                    for tuple in tuples {
                        if self.evaluator.evaluate_filter(&predicate, &tuple)? {
                            results.push(tuple);
                        }
                    }

                    Ok(results)
                }
                PhysicalPlan::Project { columns, input } => {
                    let tuples = self.execute_with_sandbox(*input, sandbox).await?;
                    let mut results = Vec::new();

                    for tuple in tuples {
                        let mut projected = Tuple::new();
                        
                        for col_intent in &columns {
                            match col_intent {
                                crate::language::intent::ColumnIntent::Named(name) => {
                                    if let Some(val) = tuple.get(name) {
                                        projected.insert(name.clone(), val.clone());
                                    }
                                }
                                crate::language::intent::ColumnIntent::All => {
                                    for (k, v) in &tuple.values {
                                        projected.insert(k.clone(), v.clone());
                                    }
                                }
                                _ => {}
                            }
                        }

                        results.push(projected);
                    }

                    Ok(results)
                }
                PhysicalPlan::Limit { count, offset, input } => {
                    let tuples = self.execute_with_sandbox(*input, sandbox).await?;
                    Ok(tuples.into_iter().skip(offset).take(count).collect())
                }
                PhysicalPlan::Insert { table, columns, values } => {
                    tracing::info!("INSERT into {} with {} rows", table, values.len());
                    
                    self.storage.wal_flush()?;

                    let mut inserted_count = 0;
                    for row in &values {
                        let mut tuple = Tuple::new();
                        for (i, col_name) in columns.iter().enumerate() {
                            if let Some(value) = row.get(i) {
                                let val = match value {
                                    crate::language::intent::ConstantValue::Null => {
                                        crate::execution::tuple::Value::Null
                                    }
                                    crate::language::intent::ConstantValue::Boolean(b) => {
                                        crate::execution::tuple::Value::Boolean(*b)
                                    }
                                    crate::language::intent::ConstantValue::Integer(i) => {
                                        crate::execution::tuple::Value::Integer(*i)
                                    }
                                    crate::language::intent::ConstantValue::Float(f) => {
                                        crate::execution::tuple::Value::Float(*f)
                                    }
                                    crate::language::intent::ConstantValue::String(s) => {
                                        crate::execution::tuple::Value::String(s.clone())
                                    }
                                };
                                tuple.insert(col_name.clone(), val);
                            }
                        }
                        inserted_count += 1;
                        let tuple_json = serde_json::to_string(&tuple)?;
                        let tuple_bytes = tuple_json.as_bytes();
                        let row_id = self.storage.insert_row(&table, tuple_bytes)?;
                        
                        tracing::debug!("Inserted row {} (rowid={}) into {}: {:?}", 
                            inserted_count, row_id, table, tuple);
                    }

                    self.storage.wal_flush()?;
                    
                    tracing::info!("Successfully inserted {} rows into {}", inserted_count, table);
                    Ok(vec![])
                }
                PhysicalPlan::Update { table, assignments, filter } => {
                    tracing::info!("UPDATE {} with {} assignments", table, assignments.len());

                    let filter_str = match &filter {
                        Some(f) => format!("{:?}", f),
                        None => "true".to_string(),
                    };

                    let assignments_json = serde_json::to_string(&assignments)?;
                    let assignments_bytes = assignments_json.as_bytes();
                    let updated_count = self.storage.update_rows(&table, &filter_str, assignments_bytes)?;

                    self.storage.wal_flush()?;
                    
                    tracing::info!("Successfully updated {} rows in {}", updated_count, table);
                    Ok(vec![])
                }
                PhysicalPlan::Delete { table, filter } => {
                    tracing::info!("DELETE from {}", table);

                    let filter_str = match &filter {
                        Some(f) => format!("{:?}", f),
                        None => "true".to_string(),
                    };
                    
                    let deleted_count = self.storage.delete_rows(&table, &filter_str)?;
                    self.storage.wal_flush()?;
                    
                    tracing::info!("Successfully deleted {} rows from {}", deleted_count, table);
                    Ok(vec![])
                }
                PhysicalPlan::CreateTable { name, columns } => {
                    tracing::info!("CREATE TABLE {} with {} columns", name, columns.len());
                    
                    let mut schema = HashMap::new();
                    for col in &columns {
                        let col_info = json!({
                            "name": col.name,
                            "type": format!("{:?}", col.data_type),
                            "nullable": col.nullable,
                            "primary_key": col.primary_key,
                        });
                        schema.insert(col.name.clone(), col_info);
                    }
                    
                    let schema_json = serde_json::to_string_pretty(&schema)?; 
                    tracing::debug!("Creating table with schema: {}", schema_json);
                    self.storage.create_table(&name, &schema_json)?;
                    self.storage.wal_flush()?;
                    self.storage.checkpoint()?;
                    
                    tracing::info!("Successfully created table: {}", name);
                    Ok(vec![])
                }                _ => {
                    anyhow::bail!("Unsupported plan type")
                }
            }
        }.boxed()
    }
          }
