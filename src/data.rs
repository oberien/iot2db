use std::mem;
use std::sync::Arc;
use indexmap::{IndexMap, map::Entry};
use serde_json::Value as JsonValue;
use crate::run_rebo;
use crate::backend::BackendEscaper;
use crate::config::{DirectValues, Mapping, Value as ConfigValue, ValueKind};
use crate::iter_json_value::iter_json_value;

pub trait DataMapper {
    fn new(mapping: Mapping, escaper: Arc<dyn BackendEscaper + Send + Sync + 'static>) -> Self where Self: Sized;
    fn consume_value(&mut self, value: JsonValue) -> Option<IndexMap<String, String>>;
}
pub struct WideToWide {
    mapping: Mapping,
    escaper: Arc<dyn BackendEscaper + Send + Sync + 'static>,
}

// does *not* handle constants
fn get_and_process_values(value: &JsonValue, mapping: &Mapping, escaper: &dyn BackendEscaper) -> IndexMap<String, String> {
    iter_json_value(value).filter_map(|(json_pointer, json_value)| {
        let config_value = mapping.values.iter()
            .find(|(_, value)| matches!(&value.kind, ValueKind::Pointer { pointer } if *pointer == json_pointer));
        let (name, preprocess, postprocess) = match (config_value, &mapping.direct_values) {
            (Some((config_value_name, ConfigValue { kind: ValueKind::Pointer { .. }, preprocess, postprocess, aggregate: _ })), _) => (config_value_name.clone(), preprocess.clone(), postprocess.clone()),
            (Some((_, ConfigValue { kind: ValueKind::Constant { .. }, .. })), _) => return None,
            (None, Some(DirectValues::All(_))) => (json_pointer_to_key(&json_pointer), None, None),
            (None, Some(DirectValues::Keys(keys))) if keys.iter().any(|k| *k == json_pointer) => (json_pointer_to_key(&json_pointer), None, None),
            _ => return None,
        };
        // `String("uiae").to_string()` results in `"\"uiae\""` but we want `"uiae"`
        let val = match json_value {
            JsonValue::String(s) => s.clone(),
            val => val.to_string(),
        };
        Some((name, process_value(val, preprocess, postprocess, escaper)))
    }).collect()
}

fn process_value(val: String, preprocess: Option<String>, postprocess: Option<String>, escaper: &dyn BackendEscaper) -> String {
    let val = match preprocess.clone() {
        Some(preprocess) => run_rebo(preprocess, val),
        None => val,
    };
    let val = escaper.escape_value(val);
    let val = match postprocess.clone() {
        Some(postprocess) => run_rebo(postprocess, val),
        None => val,
    };
    val
}

fn iter_mapped_constants<'a>(mapping: &'a Mapping, escaper: &'a dyn BackendEscaper) -> impl Iterator<Item = (String, String)> + 'a {
    mapping.values.iter().filter_map(|(key, mapping_value)| {
        let val = match &mapping_value.kind {
            ValueKind::Pointer { .. } => return None,
            ValueKind::Constant { constant_value: const_value } => const_value.clone(),
        };
        let val = process_value(val, mapping_value.preprocess.clone(), mapping_value.postprocess.clone(), escaper);
        Some((key.clone(), val))
    })
}

impl DataMapper for WideToWide {
    fn new(mapping: Mapping, escaper: Arc<dyn BackendEscaper + Send + Sync + 'static>) -> Self
    where Self: Sized
    {
        WideToWide { mapping, escaper }
    }

    fn consume_value(&mut self, value: JsonValue) -> Option<IndexMap<String, String>> {
        let mut map = get_and_process_values(&value, &self.mapping, &*self.escaper);
        map.extend(iter_mapped_constants(&self.mapping, &*self.escaper));
        Some(map)
    }
}

pub struct NarrowToWide {
    mapping: Mapping,
    escaper: Arc<dyn BackendEscaper + Send + Sync + 'static>,
    buffered_value: IndexMap<String, String>,
}

impl DataMapper for NarrowToWide {
    fn new(mapping: Mapping, escaper: Arc<dyn BackendEscaper + Send + Sync + 'static>) -> Self
    where Self: Sized
    {
        let values_len = mapping.values.len();
        NarrowToWide { mapping, escaper, buffered_value: IndexMap::with_capacity(values_len) }
    }

    fn consume_value(&mut self, value: JsonValue) -> Option<IndexMap<String, String>> {
        let map = get_and_process_values(&value, &self.mapping, &*self.escaper);
        assert!(map.len() <= 1);
        let (key, value) = map.into_iter().next()?;

        match self.buffered_value.entry(key.clone()) {
            Entry::Vacant(vacant) => { vacant.insert(value); },
            Entry::Occupied(_) => {
                let new = IndexMap::with_capacity(self.mapping.values.len());
                let mut map = mem::replace(&mut self.buffered_value, new);
                self.buffered_value.insert(key.clone(), value);
                // add constants to map
                for (key, mapping_value) in &self.mapping.values {
                    let val = match &mapping_value.kind {
                        ValueKind::Pointer { .. } => continue,
                        ValueKind::Constant { constant_value: const_value } => const_value.clone(),
                    };
                    let val = process_value(val, mapping_value.preprocess.clone(), mapping_value.postprocess.clone(), &*self.escaper);
                    map.insert(key.clone(), val);
                }
                return Some(map)
            }
        }
        None
    }
}

fn json_pointer_to_key(json_pointer: &String) -> String {
    assert_eq!(&json_pointer[..1], "/");
    json_pointer[1..].replace("/", "_").replace("~1", "/").replace("~0", "~")
}
