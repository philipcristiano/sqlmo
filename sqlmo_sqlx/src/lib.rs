use std::str::FromStr;
use anyhow::{Error, Result};
use itertools::Itertools;
use sqlx::PgConnection;
use async_trait::async_trait;

use sqlmo::{Schema, Column, Table, schema};

const QUERY_COLUMNS: &str = include_str!("sql/query_columns.sql");
const QUERY_TABLES: &str = include_str!("sql/query_tables.sql");
const QUERY_TABLE_FK_CONSTRAINTS: &str = include_str!("sql/query_table_fk_constraints.sql");

#[async_trait]
pub trait FromPostgres: Sized {
    async fn try_from_postgres(conn: &mut PgConnection, schema_name: &str) -> Result<Self>;
}

#[derive(sqlx::FromRow)]
struct SchemaColumn {
    pub table_name: String,
    pub column_name: String,
    #[allow(dead_code)]
    pub ordinal_position: i32,
    pub is_nullable: String,
    pub data_type: String,
    pub numeric_precision: Option<i32>,
    pub numeric_scale: Option<i32>,
    pub inner_type: Option<String>,
}

async fn query_schema_columns(conn: &mut PgConnection, schema_name: &str) -> Result<Vec<SchemaColumn>> {
    let result = sqlx::query_as::<_, SchemaColumn>(QUERY_COLUMNS)
        .bind(schema_name)
        .fetch_all(conn)
        .await?;
    Ok(result)
}

#[derive(sqlx::FromRow)]
struct TableSchema {
    #[allow(dead_code)]
    pub table_schema: String,
    pub table_name: String,
}

async fn query_table_names(conn: &mut PgConnection, schema_name: &str) -> Result<Vec<String>> {
    let result = sqlx::query_as::<_, TableSchema>(QUERY_TABLES)
        .bind(schema_name)
        .fetch_all(conn)
        .await?;
    Ok(result.into_iter().map(|t| t.table_name).collect())
}

#[derive(sqlx::FromRow)]
struct TableConstraintSchema {
    #[allow(dead_code)]
    pub conname: String,
    pub definition: String,
}
async fn query_table_fk_contraints(conn: &mut PgConnection, schema_name: &str, table_name: &str) -> Result<Vec<TableConstraintSchema>> {
    let name = format!("{schema_name}.{table_name}");
    let result = sqlx::query_as::<_, TableConstraintSchema>(QUERY_TABLE_FK_CONSTRAINTS)
        .bind(name)
        .fetch_all(conn)
        .await?;
    Ok(result)
}


impl TryInto<Column> for SchemaColumn {
    type Error = Error;

    fn try_into(self) -> std::result::Result<Column, Self::Error> {
        use schema::Type::*;
        let nullable = self.is_nullable == "YES";
        let typ = match self.data_type.as_str() {
            "ARRAY" => {
                let inner = schema::Type::from_str(&self.inner_type.expect("Encounterd ARRAY with no inner type."))?;
                Array(Box::new(inner))
            }
            "numeric" if self.numeric_precision.is_some() && self.numeric_scale.is_some() => {
                Numeric(self.numeric_precision.unwrap() as u8, self.numeric_scale.unwrap() as u8)
            }
            z => schema::Type::from_str(z)?,
        };
        Ok(Column {
            name: self.column_name.clone(),
            typ,
            nullable,
            primary_key: false,
            default: None,
        })
    }
}

#[async_trait]
impl FromPostgres for Schema {
    async fn try_from_postgres(conn: &mut PgConnection, schema_name: &str) -> Result<Schema> {
        let column_schemas = query_schema_columns(conn, schema_name).await?;
        let mut tables = column_schemas.into_iter()
            .group_by(|c| c.table_name.clone())
            .into_iter()
            .map(|(table_name, group)| {
                let columns = group
                    .map(|c: SchemaColumn| c.try_into())
                    .collect::<Result<Vec<_>, Error>>()?;
                Ok(Table {
                    schema: Some(schema_name.to_string()),
                    name: table_name,
                    columns,
                    indexes: vec![],
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;

        // Degenerate case but you can have tables with no columns...
        let table_names = query_table_names(conn, schema_name).await?;
        for name in table_names {
            if tables.iter().any(|t| t.name == name) {
                continue;
            }
            let t = Table {
                schema: Some(schema_name.to_string()),
                name,
                columns: vec![],
                indexes: vec![],
            };

            tables.push(t)
        }

        for t in tables.clone() {
            println!("q {:?} {}", &schema_name, t.name.clone());
            let fk_constraints = query_table_fk_contraints(conn, &schema_name, t.name.clone().as_str()).await?;
            println!("fk {:?}", fk_constraints.len());

            let dialect = sqlparser::dialect::PostgreSqlDialect {};

            for fkc in fk_constraints {
                let parser = sqlparser::parser::Parser::new(&dialect);
                let mut parser = parser.try_with_sql(&fkc.definition)?;
                let ast = parser.parse_optional_table_constraint()?;
                println!("{ast:?}")


            }
        }
        Ok(Schema { tables })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_numeric() {
        let c = SchemaColumn {
            table_name: "foo".to_string(),
            column_name: "bar".to_string(),
            ordinal_position: 1,
            is_nullable: "NO".to_string(),
            data_type: "numeric".to_string(),
            numeric_precision: Some(10),
            numeric_scale: Some(2),
            inner_type: None,
        };
        let column: Column = c.try_into().unwrap();
        assert_eq!(column.typ, schema::Type::Numeric(10, 2));
    }

    #[test]
    fn test_integer() {
        let c = SchemaColumn {
            table_name: "foo".to_string(),
            column_name: "bar".to_string(),
            ordinal_position: 1,
            is_nullable: "NO".to_string(),
            data_type: "integer".to_string(),
            numeric_precision: Some(32),
            numeric_scale: Some(0),
            inner_type: None,
        };
        let column: Column = c.try_into().unwrap();
        assert_eq!(column.typ, schema::Type::I32);
    }
}
