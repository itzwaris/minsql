use crate::execution::tuple::{Tuple, Value};
use crate::language::intent::*;
use anyhow::Result;

pub struct ExpressionEvaluator;

impl ExpressionEvaluator {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate(&self, expr: &ExpressionIntent, tuple: &Tuple) -> Result<Value> {
        match expr {
            ExpressionIntent::Column(name) => Ok(tuple.get(name).cloned().unwrap_or(Value::Null)),
            ExpressionIntent::QualifiedColumn { column, .. } => {
                Ok(tuple.get(column).cloned().unwrap_or(Value::Null))
            }
            ExpressionIntent::Constant(val) => Ok(self.convert_constant(val)),
            ExpressionIntent::Arithmetic { op, left, right } => {
                let left_val = self.evaluate(left, tuple)?;
                let right_val = self.evaluate(right, tuple)?;
                self.eval_arithmetic(op, &left_val, &right_val)
            }
            ExpressionIntent::Function { name, args } => self.eval_function(name, args, tuple),
        }
    }

    pub fn evaluate_filter(&self, filter: &FilterIntent, tuple: &Tuple) -> Result<bool> {
        match filter {
            FilterIntent::Always => Ok(true),
            FilterIntent::Never => Ok(false),
            FilterIntent::Comparison { op, left, right } => {
                let left_val = self.evaluate(left, tuple)?;
                let right_val = self.evaluate(right, tuple)?;
                self.eval_comparison(op, &left_val, &right_val)
            }
            FilterIntent::Logical { op, operands } => match op {
                LogicalOp::And => {
                    for operand in operands {
                        if !self.evaluate_filter(operand, tuple)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
                LogicalOp::Or => {
                    for operand in operands {
                        if self.evaluate_filter(operand, tuple)? {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                }
                LogicalOp::Not => {
                    if operands.len() != 1 {
                        anyhow::bail!("NOT expects exactly one operand");
                    }
                    Ok(!self.evaluate_filter(&operands[0], tuple)?)
                }
            },
        }
    }

    fn eval_arithmetic(&self, op: &ArithmeticOp, left: &Value, right: &Value) -> Result<Value> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => {
                let result = match op {
                    ArithmeticOp::Add => l + r,
                    ArithmeticOp::Subtract => l - r,
                    ArithmeticOp::Multiply => l * r,
                    ArithmeticOp::Divide => {
                        if *r == 0 {
                            anyhow::bail!("Division by zero");
                        }
                        l / r
                    }
                };
                Ok(Value::Integer(result))
            }
            (Value::Float(l), Value::Float(r)) => {
                let result = match op {
                    ArithmeticOp::Add => l + r,
                    ArithmeticOp::Subtract => l - r,
                    ArithmeticOp::Multiply => l * r,
                    ArithmeticOp::Divide => l / r,
                };
                Ok(Value::Float(result))
            }
            _ => anyhow::bail!("Type mismatch in arithmetic operation"),
        }
    }

    fn eval_comparison(&self, op: &ComparisonOp, left: &Value, right: &Value) -> Result<bool> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => Ok(match op {
                ComparisonOp::Equal => l == r,
                ComparisonOp::NotEqual => l != r,
                ComparisonOp::LessThan => l < r,
                ComparisonOp::LessThanOrEqual => l <= r,
                ComparisonOp::GreaterThan => l > r,
                ComparisonOp::GreaterThanOrEqual => l >= r,
            }),
            (Value::Float(l), Value::Float(r)) => Ok(match op {
                ComparisonOp::Equal => (l - r).abs() < f64::EPSILON,
                ComparisonOp::NotEqual => (l - r).abs() >= f64::EPSILON,
                ComparisonOp::LessThan => l < r,
                ComparisonOp::LessThanOrEqual => l <= r,
                ComparisonOp::GreaterThan => l > r,
                ComparisonOp::GreaterThanOrEqual => l >= r,
            }),
            (Value::String(l), Value::String(r)) => Ok(match op {
                ComparisonOp::Equal => l == r,
                ComparisonOp::NotEqual => l != r,
                ComparisonOp::LessThan => l < r,
                ComparisonOp::LessThanOrEqual => l <= r,
                ComparisonOp::GreaterThan => l > r,
                ComparisonOp::GreaterThanOrEqual => l >= r,
            }),
            _ => anyhow::bail!("Type mismatch in comparison"),
        }
    }

    fn eval_function(&self, name: &str, args: &[ExpressionIntent], tuple: &Tuple) -> Result<Value> {
        match name.to_lowercase().as_str() {
            "count" => Ok(Value::Integer(1)),
            "sum" => {
                if args.is_empty() {
                    return Ok(Value::Integer(0));
                }
                let val = self.evaluate(&args[0], tuple)?;
                Ok(val)
            }
            "avg" => {
                if args.is_empty() {
                    return Ok(Value::Float(0.0));
                }
                let val = self.evaluate(&args[0], tuple)?;
                Ok(val)
            }
            _ => anyhow::bail!("Unknown function: {}", name),
        }
    }

    fn convert_constant(&self, val: &ConstantValue) -> Value {
        match val {
            ConstantValue::Null => Value::Null,
            ConstantValue::Boolean(b) => Value::Boolean(*b),
            ConstantValue::Integer(i) => Value::Integer(*i),
            ConstantValue::Float(f) => Value::Float(*f),
            ConstantValue::String(s) => Value::String(s.clone()),
        }
    }
}
