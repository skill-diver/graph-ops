use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum RowCell {
    String(String),
    Float(f32),
    Double(f64),
    Int(i64),
    Boolean(bool),
    Date(String),
    Time(String),
    DateTime(String),
    Duration(u64),
    Array(Vec<RowCell>),
    Null,
}

impl RowCell {
    pub fn string(&self) -> &String {
        if let RowCell::String(str) = self {
            str
        } else {
            panic!("Value is not String");
        }
    }

    pub fn double(&self) -> f64 {
        if let RowCell::Double(v) = self {
            *v
        } else {
            panic!("Value is not Double");
        }
    }
}

impl std::fmt::Display for RowCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RowCell::String(val) => f.write_fmt(format_args!("{val}")),
            RowCell::Float(val) => f.write_fmt(format_args!("{val}")),
            RowCell::Double(val) => f.write_fmt(format_args!("{val}")),
            RowCell::Int(val) => f.write_fmt(format_args!("{val}")),
            RowCell::Boolean(val) => f.write_fmt(format_args!("{val}")),
            RowCell::Date(val) => f.write_fmt(format_args!("{val}")),
            RowCell::Time(val) => f.write_fmt(format_args!("{val}")),
            RowCell::DateTime(val) => f.write_fmt(format_args!("{val}")),
            RowCell::Duration(val) => f.write_fmt(format_args!("{val}")),
            RowCell::Array(val) => f.write_str(
                &val.iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            ),
            RowCell::Null => f.write_str(""),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Row {
    fields: Vec<RowCell>,
}

impl Row {
    pub fn new(fields: Vec<RowCell>) -> Self {
        Self { fields }
    }

    pub fn get(&self, index: usize) -> &RowCell {
        &self.fields[index]
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}
