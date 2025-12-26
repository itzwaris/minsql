use crate::execution::tuple::Tuple;
use crate::graphql::schema::GraphQLSchema;
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

pub struct GraphQLResolver {
    schema: GraphQLSchema,
}

impl GraphQLResolver {
    pub fn new(schema: GraphQLSchema) -> Self {
        Self { schema }
    }

    pub async fn resolve_query(
        &self,
        query_name: &str,
        arguments: HashMap<String, Value>,
    ) -> Result<Value> {
        let query = self.schema.queries.get(query_name)
            .ok_or_else(|| anyhow::anyhow!("Query not found: {}", query_name))?;

        let sql = self.build_sql(&query.sql_template, &arguments)?;

        Ok(Value::Null)
    }

    fn build_sql(&self, template: &str, arguments: &HashMap<String, Value>) -> Result<String> {
        let mut sql = template.to_string();

        for (key, value) in arguments {
            let placeholder = format!("${}", key);
            let value_str = match value {
                Value::String(s) => format!("'{}'", s),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => value.to_string(),
            };
            sql = sql.replace(&placeholder, &value_str);
        }

        Ok(sql)
    }

    pub fn tuple_to_json(&self, tuple: &Tuple, type_name: &str) -> Result<Value> {
        let gql_type = self.schema.types.get(type_name)
            .ok_or_else(|| anyhow::anyhow!("Type not found: {}", type_name))?;

        let mut obj = serde_json::Map::new();

        for field in &gql_type.fields {
            let column_name = field.column_mapping.as_ref().unwrap_or(&field.name);
            
            if let Some(value) = tuple.get(column_name) {
                let json_value = match value {
                    crate::execution::tuple::Value::Null => Value::Null,
                    crate::execution::tuple::Value::Boolean(b) => Value::Bool(*b),
                    crate::execution::tuple::Value::Integer(i) => Value::Number((*i).into()),
                    crate::execution::tuple::Value::Float(f) => {
                        serde_json::Number::from_f64(*f)
                            .map(Value::Number)
                            .unwrap_or(Value::Null)
                    }
                    crate::execution::tuple::Value::String(s) => Value::String(s.clone()),
                };
                obj.insert(field.name.clone(), json_value);
            }
        }

        Ok(Value::Object(obj))
    }
}
