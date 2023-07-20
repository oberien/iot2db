use std::collections::HashMap;

pub mod postgres;

#[async_trait::async_trait]
pub trait Backend {
    type Config;
    type Ref;

    async fn new(config: &Self::Config, r: &Self::Ref) -> Self;
    async fn consume(&self, escaped_values: HashMap<&str, String>);

    fn escape_value(value: String) -> String;
}
