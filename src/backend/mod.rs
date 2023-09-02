use std::sync::Arc;
use indexmap::IndexMap;

pub mod postgres;

pub struct DataToInsert {
    pub escaped_values: IndexMap<String, String>,
    pub persistent_every_secs: Option<u32>,
}

#[async_trait::async_trait]
pub trait Backend {
    type Config;
    type Ref;

    async fn new(config: Self::Config) -> Self;
    async fn escaper(&self) -> Arc<dyn BackendEscaper + Send + Sync + 'static>;
    async fn inserter(&self, r: Self::Ref) -> Arc<dyn BackendInserter + Send + Sync + 'static>;
}

#[async_trait::async_trait]
pub trait BackendInserter {
    async fn insert(&self, data: DataToInsert);
    async fn delete_old_non_persistent(&self, delete_older_than_days: u32);
}

pub trait BackendEscaper {
    fn escape_value(&self, value: String) -> String;
}

pub struct NoopEscaper;

impl BackendEscaper for NoopEscaper {
    fn escape_value(&self, value: String) -> String {
        value
    }
}
