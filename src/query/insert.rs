use crate::{Dialect, Select, ToSql};
use crate::util::SqlExtension;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OnConflict {
    Ignore,
    Abort,
    // Replace,
}

impl Default for OnConflict {
    fn default() -> Self {
        OnConflict::Abort
    }
}

impl ToSql for Values {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        match self {
            Values::Values(values) => {
                let mut first_value = true;
                for value in values {
                    if !first_value {
                        buf.push_str(", ");
                    }
                    let mut first = true;
                    buf.push('(');
                    for v in value {
                        if !first {
                            buf.push_str(", ");
                        }
                        buf.push_str(&v);
                        first = false;
                    }
                    buf.push(')');
                    first_value = false;
                }
            }
            Values::Select(select) => {
                buf.push_sql(select, dialect);
            }
            Values::DefaultValues => {
                buf.push_str("DEFAULT VALUES");
            }
        }
    }
}

pub enum Values {
    Values(Vec<Vec<String>>),
    Select(Select),
    DefaultValues,
}

pub struct Insert {
    pub schema: Option<String>,
    pub table: String,
    pub columns: Vec<String>,
    pub values: Values,
    pub on_conflict: OnConflict,
    pub returning: Vec<String>,
}

impl ToSql for Insert {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        use Dialect::*;
        use OnConflict::*;
        if dialect == Sqlite {
            match self.on_conflict {
                Ignore => buf.push_str("INSERT OR IGNORE INTO "),
                Abort => buf.push_str("INSERT OR ABORT INTO "),
            }
        } else {
            buf.push_str("INSERT INTO ");
        }
        buf.push_table_name(&self.schema, &self.table, None);
        buf.push_str(" (");
        let mut first = true;
        for c in &self.columns {
            if first {
                first = false;
            } else {
                buf.push_str(", ");
            }
            buf.push_quoted(c);
        }
        buf.push_str(") VALUES ");
        self.values.write_sql(buf, dialect);

        if !self.returning.is_empty() {
            buf.push_str(" RETURNING ");
            let mut first = true;
            for column in &self.returning {
                if !first {
                    buf.push_str(", ");
                }
                buf.push_quoted(column);
                first = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let insert = Insert {
            schema: None,
            table: "foo".to_string(),
            columns: vec!["bar".to_string(), "baz".to_string()],
            values: Values::Values(vec![
                vec!["1".to_string(), "2".to_string()],
                vec!["3".to_string(), "4".to_string()],
            ]),
            on_conflict: OnConflict::Abort,
            returning: vec!["id".to_string()],
        };
        assert_eq!(
            insert.to_sql(Dialect::Postgres),
            r#"INSERT INTO "foo" ("bar", "baz") VALUES (1, 2), (3, 4) RETURNING "id""#
        );
    }
}