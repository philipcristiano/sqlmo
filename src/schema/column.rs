use crate::query::Expr;
use crate::util::SqlExtension;
use crate::{Dialect, ToSql, Type};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Column {
    pub name: String,
    pub typ: Type,
    pub nullable: bool,
    pub primary_key: bool,
    pub default: Option<Expr>,
}

impl ToSql for Column {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        buf.push_quoted(&self.name);
        buf.push(' ');
        buf.push_str(&self.typ.to_sql(dialect));
        if !self.nullable {
            buf.push_str(" NOT NULL");
        }
        if self.primary_key {
            buf.push_str(" PRIMARY KEY");
        }
        if let Some(default) = &self.default {
            buf.push_str(" DEFAULT ");
            buf.push_sql(default, dialect);
        }
    }
}
