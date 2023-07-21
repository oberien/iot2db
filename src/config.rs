use std::collections::HashMap;
use std::convert::Infallible;
use std::str::FromStr;
use serde::Deserialize;
use serde_with::serde_as;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub frontend: HashMap<String, FrontendConfig>,
    pub backend: HashMap<String, BackendConfig>,
    pub data: HashMap<String, DataConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum FrontendConfig {
    HttpRest(HttpRestConfig),
    Mqtt(MqttConfig),
}
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum BackendConfig {
    Postgres(PostgresConfig),
}

// frontends
#[derive(Debug, Clone, Deserialize)]
pub struct HttpRestConfig {
    pub url: String,
    pub basic_auth: Option<BasicAuth>,
    pub frequency_secs: u32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct BasicAuth {
    pub username: String,
    pub password: Option<String>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct MqttConfig {
    pub host: String,
    #[serde(default = "default_mqtt_port")]
    pub port: u16,
    pub auth: Option<MqttAuth>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct MqttAuth {
    pub username: String,
    pub password: String,
}

// backends
#[derive(Debug, Clone, Deserialize)]
pub struct PostgresConfig {
    pub host: String,
    #[serde(default = "default_postgres_port")]
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: Option<String>,
}

// data
#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct DataConfig {
    pub frontend: FrontendRef,
    pub backend: BackendRef,
    pub persistent_every_secs: Option<u32>,
    pub clean_non_persistent_after_secs: Option<u32>,
    #[serde_as(as = "HashMap<_, serde_with::PickFirst<(serde_with::DisplayFromStr, _)>>")]
    pub values: HashMap<String, Value>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Value {
    pub pointer: String,
    /// rebo code taking `value`-string before escaping and returning its replacement-string
    pub preprocess: Option<String>,
    /// rebo code taking `value`-string after escaping and returning its replacement-string
    pub postprocess: Option<String>,
    #[serde(default)]
    pub aggregate: Aggregate,
}
#[derive(Debug, Clone, Deserialize)]
pub enum Aggregate {
    None,
    IncrementingValueWhichMayReset,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum FrontendRef {
    Mqtt { name: String, mqtt_topic: String },
    HttpRest { name: String },
}
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum BackendRef {
    Postgres(PostgresRef),
}
#[derive(Debug, Clone, Deserialize)]
pub struct PostgresRef {
    pub name: String,
    pub postgres_table: String,
}



impl FromStr for Value {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Value {
            pointer: s.to_string(),
            preprocess: None,
            postprocess: None,
            aggregate: Aggregate::default(),
        })
    }
}
impl Default for Aggregate {
    fn default() -> Self {
        Aggregate::None
    }
}

fn default_mqtt_port() -> u16 { 1883 }
fn default_postgres_port() -> u16 { 5432 }
