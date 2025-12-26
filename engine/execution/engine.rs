use crate::execution::expression::ExpressionEvaluator;
use crate::execution::operators::scan::SeqScan;
use crate::execution::sandbox::{QueryLimits, Sandbox};
use crate::execution::tuple::Tuple;
use crate::ffi::storage::StorageEngine;
use crate::planner::physical::PhysicalPlan;
use anyhow::Result;
use futures::future::BoxFuture;
use futures::FutureExt;

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
                _ => {
                    anyhow::bail!("Unsupported plan type")
                }
            }
        }.boxed()
    }
          }
