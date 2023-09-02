use std::sync::Arc;
use indexmap::IndexMap;
use serde_json::Value;
use crate::{config, run_rebo};
use crate::backend::BackendEscaper;

pub fn mapper(values: IndexMap<String, config::Value>, escaper: Arc<dyn BackendEscaper + Send + Sync + 'static>) -> impl Fn(Value) -> IndexMap<String, String> + 'static {
    move |value| {
        let values = values.clone();
        let mut map = IndexMap::with_capacity(values.len());
        for (key, pointer) in values {
            let val = value.pointer(&pointer.pointer).unwrap_or(&Value::Null);
            // `String("uiae").to_string()` results in `"\"uiae\""` but we want `"uiae"`
            let val = match val {
                Value::String(s) => s.clone(),
                val => val.to_string(),
            };
            let val = match pointer.preprocess.clone() {
                Some(preprocess) => run_rebo(preprocess, val),
                None => val,
            };
            let val = escaper.escape_value(val);
            let val = match pointer.postprocess.clone() {
                Some(postprocess) => run_rebo(postprocess, val),
                None => val,
            };
            map.insert(key.clone(), val.to_string());
        }
        map
    }
}