use std::mem;
use std::sync::Arc;
use indexmap::{IndexMap, map::Entry};
use serde_json::Value;
use crate::{config, run_rebo};
use crate::backend::BackendEscaper;

pub trait DataMapper {
    fn new(values: IndexMap<String, config::Value>, escaper: Arc<dyn BackendEscaper + Send + Sync + 'static>) -> Self where Self: Sized;
    fn consume_value(&mut self, value: Value) -> Option<IndexMap<String, String>>;
}
pub struct WideToWide {
    values: IndexMap<String, config::Value>,
    escaper: Arc<dyn BackendEscaper + Send + Sync + 'static>,
}

fn get_and_process_value(value: &Value, config_value: &config::Value, escaper: &dyn BackendEscaper) -> Option<String> {
    let val = value.pointer(&config_value.pointer)?;
    // `String("uiae").to_string()` results in `"\"uiae\""` but we want `"uiae"`
    let val = match val {
        Value::String(s) => s.clone(),
        val => val.to_string(),
    };
    let val = match config_value.preprocess.clone() {
        Some(preprocess) => run_rebo(preprocess, val),
        None => val,
    };
    let val = escaper.escape_value(val);
    let val = match config_value.postprocess.clone() {
        Some(postprocess) => run_rebo(postprocess, val),
        None => val,
    };
    Some(val)
}

impl DataMapper for WideToWide {
    fn new(values: IndexMap<String, config::Value>, escaper: Arc<dyn BackendEscaper + Send + Sync + 'static>) -> Self
    where Self: Sized
    {
        WideToWide { values, escaper }
    }

    fn consume_value(&mut self, value: Value) -> Option<IndexMap<String, String>> {
        let values = self.values.clone();
        let mut map = IndexMap::with_capacity(values.len());
        for (key, config_value) in &values {
            let val = get_and_process_value(&value, &config_value, &*self.escaper)
                .unwrap_or_else(|| "null".to_string());
            map.insert(key.clone(), val);
        }
        Some(map)
    }
}

pub struct NarrowToWide {
    values: IndexMap<String, config::Value>,
    escaper: Arc<dyn BackendEscaper + Send + Sync + 'static>,
    buffered_value: IndexMap<String, String>,
}

impl DataMapper for NarrowToWide {
    fn new(values: IndexMap<String, config::Value>, escaper: Arc<dyn BackendEscaper + Send + Sync + 'static>) -> Self
    where Self: Sized
    {
        let values_len = values.len();
        NarrowToWide { values, escaper, buffered_value: IndexMap::with_capacity(values_len) }
    }

    fn consume_value(&mut self, value: Value) -> Option<IndexMap<String, String>> {
        let values = self.values.clone();
        for (key, config_value) in &values {
            let Some(val) = get_and_process_value(&value, &config_value, &*self.escaper) else { continue };

            match self.buffered_value.entry(key.clone()) {
                Entry::Vacant(vacant) => { vacant.insert(val); },
                Entry::Occupied(_) => {
                    let map = mem::replace(&mut self.buffered_value, IndexMap::with_capacity(self.values.len()));
                    self.buffered_value.insert(key.clone(), val);
                    return Some(map)
                }
            }
        }
        None
    }
}
