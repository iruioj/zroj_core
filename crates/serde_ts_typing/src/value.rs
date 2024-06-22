use std::collections::BTreeMap;

/// JSON value, modified from `serde_json::Value`
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    /// Represents a JSON null value.
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// let v = json!(null);
    /// ```
    Null,

    /// Represents a JSON boolean.
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// let v = json!(true);
    /// ```
    Bool(bool),

    /// Represents a JSON number, whether integer or floating point.
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// let v = json!(12.5);
    /// ```
    ///
    /// for convenience, restrict to int
    Number(i64),

    /// Represents a JSON string.
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// let v = json!("a string");
    /// ```
    String(String),

    /// Represents a JSON array.
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// let v = json!(["an", "array"]);
    /// ```
    Array(Vec<Value>),

    /// Represents a JSON object.
    ///
    /// By default the map is backed by a BTreeMap. Enable the `preserve_order`
    /// feature of serde_json to use IndexMap instead, which preserves
    /// entries in the order they are inserted into the map. In particular, this
    /// allows JSON data to be deserialized into a Value and serialized to a
    /// string while retaining the order of map keys in the input.
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// let v = json!({ "an": "object" });
    /// ```
    Object(BTreeMap<String, Value>),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => f.write_str("null"),
            Value::Bool(b) => f.write_str({
                if *b {
                    "true"
                } else {
                    "false"
                }
            }),
            Value::Number(n) => n.fmt(f),
            Value::String(s) => write!(f, "\"{s}\""),
            Value::Array(a) => {
                let mut sep = "";
                write!(
                    f,
                    "{}",
                    a.iter().fold(String::from("["), |acc, cur| {
                        let r = acc + sep + &cur.to_string();
                        sep = ",";
                        r
                    }) + "]"
                )
            }
            Value::Object(o) => {
                let mut sep = "";
                write!(
                    f,
                    "{}",
                    o.iter().fold(String::from("{"), |acc, (k, v)| {
                        let r = acc + sep + k + ":" + &v.to_string();
                        sep = ",";
                        r
                    }) + "}"
                )
            }
        }
    }
}