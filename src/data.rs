use std::collections::HashMap;
use crate::config;

pub fn mapper<'a>(values: &'a HashMap<String, config::Value>) -> impl Fn(serde_json::Value) -> HashMap<&'a str, String> {
    move |value| {
        let mut map = HashMap::with_capacity(values.len());
        for (key, pointer) in values {
            if let Some(val) = value.pointer(&pointer.pointer) {
                map.insert(key.as_str(), val.to_string());
            }
        }
        map
    }
}