use std::collections::HashMap;
use crate::{config, run_rebo};
use crate::backend::Escaper;

pub fn mapper(values: HashMap<String, config::Value>, escaper: impl Escaper + 'static) -> impl Fn(serde_json::Value) -> HashMap<String, String> + 'static {
    move |value| {
        let values = values.clone();
        let mut map = HashMap::with_capacity(values.len());
        for (key, pointer) in values {
            if let Some(val) = value.pointer(&pointer.pointer) {
                let val = val.to_string();
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
        }
        map
    }
}