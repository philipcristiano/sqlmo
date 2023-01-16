#[cfg(feature = "sqlx")]
mod from_postgres;
#[cfg(feature = "openapi")]
mod from_openapi;
mod table;
mod r#type;
mod column;
mod index;


pub use column::Column;
pub use r#type::Type;
pub use table::Table;

use anyhow::Result;
use crate::migrate::{Migration, migrate, MigrationOptions};

/// Represents a SQL database schema.
#[derive(Debug)]
pub struct Schema {
    pub tables: Vec<Table>,
}

impl Default for Schema {
    fn default() -> Self {
        Schema {
            tables: vec![],
        }
    }
}

impl Schema {
    /// Calculate the migration necessary to move from `self: Schema` to the argument `desired: Schema`.
    pub fn migrate_to(self, desired: Schema, options: &MigrationOptions) -> Result<Migration> {
        migrate(self, desired, options)
    }

    /// Propagate the schema name to all tables.
    pub fn name_schema(&mut self, schema: &str) {
        for table in &mut self.tables {
            table.schema = Some(schema.to_string());
        }
    }
}