use serde_json::Value as JsonValue;
use genawaiter::{yield_, rc::gen};

pub fn iter_json_value(value: &JsonValue) -> impl Iterator<Item = (String, &JsonValue)> {
    iter_json_value_in_pointer(String::new(), value)
}

fn iter_json_value_in_pointer(pointer: String, value: &JsonValue) -> impl Iterator<Item = (String, &JsonValue)> {
    let generator = gen!({
        yield_!((pointer.clone(), value));
        match value {
            JsonValue::Null | JsonValue::Bool(_) | JsonValue::Number(_) | JsonValue::String(_) => (),
            JsonValue::Array(array) => {
                for (i, val) in array.iter().enumerate() {
                    let pointer = format!("{pointer}/{i}");
                    for item in iter_json_value_in_pointer(pointer.clone(), val) {
                        yield_!(item)
                    }
                }
            },
            JsonValue::Object(map) => {
                for (key, val) in map.iter() {
                    // json-pointer escape the key
                    let key = key.replace("~", "~0").replace("/", "~1");
                    let pointer = format!("{pointer}/{key}");
                    for item in iter_json_value_in_pointer(pointer.clone(), val) {
                        yield_!(item)
                    }
                }
            },
        }
    });
    generator.into_iter()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn iter_json_value_test() {
        let val = serde_json::from_str(r#"
            {
                "bar": 42,
                "foo": {
                    "baz": [1, 2, "3"],
                    "qux": "corge"
                }
            }
        "#).unwrap();
        // serde_json::Value objects use a BTreeMap by default -> keys are sorted
        let mut iter = iter_json_value(&val);
        let (pointer, value) = iter.next().unwrap();
        assert_eq!(pointer, "");
        assert!(value.is_object());
        let (pointer, value) = iter.next().unwrap();
        assert_eq!(pointer, "/bar");
        assert_eq!(value, 42);
        let (pointer, value) = iter.next().unwrap();
        assert_eq!(pointer, "/foo");
        assert!(value.is_object());
        let (pointer, value) = iter.next().unwrap();
        assert_eq!(pointer, "/foo/baz");
        assert!(value.is_array());
        let (pointer, value) = iter.next().unwrap();
        assert_eq!(pointer, "/foo/baz/0");
        assert_eq!(value, 1);
        let (pointer, value) = iter.next().unwrap();
        assert_eq!(pointer, "/foo/baz/1");
        assert_eq!(value, 2);
        let (pointer, value) = iter.next().unwrap();
        assert_eq!(pointer, "/foo/baz/2");
        assert_eq!(value, "3");
        let (pointer, value) = iter.next().unwrap();
        assert_eq!(pointer, "/foo/qux");
        assert_eq!(value, "corge");
    }
}