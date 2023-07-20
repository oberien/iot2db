use std::collections::HashMap;
use crate::{config, run_rebo};
use crate::backend::Backend;

pub fn mapper<'a, B: Backend>(values: &'a HashMap<String, config::Value>) -> impl Fn(serde_json::Value) -> HashMap<&'a str, String> {
    move |value| {
        let mut map = HashMap::with_capacity(values.len());
        for (key, pointer) in values {
            if let Some(val) = value.pointer(&pointer.pointer) {
                let val = val.to_string();
                let val = match pointer.preprocess.clone() {
                    Some(preprocess) => run_rebo(preprocess, val),
                    None => val,
                };
                let val = B::escape_value(val);
                let val = match pointer.postprocess.clone() {
                    Some(postprocess) => run_rebo(postprocess, val),
                    None => val,
                };
                map.insert(key.as_str(), val.to_string());
            }
        }
        map
    }
}