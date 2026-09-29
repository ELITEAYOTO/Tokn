use std::collections::BTreeMap;

use serde_json::Value;

#[derive(Debug, Default, Clone)]
pub struct SchemaInspector {
    pub top_types: BTreeMap<String, u64>,
    pub payload_types: BTreeMap<String, u64>,
    pub unknown_top_level: u64,
}

impl SchemaInspector {
    pub fn observe(&mut self, value: &Value) {
        let top = value
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("<unknown>");
        *self.top_types.entry(top.to_string()).or_default() += 1;

        if top == "<unknown>" {
            self.unknown_top_level += 1;
        }

        if let Some(kind) = value
            .get("payload")
            .and_then(|v| v.get("type"))
            .and_then(Value::as_str)
        {
            *self.payload_types.entry(kind.to_string()).or_default() += 1;
        }
    }
}
