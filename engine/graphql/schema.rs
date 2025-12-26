use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLSchema {
    pub types: HashMap<String, GraphQLType>,
    pub queries: HashMap<String, GraphQLQuery>,
    pub mutations: HashMap<String, GraphQLMutation>,
    pub subscriptions: HashMap<String, GraphQLSubscription>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLType {
    pub name: String,
    pub fields: Vec<GraphQLField>,
    pub table_mapping: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLField {
    pub name: String,
    pub field_type: String,
    pub nullable: bool,
    pub column_mapping: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLQuery {
    pub name: String,
    pub return_type: String,
    pub arguments: Vec<GraphQLArgument>,
    pub sql_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLMutation {
    pub name: String,
    pub return_type: String,
    pub arguments: Vec<GraphQLArgument>,
    pub sql_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLSubscription {
    pub name: String,
    pub return_type: String,
    pub trigger_table: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLArgument {
    pub name: String,
    pub arg_type: String,
    pub required: bool,
}

pub struct SchemaGenerator;

impl SchemaGenerator {
    pub fn generate_from_tables(tables: Vec<String>) -> Result<GraphQLSchema> {
        let mut types = HashMap::new();
        let mut queries = HashMap::new();

        for table in tables {
            let type_name = Self::to_pascal_case(&table);

            let gql_type = GraphQLType {
                name: type_name.clone(),
                fields: vec![GraphQLField {
                    name: "id".to_string(),
                    field_type: "ID".to_string(),
                    nullable: false,
                    column_mapping: Some("id".to_string()),
                }],
                table_mapping: Some(table.clone()),
            };

            types.insert(type_name.clone(), gql_type);

            queries.insert(
                format!("get{}", type_name),
                GraphQLQuery {
                    name: format!("get{}", type_name),
                    return_type: type_name.clone(),
                    arguments: vec![GraphQLArgument {
                        name: "id".to_string(),
                        arg_type: "ID".to_string(),
                        required: true,
                    }],
                    sql_template: format!("retrieve * from {} where id = $id", table),
                },
            );

            queries.insert(
                format!("list{}s", type_name),
                GraphQLQuery {
                    name: format!("list{}s", type_name),
                    return_type: format!("[{}]", type_name),
                    arguments: vec![
                        GraphQLArgument {
                            name: "limit".to_string(),
                            arg_type: "Int".to_string(),
                            required: false,
                        },
                        GraphQLArgument {
                            name: "offset".to_string(),
                            arg_type: "Int".to_string(),
                            required: false,
                        },
                    ],
                    sql_template: format!("retrieve * from {} limit $limit offset $offset", table),
                },
            );
        }

        Ok(GraphQLSchema {
            types,
            queries,
            mutations: HashMap::new(),
            subscriptions: HashMap::new(),
        })
    }

    fn to_pascal_case(s: &str) -> String {
        s.split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect()
    }

    pub fn generate_sdl(schema: &GraphQLSchema) -> String {
        let mut sdl = String::new();

        for gql_type in schema.types.values() {
            sdl.push_str(&format!("type {} {{\n", gql_type.name));
            for field in &gql_type.fields {
                let nullable = if field.nullable { "" } else { "!" };
                sdl.push_str(&format!(
                    "  {}: {}{}\n",
                    field.name, field.field_type, nullable
                ));
            }
            sdl.push_str("}\n\n");
        }

        sdl.push_str("type Query {\n");
        for query in schema.queries.values() {
            let args: Vec<String> = query
                .arguments
                .iter()
                .map(|arg| {
                    let required = if arg.required { "!" } else { "" };
                    format!("{}: {}{}", arg.name, arg.arg_type, required)
                })
                .collect();

            sdl.push_str(&format!(
                "  {}({}): {}\n",
                query.name,
                args.join(", "),
                query.return_type
            ));
        }
        sdl.push_str("}\n");

        sdl
    }
}
